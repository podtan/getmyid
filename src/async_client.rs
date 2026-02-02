//! Asynchronous client for the whoami daemon (requires `tokio` feature).

use std::path::{Path, PathBuf};
use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

use crate::client::{parse_response, DEFAULT_SOCKET_PATH, DEFAULT_TIMEOUT};
use crate::error::{GetMyIdError, Result};
use crate::types::{Identity, RunnerRequest};

/// Asynchronous client for communicating with the whoami daemon.
///
/// This client requires the `tokio` feature to be enabled.
///
/// # Example
///
/// ```no_run
/// use getmyid::AsyncClient;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), getmyid::GetMyIdError> {
/// let client = AsyncClient::new();
/// let identity = client.get_identity().await?;
/// println!("Identity: {}", identity.identity);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct AsyncClient {
    socket_path: PathBuf,
    timeout: Option<Duration>,
}

impl Default for AsyncClient {
    fn default() -> Self {
        Self::new()
    }
}

impl AsyncClient {
    /// Create a new async client with default settings.
    pub fn new() -> Self {
        Self {
            socket_path: PathBuf::from(DEFAULT_SOCKET_PATH),
            timeout: Some(DEFAULT_TIMEOUT),
        }
    }

    /// Create an async client builder for custom configuration.
    pub fn builder() -> AsyncClientBuilder {
        AsyncClientBuilder::new()
    }

    /// Get the identity of the current process asynchronously.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The socket does not exist or cannot be connected to
    /// - The daemon returns an error (e.g., no matching rule)
    /// - The response cannot be parsed
    /// - The operation times out
    pub async fn get_identity(&self) -> Result<Identity> {
        self.get_identity_with_runner(None).await
    }

    /// Get the identity with client-provided runner context.
    ///
    /// The runner request allows you to send context (like instance_id, timestamp)
    /// that will be merged with server-injected identity in the response's `runner` object.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use getmyid::{AsyncClient, RunnerRequest};
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), getmyid::GetMyIdError> {
    /// let client = AsyncClient::new();
    /// let runner_req = RunnerRequest::new()
    ///     .with_instance_id(42)
    ///     .with_current_timestamp();
    ///
    /// let identity = client.get_identity_with_runner(Some(runner_req)).await?;
    /// println!("Instance: {:?}", identity.runner.instance_id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_identity_with_runner(&self, runner: Option<RunnerRequest>) -> Result<Identity> {
        // Check socket exists
        if !self.socket_path.exists() {
            return Err(GetMyIdError::SocketNotFound(self.socket_path.clone()));
        }

        let get_identity_inner = async {
            // Connect to the socket
            let mut stream = UnixStream::connect(&self.socket_path).await.map_err(|e| {
                GetMyIdError::ConnectionFailed {
                    path: self.socket_path.clone(),
                    source: e,
                }
            })?;

            // Send runner request if provided
            if let Some(ref runner_req) = runner {
                let request = serde_json::json!({ "runner": runner_req });
                let request_str = serde_json::to_string(&request).map_err(GetMyIdError::InvalidJson)?;
                stream
                    .write_all(request_str.as_bytes())
                    .await
                    .map_err(GetMyIdError::WriteError)?;
                stream.flush().await.map_err(GetMyIdError::WriteError)?;
                // Shutdown write side to signal we're done sending
                stream.shutdown().await.ok();
            }

            // Read the response
            let mut response = String::new();
            stream
                .read_to_string(&mut response)
                .await
                .map_err(GetMyIdError::ReadError)?;

            // Parse response
            parse_response(&response)
        };

        // Apply timeout if configured
        if let Some(timeout) = self.timeout {
            tokio::time::timeout(timeout, get_identity_inner)
                .await
                .map_err(|_| GetMyIdError::Timeout(timeout))?
        } else {
            get_identity_inner.await
        }
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

/// Builder for creating a customized [`AsyncClient`].
///
/// # Example
///
/// ```no_run
/// use std::time::Duration;
/// use getmyid::AsyncClient;
///
/// let client = AsyncClient::builder()
///     .socket_path("/tmp/whoami.sock")
///     .timeout(Duration::from_secs(10))
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct AsyncClientBuilder {
    socket_path: PathBuf,
    timeout: Option<Duration>,
}

impl Default for AsyncClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl AsyncClientBuilder {
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

    /// Build the async client.
    pub fn build(self) -> AsyncClient {
        AsyncClient {
            socket_path: self.socket_path,
            timeout: self.timeout,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_async_client_builder() {
        let client = AsyncClient::builder()
            .socket_path("/tmp/test.sock")
            .timeout(Duration::from_secs(10))
            .build();

        assert_eq!(client.socket_path(), Path::new("/tmp/test.sock"));
        assert_eq!(client.timeout(), Some(Duration::from_secs(10)));
    }

    #[test]
    fn test_async_client_builder_no_timeout() {
        let client = AsyncClient::builder().timeout(None).build();

        assert_eq!(client.timeout(), None);
    }

    #[test]
    fn test_default_async_client() {
        let client = AsyncClient::new();

        assert_eq!(client.socket_path(), Path::new(DEFAULT_SOCKET_PATH));
        assert_eq!(client.timeout(), Some(DEFAULT_TIMEOUT));
    }
}
