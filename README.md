# getmyid

A Rust client library for the [whoami](https://github.com/tanbal/whoami) Identity-by-PID daemon.

[![Crates.io](https://img.shields.io/crates/v/getmyid.svg)](https://crates.io/crates/getmyid)
[![Documentation](https://docs.rs/getmyid/badge.svg)](https://docs.rs/getmyid)
[![License](https://img.shields.io/crates/l/getmyid.svg)](LICENSE-MIT)

## Overview

`getmyid` provides a type-safe, ergonomic Rust interface for querying process identity from the whoami daemon. The whoami daemon uses the Linux kernel's `SO_PEERCRED` mechanism to securely identify local processes without passwords - the kernel vouches for their identity.

## Features

- **Synchronous client**: Default, no additional dependencies
- **Asynchronous client**: Enable the `tokio` feature for async support  
- **Builder pattern**: Flexible client configuration
- **Type-safe**: Strongly typed identity and error types
- **Zero-copy parsing**: Efficient JSON deserialization

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
getmyid = "0.1"

# For async support:
getmyid = { version = "0.1", features = ["tokio"] }
```

## Quick Start

### Synchronous Usage

```rust
use getmyid::Client;

fn main() -> Result<(), getmyid::GetMyIdError> {
    let client = Client::new();
    let identity = client.get_identity()?;
    
    println!("Identity: {}", identity.identity);
    println!("Kanidm URL: {}", identity.kanidm_url);
    println!("Process: {} (PID: {})", identity.process, identity.pid);
    
    Ok(())
}
```

### Convenience Function

```rust
let identity = getmyid::get_identity()?;
println!("Identity: {}", identity.identity);
```

### Asynchronous Usage

```rust
use getmyid::AsyncClient;

#[tokio::main]
async fn main() -> Result<(), getmyid::GetMyIdError> {
    let client = AsyncClient::new();
    let identity = client.get_identity().await?;
    
    println!("Identity: {}", identity.identity);
    
    Ok(())
}
```

### Custom Configuration

```rust
use std::time::Duration;
use getmyid::Client;

let client = Client::builder()
    .socket_path("/tmp/whoami.sock")
    .timeout(Duration::from_secs(10))
    .build();
```

## How It Works

1. Your application connects to the whoami daemon's Unix Domain Socket
2. The daemon uses `SO_PEERCRED` to get your process's PID, UID, and GID from the kernel
3. The daemon reads additional info from `/proc/[PID]/` (process name, executable path)
4. The daemon matches your identity against configured rules
5. If a match is found, returns the application-level identity and Kanidm OAuth2 URL

## Identity Response

The `Identity` struct contains:

| Field | Type | Description |
|-------|------|-------------|
| `identity` | `String` | Application-level identity name |
| `kanidm_url` | `String` | Kanidm OAuth2 URL for this identity |
| `pid` | `u32` | Process ID |
| `uid` | `u32` | User ID |
| `gid` | `u32` | Group ID |
| `process` | `String` | Process name |

## Error Handling

All errors are represented by `GetMyIdError`:

- `ConnectionFailed` - Socket connection failed
- `ReadError` / `WriteError` - I/O errors
- `InvalidJson` - Response parsing failed
- `DaemonError` - Daemon returned an error (e.g., no matching rule)
- `SocketNotFound` - Socket path doesn't exist
- `Timeout` - Operation timed out

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
