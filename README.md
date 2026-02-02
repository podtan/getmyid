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
- **Runner context**: Send client context for dynamic configuration routing
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
    
    println!("Identity:   {}", identity.identity);
    println!("IDM URL:    {}", identity.idm_url);
    println!("Config URL: {}", identity.config_url);
    println!("Token:      {}", identity.token);
    println!("Hostname:   {}", identity.runner.hostname);
    println!("Process:    {} (PID: {})", identity.runner.process, identity.runner.pid);
    
    Ok(())
}
```

### With Runner Context (Dynamic Configuration)

For ephemeral applications that need dynamic configuration routing:

```rust
use getmyid::{Client, RunnerRequest};

fn main() -> Result<(), getmyid::GetMyIdError> {
    let client = Client::new();
    
    // Send context that will be merged with identity in runner object
    let runner_req = RunnerRequest::new()
        .with_instance_id(42)
        .with_current_timestamp();
    
    let identity = client.get_identity_with_runner(Some(runner_req))?;
    
    // The runner object can be passed directly to a config server
    println!("Runner: {:?}", identity.runner);
    
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
2. Optionally sends a runner request with client context (instance_id, timestamp, etc.)
3. The daemon uses `SO_PEERCRED` to get your process's PID, UID, and GID from the kernel
4. The daemon reads additional info from `/proc/[PID]/` (process name, executable path)
5. The daemon matches your identity against configured rules
6. Returns identity with a `runner` object containing merged client + server fields

## Identity Response

The `Identity` struct contains:

| Field | Type | Description |
|-------|------|-------------|
| `identity` | `String` | Application-level identity name |
| `idm_url` | `String` | Identity Management (Kanidm) OAuth2/OIDC URL |
| `config_url` | `String` | Application configuration endpoint URL |
| `token` | `String` | Pre-shared authentication token |
| `runner` | `Runner` | Combined client context + server identity |

### Runner Object

The `runner` object is designed to be passed directly to a config server:

| Field | Source | Description |
|-------|--------|-------------|
| `identity` | server | Application-level identity name |
| `hostname` | server | Machine hostname |
| `process` | server | Process name |
| `pid` | server | Process ID (kernel-verified) |
| `uid` | server | User ID (kernel-verified) |
| `gid` | server | Group ID (kernel-verified) |
| `instance_id` | client | Client-provided instance identifier (optional) |
| `timestamp` | client | Client-provided timestamp (optional) |
| `extra` | client | Additional custom fields |

### Example Output

```
Identity retrieved successfully!

  Identity:   TRUSTEE_AGENT
  IDM URL:    https://auth.example.com/oauth2/trustee
  Config URL: https://config.example.com/api/trustee
  Token:      tok_trustee_xxx

  Runner:
    Hostname:    worker-node-03
    Process:     trustee
    PID:         26567
    UID:         1000
    GID:         1000
    Instance ID: 42
    Timestamp:   1738512000
```

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
