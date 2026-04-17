# Kyle

> A universal task runner that speaks everyone's language.

Kyle is a command-line task runner that reads task definitions from many file formats and executes them. It works with existing Makefiles, justfiles, package.json, Cargo.toml, and 13 other file types — zero migration needed.

## Quick Start

Create a `Kylefile` in your project:

```yaml
name: my-project

tasks:
  build:
    desc: Build the project
    run: cargo build --release

  test:
    desc: Run tests
    run: cargo test
    deps: [build]
```

Run tasks:
- `kyle` — lists all tasks
- `kyle build` — runs the build task
- `kyle test` — runs test (and its dependency build)
- `kyle backend:build` — runs build in the backend/ subdirectory

## Kylefile Schema

### YAML Format

```yaml
name: string           # Project name (optional)

tasks:
  task-name:
    desc: string       # Description shown in task list (optional)
    run: string        # Shell command to execute (required)
    deps: [string]     # List of tasks to run first (optional)

includes:
  alias: path/to/dir   # Register namespace directories (optional)
```

### TOML Format

Use `Kylefile.toml` or add `# kyle: toml` header to extensionless `Kylefile`:

```toml
name = "project-name"

[tasks.build]
desc = "Build the project"
run = "make build"
deps = ["clean"]

[includes]
backend = "services/backend"
```

## Supported Files

Kyle auto-detects tasks from these files (priority order):

**Native:** Kylefile, Kylefile.toml, Kylefile.yaml, Kylefile.yml

**Parsed** (scripts extracted from file content):
- Makefile / justfile / Taskfile.yml / Rakefile
- package.json / composer.json / deno.json / pyproject.toml
- pyproject.toml `[project.scripts]` (PEP-621 console entry points)

**Dispatcher extensions** (subcommands discovered from dispatcher tasks):
- Django: tasks referencing manage.py or `[project.scripts]` entry points → management commands from `**/management/commands/*.py` exposed as `kyle <command>` or `kyle <dispatcher>:<command>`

**Standard** (common commands generated automatically):
- Cargo.toml → build, test, run, check, clippy, fmt
- go.mod → build, test, run, vet, fmt
- pubspec.yaml → run, build, test, analyze, pub-get
- *.csproj → build, test, run, publish, clean
- build.gradle → build, test, run, clean
- pom.xml → compile, test, package, install, clean
- CMakeLists.txt → configure, build, test, clean

## Namespaces

Tasks in subdirectories are accessible via namespace syntax:

```bash
kyle backend:build        # Run build in ./backend/
kyle frontend.test        # Dot separator also works
kyle services/api:dev     # Nested paths
```

Namespaces are auto-discovered from subdirectories containing any supported task file.

## Dispatcher Subcommands

