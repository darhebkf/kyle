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
