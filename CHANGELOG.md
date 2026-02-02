# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.2] - 2026-02-02

### Added

- New `token` field in `Identity` struct for authentication token
- Support for whoami daemon's updated 6-field protocol

## [0.1.1] - 2026-02-02

### Changed

- **BREAKING**: Renamed `kanidm_url` field to `idm_url` for vendor-agnostic naming
- Updated documentation examples to use new field names

### Added

- New `config_url` field in `Identity` struct for configuration server URL
- Support for whoami daemon's updated 5-field protocol

## [0.1.0] - 2026-01-28

### Added

- Initial release
- Synchronous `Client` for whoami daemon communication
- Asynchronous `AsyncClient` (requires `tokio` feature)
- Builder pattern for client configuration
- `Identity` struct with process identity information
- Comprehensive error handling with `GetMyIdError`
- Full documentation with examples
