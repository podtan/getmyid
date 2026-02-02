//! Synchronous client for the whoami daemon.

use std::io::Read;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::error::{GetMyIdError, Result};
use crate::types::{DaemonResponse, Identity, ResponseData};

/// Default socket path for the whoami daemon.
pub const DEFAULT_SOCKET_PATH: &str = "/var/run/whoami.sock";

/// Default timeout for connections.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

/// Synchronous client for communicating with the whoami daemon.
///
/// # Example
///
/// ```no_run
/// use getmyid::Client;
///
/// let client = Client::new();
/// let identity = client.get_identity()?;
/// println!("Identity: {}", identity.identity);
/// # Ok::<(), getmyid::GetMyIdError>(())
/// ```
#[derive(Debug, Clone)]
pub struct Client {
    socket_path: PathBuf,
    timeout: Option<Duration>,
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}

impl Client {
    /// Create a new client with default settings.
    ///
    /// Uses the default socket path `/var/run/whoami.sock`.
    pub fn new() -> Self {
        Self {
            socket_path: PathBuf::from(DEFAULT_SOCKET_PATH),
            timeout: Some(DEFAULT_TIMEOUT),
        }
    }

    /// Create a client builder for custom configuration.
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    /// Get the identity of the current process.
    ///
    /// Connects to the whoami daemon, which uses `SO_PEERCRED` to identify
    /// this process, then matches against configured rules to return the
    /// application-level identity.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The socket does not exist or cannot be connected to
    /// - The daemon returns an error (e.g., no matching rule)
    /// - The response cannot be parsed
    pub fn get_identity(&self) -> Result<Identity> {
        // Check socket exists
        if !self.socket_path.exists() {
            return Err(GetMyIdError::SocketNotFound(self.socket_path.clone()));
        }

        // Connect to the socket
        let mut stream = UnixStream::connect(&self.socket_path).map_err(|e| {
            GetMyIdError::ConnectionFailed {
                path: self.socket_path.clone(),
                source: e,
            }
        })?;

        // Set timeouts if configured
        if let Some(timeout) = self.timeout {
            stream
                .set_read_timeout(Some(timeout))
                .map_err(GetMyIdError::ReadError)?;
            stream
                .set_write_timeout(Some(timeout))
                .map_err(GetMyIdError::WriteError)?;
        }

        // The daemon responds immediately after connection with the identity.
        // No request needs to be sent - just read the response.
        let mut response = String::new();
        stream
            .read_to_string(&mut response)
            .map_err(GetMyIdError::ReadError)?;

        // Parse and validate response
        parse_response(&response)
    }

    /// Get the configured socket path.
    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }

    /// Get the configured timeout.
    pub fn timeout(&self) -> Option<Duration> {
        self.timeout
    }
}

/// Builder for creating a customized [`Client`].
///
/// # Example
///
/// ```no_run
/// use std::time::Duration;
/// use getmyid::Client;
///
/// let client = Client::builder()
///     .socket_path("/tmp/whoami.sock")
///     .timeout(Duration::from_secs(10))
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct ClientBuilder {
    socket_path: PathBuf,
    timeout: Option<Duration>,
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ClientBuilder {
    /// Create a new builder with default settings.
    pub fn new() -> Self {
        Self {
            socket_path: PathBuf::from(DEFAULT_SOCKET_PATH),
            timeout: Some(DEFAULT_TIMEOUT),
        }
    }

    /// Set the socket path.
    pub fn socket_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.socket_path = path.as_ref().to_path_buf();
        self
    }

    /// Set the connection timeout.
    ///
    /// Pass `None` to disable timeouts.
    pub fn timeout(mut self, timeout: impl Into<Option<Duration>>) -> Self {
        self.timeout = timeout.into();
        self
    }

    /// Build the client.
    pub fn build(self) -> Client {
        Client {
            socket_path: self.socket_path,
            timeout: self.timeout,
        }
    }
}

/// Parse a response string from the daemon into an Identity.
pub(crate) fn parse_response(response: &str) -> Result<Identity> {
    let daemon_response: DaemonResponse =
        serde_json::from_str(response).map_err(GetMyIdError::InvalidJson)?;

    if !daemon_response.is_ok() {
        match daemon_response.data {
            ResponseData::Error {
                error_code,
                message,
            } => {
                return Err(GetMyIdError::DaemonError {
                    code: error_code,
                    message,
                });
            }
            _ => {
                return Err(GetMyIdError::MissingField { field: "error_code" });
            }
        }
    }

    match daemon_response.data {
        ResponseData::Success {
            identity,
            idm_url,
            config_url,
            token,
            pid,
            uid,
            gid,
            process,
        } => Ok(Identity {
            identity,
            idm_url,
            config_url,
            token,
            pid,
            uid,
            gid,
            process,
        }),
        ResponseData::Error { .. } => Err(GetMyIdError::MissingField { field: "identity" }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_success_response() {
        let response = r#"{"status":"ok","identity":"BILLING_PROD","idm_url":"https://auth.example.com/oauth2/billing","config_url":"https://config.example.com/api/billing","token":"tok_billing_xxx","pid":1234,"uid":1001,"gid":1001,"process":"billing-app"}"#;
        
        let identity = parse_response(response).unwrap();
        
        assert_eq!(identity.identity, "BILLING_PROD");
        assert_eq!(identity.idm_url, "https://auth.example.com/oauth2/billing");
        assert_eq!(identity.config_url, "https://config.example.com/api/billing");
        assert_eq!(identity.token, "tok_billing_xxx");
        assert_eq!(identity.pid, 1234);
        assert_eq!(identity.uid, 1001);
        assert_eq!(identity.gid, 1001);
        assert_eq!(identity.process, "billing-app");
    }

    #[test]
    fn test_parse_error_response() {
        let response = r#"{"status":"error","error_code":"E_NO_MATCH","message":"No identity rule matches process 'unknown' (uid=1000)"}"#;
        
        let result = parse_response(response);
        
        match result {
            Err(GetMyIdError::DaemonError { code, message }) => {
                assert_eq!(code, "E_NO_MATCH");
                assert!(message.contains("No identity rule matches"));
            }
            _ => panic!("Expected DaemonError"),
        }
    }

    #[test]
    fn test_parse_invalid_json() {
        let response = "not json";
        
        let result = parse_response(response);
        
        assert!(matches!(result, Err(GetMyIdError::InvalidJson(_))));
    }

    #[test]
    fn test_client_builder() {
        let client = Client::builder()
            .socket_path("/tmp/test.sock")
            .timeout(Duration::from_secs(10))
            .build();
        
        assert_eq!(client.socket_path(), Path::new("/tmp/test.sock"));
        assert_eq!(client.timeout(), Some(Duration::from_secs(10)));
    }

    #[test]
    fn test_client_builder_no_timeout() {
        let client = Client::builder()
            .timeout(None)
            .build();
        
        assert_eq!(client.timeout(), None);
    }

    #[test]
    fn test_default_client() {
        let client = Client::new();
        
        assert_eq!(client.socket_path(), Path::new(DEFAULT_SOCKET_PATH));
        assert_eq!(client.timeout(), Some(DEFAULT_TIMEOUT));
    }
}