Some tasks are "dispatchers" — they proxy to an underlying tool (e.g. Django's manage.py) that accepts its own subcommands. Kyle discovers these and exposes them as first-class tasks:

```bash
kyle exportxml             # Resolves to src/manage.py exportxml
kyle ccm-admin:exportxml   # Qualified form when ambiguous
kyle backend:ccm-admin:exportxml  # From parent directory
```

Resolution rules:
- Explicit local task shadows dispatcher sub with the same name
- Local level shadows discovered namespaces
- Same-exec-prefix dispatchers dedupe alphabetically
- Ambiguous matches refuse to run and list qualified forms

## Task Arg Passthrough

`kyle <task> <args...>` passes everything after the task name raw — no `--`
separator needed. Kyle only consumes argv when argv[1] is a reserved command
or starts with a leading `-`.

```bash
kyle ccm-admin --help          # runs: ccm-admin --help (Django's help)
kyle ccm-admin help subcommand # runs: ccm-admin help subcommand
kyle ccm-admin migrate         # runs: ccm-admin migrate
```

A leading `--` is stripped for back-compat with older docs but is no longer
required.

## Shadowing Collisions and --dir

When a discovered namespace alias (or dispatcher subcommand) has the same
name as a reserved command (`init`, `config`, `version`, `upgrade`, `mcp`,
`completions`, `help`), kyle prints a warning at listing time. The reserved
command still wins for the bare form. To invoke the shadowed target, use
either the qualified form (`kyle config:list`) or the `--dir` escape:

```bash
kyle --dir config:list    # Forces task-route resolution
```

## MCP Server

Kyle includes a built-in MCP server for AI tool integration:

- `kyle mcp` — start stdio MCP server
- `kyle mcp --config` — print config JSON for AI clients

Tools: `list_tasks` (discover all tasks) and `run_task` (execute a task by name).

## CLI Reference

```
kyle                              List available tasks
kyle <task> [args...]             Run a task (args passed through)
kyle init [name] [--yaml|--toml]  Create a new Kylefile
kyle upgrade                      Upgrade to latest version
kyle upgrade --status             Show recent auto-upgrade activity
kyle mcp [--config]               MCP server / print config
kyle config list|get|set|path     Manage settings
kyle completions <shell>          Shell completions (bash, zsh, fish)
kyle version / -v / --version     Print version
```

## Project Structure

```
src/
├── cli/          # CLI entry point, subcommands
├── config/       # File parsers (kylefile, makefile, justfile, package_json, etc.)
├── dispatchers/  # Dispatcher extension system (Django, etc.)
├── mcp/          # MCP server (tools.rs = list_tasks + run_task)
├── namespace/    # Namespace resolution and auto-discovery
├── runner/       # Task execution engine with dependency resolution
├── settings/     # User config (~/.config/kyle/config.toml)
└── output.rs     # Colored terminal output
tests/
├── cli.rs        # Integration tests
└── shell/        # Shell-based integration tests
docs/             # Nextra documentation site (bun, not npm)
```

## Working on Kyle

When implementing changes on this repo, work this way. It keeps the main branch
shippable at every step, makes diffs reviewable, and turns regressions into
single-commit reverts.

### 1. Explain before touching code

For anything bigger than a one-line fix, state the plan in a few sentences
before opening an editor: what's changing, what the tradeoff is, what the blast
radius is. Wait for alignment. Don't pile on speculative scope or design for
hypothetical future requirements.

### 2. Break work into dependency-ordered stages

A feature becomes a linear chain of stages where each one is independently
reviewable and earlier stages enable later ones. Typical shape:

1. Introduce scaffolding / new data types (no wiring)
2. Extend existing types (no behaviour change)
3. Implement the new logic (tested in isolation)
4. Wire it into the pipeline (first user-visible change)
5. Propagate to dependent subsystems (runner, CLI, completion)
6. User-facing polish (docs, shell scripts)

Mark exactly one stage in progress at a time. Finish it, commit it, verify it,
then start the next.

### 3. Test every stage before marking it complete

Non-negotiable gates:

- `cargo test` — all tests pass (lib + integration + doc)
- `cargo clippy --all-targets -- -D warnings` — zero warnings
- `tests/shell/*.sh` — run after `cargo build --release` when the stage
  changes user-visible CLI behaviour
- Manual smoke test against a real project when the change is a heuristic
  or filesystem scan (e.g. run the Django dispatcher against
  `/home/bfarahani/Surevoice/backend` and diff results against
  `find … management/commands`)

Each stage ships its own tests. Unit tests for pure logic, integration tests
(lib tests that load real fixtures through the loader) for end-to-end
behaviour, shell tests for the CLI surface.

### 4. Side quests are first-class

If a bug surfaces mid-stage that is adjacent to your current work (noisy
auto-upgrade output during testing, pre-existing clippy warnings in
`upgrade.rs`, etc.), stop, scope the fix as a discrete side quest, run the same
gates on it, land it as its own commit, then resume the stage. Don't silently
pile unrelated fixes into the main work, and don't silently defer them either.

### 5. One commit per stage, message describes the actual change

The commit subject should describe the schema or behaviour change that landed,
not the incidental refactor around it. "feat: add `Task.dispatcher` field +
`Dispatcher`/`Subcommand` serde types" beats "feat: dispatcher + main config
file updates". Rule of thumb: if a reviewer reads only the subject, they should
be able to predict the diff.

### 6. Style conformance

This repo follows `~/.claude/rules/rust/`. In short:

- Run `cargo fmt` before every commit
- Short, terse doc comments; no wall-of-prose, no speculative generality
- Immutable-first: `let` by default, `&str`/`&[T]` in signatures, no cloning
  to satisfy the borrow checker
- Errors: `Result<T, E>` + `?`, `anyhow::Context` at application edges,
  `thiserror` in library code, never `unwrap()` outside tests
- No speculative abstractions: three similar lines is better than a premature
  trait
- No dead comments, no `// removed X` markers, no renamed `_var` shims

### 7. End-of-stage summary

After each stage, post a short status:

- What landed (files touched, new modules, schema changes)
- What's unchanged (explicitly — "still zero user-visible change" is useful)
- Test totals (N lib + M integration passing, clippy clean, shell green)
- What the user can verify manually if they want to
- What's next

This makes multi-stage work reviewable in retrospect and keeps the user in the
loop without forcing them to read the diff.

### Common commands

```bash
cargo test                                    # full suite
cargo clippy --all-targets -- -D warnings     # clippy gate
cargo build --release                         # required before shell tests
bash tests/shell/test-namespaces.sh           # run a single shell test
rm -f ~/.local/bin/kyle && \
  cp target/release/kyle ~/.local/bin/kyle    # install locally (handles ETXTBSY)
```
