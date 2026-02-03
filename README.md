# Kyle

A simple, polyglot task runner. Define your project tasks in a `Kylefile` and run them with `kyle <task>`.

## Installation

```bash
curl -fsSL https://kylefile.dev/install.sh | sh
```

Windows:
```powershell
irm https://kylefile.dev/install.ps1 | iex
```

Or build from source:
```bash
cargo build --release
mv target/release/kyle ~/.local/bin/
```

## Usage

```bash
kyle          # List available tasks
kyle build    # Run the build task
kyle test     # Run test task
```

### Zero Config

Kyle auto-detects and runs existing project files:

```bash
# Have a Makefile? Just works.
kyle build    # runs make build

# Have a Justfile? Just works.
kyle test     # runs just test
```

### Kylefile

Create a `Kylefile` for custom tasks:

```toml
# kyle: toml
name = "my-project"

[tasks.build]
desc = "Build the project"
run = "cargo build --release"

[tasks.test]
desc = "Run tests"
run = "cargo test"
deps = ["build"]
```

Or YAML:
```yaml
# kyle: yaml
name: my-project

tasks:
  build:
    desc: Build the project
    run: cargo build --release
```

## Namespaces

Run tasks in subdirectories:

```bash
kyle backend:build      # runs build task in ./backend/
kyle frontend:test      # runs test task in ./frontend/
kyle apps/mobile:dev    # nested namespaces work too
```

Cross-namespace dependencies:
```toml
[tasks.deploy]
deps = ["backend:build", "frontend:build"]
run = "echo deploying"
```

## File Priority

Kyle looks for files in this order:

1. `Kylefile`, `Kylefile.yaml`, `Kylefile.yml`, `Kylefile.toml`
2. `Makefile`, `makefile`, `GNUmakefile`
3. `justfile`, `Justfile`

## Configuration

```bash
kyle config list              # Show all settings
kyle config get default_format
kyle config set default_format yaml
```

## Features

- Multi-format: YAML, TOML, and more coming soon
- Zero migration: works with existing Makefiles and Justfiles with other manifests coming soon
- Namespaces: `kyle backend:build` for monorepos
- Task dependencies
- Argument passthrough: `kyle build --release`
- Cross-platform: Linux, macOS, Windows

## License

AGPL-3.0 - See [LICENSE](LICENSE) for details.
