# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.3] - 2026-05-26

### Added

- **OpenCode MCP installer support** — `install.sh` and `install.ps1` now include OpenCode (Anomaly) as a first-party MCP setup target using OpenCode's native `~/.config/opencode/opencode.json` `mcp` schema.

## [0.2.2] - 2026-04-30

### Added

- **Autocorrect for mistyped task names** — Damerau-Levenshtein matching against the local task list, dispatcher subcommands, and discovered namespaces. New `autocorrect` setting with three modes: `suggest` (default — print `Did you mean 'build'?` and exit 1), `off` (plain `task not found`), and `autocorrect` (print stderr notice and run the corrected task). Args are preserved across the re-run (`kyle buld --force` → `build --force`)
- **Autocorrect safety guards** — reserved kyle commands (`upgrade`, `config`, ...) are never auto-targeted; ambiguous matches at the best distance fall back to suggest mode regardless of setting; distance ≤ 2 with a 50% match-ratio guard so short or distant inputs never match; namespaced inputs (`backed:tst`) match per-segment so segment counts must align
- **Fuzzy tab completion** — when prefix completion in bash/zsh/fish returns nothing for a non-empty input, the completion script falls back to fuzzy matching via a new internal `--complete-fuzzy <PARTIAL>` flag. `kyle buld<TAB>` now offers `build`. Always-on regardless of the `autocorrect` setting
- **Install-time prompt** — both `install.sh` and `install.ps1` ask `Autocorrect typos? [suggest/off/autocorrect]` after the auto-update prompt and persist the choice via `kyle config set autocorrect ...`

## [0.2.1] - 2026-04-17

### Added

- **Pre-clap task bypass** — `kyle <task> <args...>` passes everything past the task name raw to the task. `--help`, `--version`, `help`, etc. now work transparently as task arguments without needing `--` as a separator
- **`--dir <task>` escape** — forces task-route resolution when a namespace alias or task name shadows a reserved command (e.g. `kyle --dir config:list`)
- **Shadow warnings** — at `kyle` listing time, warn when a discovered namespace alias or dispatcher subcommand shadows a reserved command, with example invocation syntax
- **Context-aware completion** — `kyle --completion-for <task>` emits dispatcher subcommands or namespace tasks for the given first-word. Bash/zsh/fish scripts now complete `kyle <task> <TAB>` with the task's valid next arguments
- **Grouped `kyle` listing** — dispatcher subcommands display grouped by Django app (`[surevoice]`, `[dfs]`, ...), matching the layout of `ccm-admin help`. Equivalent dispatchers (same exec_prefix) dedupe to the alphabetically-first task name
- **Django app-name metadata** — `Subcommand.group` field populated during scan from the `<app>/management/commands/` path segment

### Fixed

- **`kyle ccm-admin` (bare) failing with `sh: ccm.__main__:main: not found`** — `[project.scripts]` entries now store the script name as `run` (the actual invocable command after pdm/uv install), with the entry-point reference kept in a new internal `entry_point` field for dispatcher detection
- **PDM `{cmd = [...]}` array format** — table-style scripts with cmd-as-array (e.g. `format = {cmd = ["bash", "-c", "isort src ; black src"]}`) were silently dropped; now parsed as space-joined
- **Release workflow SHA256SUMS collision** — per-platform checksum files had identical names and overwrote each other when the release job merged artifacts, leaving `SHA256SUMS` with only one platform's checksum. Now per-platform files are named `checksums-<target>.txt` and the combine step globs them all

### Changed

- **Dispatcher detection split paths** — Django extension now detects via direct `manage.py` reference, entry-point metadata (new), or entry-point-shaped command (legacy)
- **Dispatcher sub exec uses shebang** — the Django extension emits the plain `manage.py` path as `exec_prefix` (no interpreter hardcode); relies on the file's shebang + kyle's `.venv/bin` PATH prepending
- **MCP `list_tasks`** now renders dispatcher subcommands grouped by app, matching the CLI listing

## [0.2.0] - 2026-04-16 — Dispatcher Extensions & Shell Completion Overhaul

### Added

- **Dispatcher extension system** — pluggable trait for expanding "dispatcher" tasks (e.g. Django manage.py, Rails, artisan) into discoverable subcommands that kyle exposes as first-class tasks
- **Django extension** — auto-detects manage.py and `[project.scripts]` entry points, enumerates management commands from `**/management/commands/*.py`, registers them as subcommands (`kyle exportxml` → `src/manage.py exportxml`)
- **`[project.scripts]` parsing** — PEP-621 console script entry points (e.g. `ccm-admin = "ccm.__main__:main"`) are now recognized as kyle tasks alongside pdm/hatch/rye shortcuts
- **Full resolution cascade** — `kyle <name>` searches local tasks → local dispatcher subcommands → discovered namespace tasks → namespace dispatcher subcommands, with conflict detection and qualified disambiguation syntax (`kyle ccm-admin:exportxml`, `kyle backend:ccm-admin:exportxml`)
- **`--completion-feed`** hidden flag emitting the full sorted candidate set: reserved commands, local tasks, dispatcher subs (bare + qualified), namespace prefixes, namespaced tasks, and namespace dispatcher subs
- **Rewritten shell completions** (bash/zsh/fish) consuming the new feed — bash handles `:` word-break correctly, all shells now complete dispatcher subcommands and namespace-qualified tasks
- **`kyle upgrade --status`** — shows recent auto-upgrade activity from the log
- **Bun workspace support verified** — shell test confirms bun projects work end-to-end via package.json parser

### Fixed

- **Auto-upgrade noise** — throttled to once per 24h via stamp file, detached via background re-exec, failures logged silently instead of printing to stderr on every invocation
- **PDM `{cmd = [...]}` array format** — table-style scripts with cmd-as-array (e.g. `format = {cmd = ["bash", "-c", "isort src ; black src"]}`) were silently dropped; now parsed correctly
- **Clippy warnings** — fixed pre-existing `uninlined_format_args` in upgrade.rs

### Changed

- **Precedence rules**: explicit local task shadows dispatcher sub with same name; local level shadows discovered namespaces; same-exec-prefix dispatchers dedupe alphabetically
- **Ambiguity errors** now show all qualified forms the user can type instead of just listing namespaces
- **MCP `list_tasks`** now enumerates dispatcher subcommands alongside regular tasks
- **`--summary`** preserved for back-compat (task names only, excluding reserved commands)

## [0.1.10] - 2026-04-02

### Added

- `kyle <task>` from a parent directory now auto-discovers tasks in child namespaces
- Conflict detection: when multiple child namespaces define the same task, kyle shows all matches with namespace syntax hints
- Task-not-found errors now list discovered namespaces with usage hints

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

[0.2.2]: https://github.com/darhebkf/kyle/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/darhebkf/kyle/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/darhebkf/kyle/compare/v0.1.10...v0.2.0
[0.1.10]: https://github.com/darhebkf/kyle/compare/v0.1.9...v0.1.10
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
