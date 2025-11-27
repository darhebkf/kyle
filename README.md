# Kyle

A simple, extensible task runner. Define your project tasks in a `Kylefile` and run them with `kyle <task>`.

## Installation

```bash
curl -sSL https://kylefile.dev/install.sh | sh
```

Or build from source:

```bash
go build -o kyle ./cmd/do
mv kyle ~/.local/bin/
```

## Usage

Create a `Kylefile` in your project root:

```yaml
name: my-project

tasks:
  build:
    desc: Build the project
    run: make build

  test:
    desc: Run tests
    run: make test
    deps: [build]

  clean:
    desc: Clean build artifacts
    run: rm -rf build/
```

Then run:

```bash
kyle          # List available tasks
kyle build    # Run the build task
kyle test     # Run test (runs build first due to deps)
```

**Note:** Task names `init`, `config`, `help`, `version` are reserved for built-in commands.

## Kylefile Formats

Kyle supports multiple config formats:

- `Kylefile` (YAML by default, or specify format with header)
- `Kylefile.yaml` / `Kylefile.yml`
- `Kylefile.toml`

For extensionless `Kylefile`, you can specify the format with a header comment:

```yaml
# kyle: yaml
name: my-project
tasks:
  build:
    run: make
```

```toml
# kyle: toml
name = "my-project"

[tasks.build]
run = "make"
```

## Features

- Simple YAML/TOML configuration
- Task dependencies
- Multi-format support
- Format auto-detection via header (`# kyle: toml`)
- Extensible architecture

## Roadmap

Planned features for future releases:

- [ ] JSON format support
- [ ] XML format support
- [ ] Auto-fix suggestions for failed commands
- [ ] Global configuration (`~/.config/kyle/`)
- [ ] Virtual environment support (venv, nvm, etc.)
- [ ] Plugin/extension system
- [ ] Interactive TUI mode
- [ ] LLM integration for task suggestions
- [ ] MCP server for IDE integration
- [ ] Makefile format support

## License

AGPL-3.0 - See [LICENSE](LICENSE) for details.
