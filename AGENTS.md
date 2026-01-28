# AGENTS.md

This file provides guidance to AI coding assistants when working with the getmyid project.

## Project Overview

**getmyid** is a Rust client library for the [whoami](https://github.com/tanbal/whoami) Identity-by-PID daemon. It provides type-safe, ergonomic access to process identity information using Unix Domain Sockets and the kernel's `SO_PEERCRED` mechanism.

**Key Characteristics:**
- **Type**: Library crate (published on crates.io)
- **Language**: Rust 2021 edition
- **Dependencies**: Minimal (serde, serde_json, thiserror, optional tokio)
- **Platforms**: Linux only (uses Unix Domain Sockets and `/proc`)

## Build Commands

```bash
# Build (debug)
cargo build

# Build with all features
cargo build --all-features

# Run all tests
cargo test --all-features

# Build for release
cargo build --release

# Check formatting
cargo fmt --check

# Run linter
cargo clippy -- -D warnings

# Generate documentation
cargo doc --open
```

## Project Structure

```
getmyid/
├── Cargo.toml          # Package manifest with crates.io metadata
├── README.md           # Documentation
├── AGENTS.md           # This file
├── .gitignore          # Git ignore rules
├── LICENSE-MIT         # MIT license
├── LICENSE-APACHE      # Apache 2.0 license
└── src/
    ├── lib.rs          # Library entry point, re-exports
    ├── error.rs        # Error types (GetMyIdError)
    ├── types.rs        # Identity struct and daemon response types
    ├── client.rs       # Synchronous client implementation
    └── async_client.rs # Async client (requires tokio feature)
```

## Architecture

### Module Structure

- **`lib.rs`**: Library entry point, documentation, re-exports public API
- **`error.rs`**: `GetMyIdError` enum with all possible error variants
- **`types.rs`**: `Identity` struct and internal response parsing types
- **`client.rs`**: Synchronous `Client` and `ClientBuilder`
- **`async_client.rs`**: Asynchronous `AsyncClient` and `AsyncClientBuilder` (feature-gated)

### Key Design Patterns

1. **Builder Pattern**: Both `Client` and `AsyncClient` use builders for configuration
2. **Feature Gating**: Async support is behind the `tokio` feature flag
3. **Type Safety**: All errors are represented in `GetMyIdError`, no panics
4. **Zero-Trust**: The library trusts the kernel's identity assertion via `SO_PEERCRED`

### Communication Flow

```
Application -> getmyid::Client -> Unix Socket -> whoami daemon
                                                      |
                                                      v
                                              SO_PEERCRED (kernel)
                                                      |
                                                      v
                                              Rules matching
                                                      |
                                                      v
Application <- Identity struct <- JSON response <- whoami daemon
```

## Code Style

- **Rust Edition**: 2021
- **Documentation**: All public items must have rustdoc with examples
- **Error Handling**: Use `thiserror` for error types, return `Result<T, GetMyIdError>`
- **Async**: Feature-gated behind `tokio`, use tokio's async I/O primitives
- **Testing**: Unit tests in modules, doc tests for examples

## Important Notes for Assistants

### Constraints

- **Linux Only**: This library uses Linux-specific APIs (`/proc`, Unix sockets)
- **Library Crate**: No binary, only library exports
- **Minimal Dependencies**: Keep dependency count low for fast compilation
- **Backward Compatibility**: Follow semver strictly

### Adding New Features

1. Consider if it should be feature-gated
2. Add documentation with examples
3. Add unit tests
4. Update README.md if user-facing
5. Update CHANGELOG.md

### Publishing to crates.io

```bash
# Verify package contents
cargo package --list

# Dry run
cargo publish --dry-run

# Publish
cargo publish
```

## Testing Against whoami Daemon

```bash
# 1. Build and run whoami daemon (in whoami directory)
cd /path/to/whoami
make
./bin/whoamid -f -d -s /tmp/test.sock -r config/rules.conf.example

# 2. Run integration tests
cd /path/to/getmyid
WHOAMI_SOCKET=/tmp/test.sock cargo test --all-features

# 3. Manual test with sample application
cd /path/to/getmyid-sample
cargo run
```

## Commit Convention

Use Angular commit convention: `type(scope): subject`
- **Types**: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`
- **Examples**:
  - `feat(async): add timeout support for AsyncClient`
  - `fix(client): handle empty response from daemon`
  - `docs: update README with async examples`

## Related Projects

- **whoami**: The daemon this library communicates with
- **getmyid-sample**: Example application using this library
- **Kanidm**: Identity provider that whoami integrates with
