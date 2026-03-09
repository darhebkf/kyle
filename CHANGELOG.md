# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.9] - 2026-03-09

### Fixed

- Tasks from package.json, composer.json, and pyproject.toml now resolve local binaries (`node_modules/.bin`, `vendor/bin`, `.venv/bin` prepended to PATH)

## [0.1.8] - 2026-03-05 — Bugfixes

### Fixed

- Removed spurious "no Kylefile found" warning when running tasks from package.json, Makefile, Cargo.toml, and other supported file types
- Fixed `kyle upgrade` failing with "Text file busy" on Linux (ETXTBSY) — binary now unlinks before replacing
- Fixed install script exiting immediately when piped via `curl | sh` — prompts now read from `/dev/tty`
- MCP `list_tasks` now shows human-readable source names (`package.json` instead of `PackageJson`)

### Changed

- Install scripts refactored with shared `ask()` and `write_mcp_json()` helpers
- Added `Display` impl for `Source` and `FileType` enums

### Note

- If `kyle upgrade` fails from v0.1.7 or earlier (due to the ETXTBSY bug), re-run the install script: `curl -fsSL https://kylefile.dev/install.sh | sh`

## [0.1.7] - 2026-03-01 — MCP Client Support

### Added

- MCP setup for Codex (OpenAI), Antigravity (Google), and GitHub Copilot in install scripts
- Install script "Other / manual" option with config instructions for all clients

### Changed

- MCP docs page expanded to 7 client tabs (Claude Code, Claude Desktop, Cursor, Windsurf, Codex, Antigravity, GitHub Copilot)
- Updated llms.txt with all MCP client config formats

## [0.1.6] - 2026-03-01 — MCP Server

### Added

- MCP server with `list_tasks` and `run_task` tools for AI client integration
- `kyle mcp --config` command to print MCP config JSON for AI clients
- MCP setup prompt in install scripts (Claude Code, Cursor, Windsurf)
- MCP documentation page

### Changed

- Install scripts (sh + ps1) at repo root are now symlinks to `docs/public/`
- Updated docs: kylefiles page with all 16 supported file types, namespaces page, CLI reference
- Updated llms.txt and AGENTS.md with full file support and MCP tools

## [0.1.5] - 2026-03-01 — Universal File Support

### Added

- Cycle detection in task dependency graphs with clear error messages (`a → b → a`)
- package.json scripts support
- composer.json scripts support
- deno.json / deno.jsonc tasks support
- Taskfile.yml (go-task) support
- Rakefile support
- pyproject.toml support (PDM, Hatch, Rye scripts; fallback to standard Python tasks)
- Standard command generation for Cargo.toml, go.mod, pubspec.yaml, *.csproj, build.gradle, pom.xml, CMakeLists.txt
- Namespace discovery for all new file types

### Fixed

- Local dependency executed_key bug in task runner

## [0.1.4] - 2026-03-01

### Added

- Reserved keyword warnings — tasks that shadow built-in commands show a warning
- Dynamic shell completions — tab completion now suggests task names from your Kylefile
- Interactive install script — prompts for auto-upgrade and shell completions setup

### Changed

- Replaced `clap_complete` with custom completion scripts (bash/zsh/fish)
- Split test suite into modular per-feature test files
- `--summary` hidden flag for machine-readable task listing

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

[0.1.9]: https://github.com/darhebkf/kyle/compare/v0.1.8...v0.1.9
[0.1.8]: https://github.com/darhebkf/kyle/compare/v0.1.7...v0.1.8
[0.1.7]: https://github.com/darhebkf/kyle/compare/v0.1.6...v0.1.7
[0.1.6]: https://github.com/darhebkf/kyle/compare/v0.1.5...v0.1.6
[0.1.5]: https://github.com/darhebkf/kyle/compare/v0.1.4...v0.1.5
[0.1.4]: https://github.com/darhebkf/kyle/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/darhebkf/kyle/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/darhebkf/kyle/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/darhebkf/kyle/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/darhebkf/kyle/releases/tag/v0.1.0
