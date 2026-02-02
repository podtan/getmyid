//! Identity types returned by the whoami daemon.

use serde::{Deserialize, Serialize};

/// Identity information returned by the whoami daemon.
///
/// This struct contains both the application-level identity (name and URLs)
/// and the process-level identity (PID, UID, GID, process name).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Identity {
    /// Application-level identity name (from rules.conf).
    pub identity: String,

    /// Identity Management (OAuth2/OIDC) URL for this identity.
    pub idm_url: String,

    /// Configuration/API server URL for this identity.
    pub config_url: String,

    /// Process ID of the calling process.
    pub pid: u32,

    /// User ID of the calling process.
    pub uid: u32,

    /// Group ID of the calling process.
    pub gid: u32,

    /// Name of the calling process (from /proc/[pid]/comm).
    pub process: String,
}

/// Raw response from the whoami daemon.
#[derive(Debug, Deserialize)]
pub(crate) struct DaemonResponse {
    pub status: String,
    #[serde(flatten)]
    pub data: ResponseData,
}

/// Response data variants.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum ResponseData {
    Success {
        identity: String,
        idm_url: String,
        config_url: String,
        pid: u32,
        uid: u32,
        gid: u32,
        process: String,
    },
    Error {
        error_code: String,
        message: String,
    },
}

impl DaemonResponse {
    /// Check if the response indicates success.
    pub fn is_ok(&self) -> bool {
        self.status == "ok"
    }
}
