# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.3] - 2026-02-13

### Added

- Shell completions (`kyle completions bash/zsh/fish`)
- SHA256 checksum verification on `kyle upgrade`
- `verify_updates` setting (default: true)
- Documentation site with full guides
- MCP server scaffold

### Fixed

- Replaced risky `unwrap()` calls with proper error handling in upgrade and runner

## [0.1.2] - 2026-02-08

### Added

- Dot (`.`) as alternative namespace separator (`kyle backend.build`)
- Local task priority over namespace resolution (tasks with `:` or `.` in names work)

### Fixed

- Bug where tasks with colons in names (e.g., `test:rust`) were incorrectly resolved as namespaces

## [0.1.1] - 2026-02-04

### Added

- `kyle upgrade` command to manually check and upgrade to the latest version
- Optional auto-upgrade feature via `kyle config set auto_upgrade true`

### Changed

- Updated install script URL to kylefile.dev

## [0.1.0] - 2026-02-03

### Added

- Initial release
- Task runner with TOML/YAML Kylefile support
- Namespace support (`kyle backend:build`)
- Auto-discovery of namespaces in subdirectories
- Cross-namespace dependencies
- Makefile and justfile compatibility (fallback)
- `kyle init` command with format selection
- `kyle config` for user settings
- Install scripts for Unix and Windows
- CI/CD with GitHub Actions

[0.1.3]: https://github.com/darhebkf/kyle/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/darhebkf/kyle/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/darhebkf/kyle/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/darhebkf/kyle/releases/tag/v0.1.0
