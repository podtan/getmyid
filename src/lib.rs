//! # getmyid
//!
//! A Rust client library for the [whoami](https://github.com/tanbal/whoami) Identity-by-PID daemon.
//!
//! This library provides both synchronous and asynchronous clients for querying process identity
//! from the whoami daemon, which uses the Linux kernel's `SO_PEERCRED` mechanism to securely
//! identify local processes.
//!
//! ## Features
//!
//! - **Synchronous client**: Default, no additional dependencies
//! - **Asynchronous client**: Enable the `tokio` feature for async support
//! - **Builder pattern**: Flexible client configuration
//! - **Type-safe**: Strongly typed identity and error types
//!
//! ## Quick Start
//!
//! ### Synchronous Usage
//!
//! ```no_run
//! use getmyid::Client;
//!
//! fn main() -> Result<(), getmyid::GetMyIdError> {
//!     let client = Client::new();
//!     let identity = client.get_identity()?;
//!     
//!     println!("Identity: {}", identity.identity);
//!     println!("IDM URL: {}", identity.idm_url);
//!     println!("Config URL: {}", identity.config_url);
//!     println!("Process: {} (PID: {})", identity.process, identity.pid);
//!     
//!     Ok(())
//! }
//! ```
//!
//! ### Asynchronous Usage (requires `tokio` feature)
//!
//! ```no_run
//! use getmyid::AsyncClient;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), getmyid::GetMyIdError> {
//!     let client = AsyncClient::new();
//!     let identity = client.get_identity().await?;
//!     
//!     println!("Identity: {}", identity.identity);
//!     
//!     Ok(())
//! }
//! ```
//!
//! ### Custom Socket Path
//!
//! ```no_run
//! use std::time::Duration;
//! use getmyid::Client;
//!
//! let client = Client::builder()
//!     .socket_path("/tmp/whoami.sock")
//!     .timeout(Duration::from_secs(10))
//!     .build();
//! ```
//!
//! ## How It Works
//!
//! 1. Your application connects to the whoami daemon's Unix Domain Socket
//! 2. The daemon uses `SO_PEERCRED` to get your process's PID, UID, and GID from the kernel
//! 3. The daemon reads additional info from `/proc/[PID]/` (process name, executable path)
//! 4. The daemon matches your identity against configured rules
//! 5. If a match is found, returns the application-level identity and URLs
//!
//! This provides zero-trust authentication where applications don't need passwords -
//! the Linux kernel vouches for their identity.

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

mod client;
mod error;
mod types;

#[cfg(feature = "tokio")]
mod async_client;

// Re-export main types
pub use client::{Client, ClientBuilder, DEFAULT_SOCKET_PATH, DEFAULT_TIMEOUT};
pub use error::{GetMyIdError, Result};
pub use types::Identity;

#[cfg(feature = "tokio")]
pub use async_client::{AsyncClient, AsyncClientBuilder};

/// Convenience function to get identity using default settings.
///
/// This is equivalent to `Client::new().get_identity()`.
///
/// # Example
///
/// ```no_run
/// let identity = getmyid::get_identity()?;
/// println!("Identity: {}", identity.identity);
/// # Ok::<(), getmyid::GetMyIdError>(())
/// ```
pub fn get_identity() -> Result<Identity> {
    Client::new().get_identity()
}

/// Convenience function to get identity using a custom socket path.
///
/// # Example
///
/// ```no_run
/// let identity = getmyid::get_identity_from("/tmp/whoami.sock")?;
/// println!("Identity: {}", identity.identity);
/// # Ok::<(), getmyid::GetMyIdError>(())
/// ```
pub fn get_identity_from<P: AsRef<std::path::Path>>(socket_path: P) -> Result<Identity> {
    Client::builder()
        .socket_path(socket_path)
        .build()
        .get_identity()
}
