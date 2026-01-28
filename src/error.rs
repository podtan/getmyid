//! Error types for getmyid client library.

use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur when communicating with the whoami daemon.
#[derive(Debug, Error)]
pub enum GetMyIdError {
    /// Failed to connect to the Unix Domain Socket.
    #[error("failed to connect to socket at {path}: {source}")]
    ConnectionFailed {
        /// The socket path that failed to connect.
        path: PathBuf,
        /// The underlying I/O error.
        #[source]
        source: std::io::Error,
    },

    /// Failed to read response from the daemon.
    #[error("failed to read response: {0}")]
    ReadError(#[source] std::io::Error),

    /// Failed to write to the daemon.
    #[error("failed to write to socket: {0}")]
    WriteError(#[source] std::io::Error),

    /// Response is not valid JSON.
    #[error("invalid JSON response: {0}")]
    InvalidJson(#[source] serde_json::Error),

    /// Daemon returned an error response.
    #[error("daemon error ({code}): {message}")]
    DaemonError {
        /// Error code from the daemon (e.g., "E_NO_MATCH").
        code: String,
        /// Human-readable error message.
        message: String,
    },

    /// Response is missing required fields.
    #[error("invalid response: missing field '{field}'")]
    MissingField {
        /// The name of the missing field.
        field: &'static str,
    },

    /// Socket path does not exist.
    #[error("socket path does not exist: {0}")]
    SocketNotFound(PathBuf),

    /// Connection timeout.
    #[error("connection timeout after {0:?}")]
    Timeout(std::time::Duration),
}

/// Result type alias for getmyid operations.
pub type Result<T> = std::result::Result<T, GetMyIdError>;
