# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1] - 2025-02-04

### Added

- `kyle upgrade` command to manually check and upgrade to the latest version
- Optional auto-upgrade feature via `kyle config set auto_upgrade true`

### Changed

- Updated install script URL to kylefile.dev

## [0.1.0] - 2025-02-03

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

[0.1.1]: https://github.com/darhebkf/kyle/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/darhebkf/kyle/releases/tag/v0.1.0
