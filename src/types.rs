//! Identity types returned by the whoami daemon.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Runner information containing both client-provided context and
/// server-injected identity fields.
///
/// This object is designed to be passed directly to a config server,
/// which can use it to route and customize configuration based on
/// both the verified identity and client-provided context.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Runner {
    /// Application-level identity name (injected by whoami).
    #[serde(default)]
    pub identity: String,

    /// Hostname where the process is running (injected by whoami).
    #[serde(default)]
    pub hostname: String,

    /// Process name (injected by whoami).
    #[serde(default)]
    pub process: String,

    /// Process ID (injected by whoami).
    #[serde(default)]
    pub pid: u32,

    /// User ID (injected by whoami).
    #[serde(default)]
    pub uid: u32,

    /// Group ID (injected by whoami).
    #[serde(default)]
    pub gid: u32,

    /// Client-provided instance identifier (optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<u64>,

    /// Client-provided timestamp (optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<u64>,

    /// Additional client-provided fields.
    #[serde(flatten, default)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Client-provided runner context to send to whoami daemon.
///
/// These fields will be merged with server-injected identity fields
/// in the response's `runner` object.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RunnerRequest {
    /// Instance identifier for dynamic configuration routing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<u64>,

    /// Timestamp for request tracking.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<u64>,

    /// Additional custom fields.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl RunnerRequest {
    /// Create a new empty runner request.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the instance ID.
    pub fn with_instance_id(mut self, id: u64) -> Self {
        self.instance_id = Some(id);
        self
    }

    /// Set the timestamp.
    pub fn with_timestamp(mut self, ts: u64) -> Self {
        self.timestamp = Some(ts);
        self
    }

    /// Set the timestamp to the current Unix timestamp.
    pub fn with_current_timestamp(mut self) -> Self {
        self.timestamp = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        );
        self
    }

    /// Add a custom field.
    pub fn with_field(mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        self.extra.insert(key.into(), value.into());
        self
    }
}

/// Identity information returned by the whoami daemon.
///
/// This struct contains the application-level identity (name and URLs)
/// and a `runner` object with process/host details and client context.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Identity {
    /// Application-level identity name (from rules.conf).
    pub identity: String,

    /// Identity Management (OAuth2/OIDC) URL for this identity.
    pub idm_url: String,

    /// Configuration/API server URL for this identity.
    pub config_url: String,

    /// Authentication token for this identity.
    pub token: String,

    /// Runner information containing process details and client context.
    /// This object can be passed directly to a config server.
    pub runner: Runner,
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
        token: String,
        runner: Runner,
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
