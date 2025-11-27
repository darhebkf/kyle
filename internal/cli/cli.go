package cli

import (
	"fmt"
	"os"

	"do/internal/config"
	"do/internal/runner"
)

func Run(args []string) int {
	if len(args) == 0 {
		return runTasks(args)
	}

	switch args[0] {
	case "help", "--help", "-h":
		printHelp()
		return 0
	case "version", "--version", "-v":
		fmt.Println("kyle v0.1.0")
		return 0
	case "init":
		return runInit(args[1:])
	case "config":
		return runConfig(args[1:])
	default:
		return runTasks(args)
	}
}

func runTasks(args []string) int {
	kf, err := config.Load("")
	if err != nil {
		fmt.Fprintf(os.Stderr, "\n  No Kylefile found in current directory.\n\n")
		fmt.Fprintf(os.Stderr, "  Run 'kyle init' to create one.\n\n")
		return 1
	}

	r := runner.New(kf)

	if len(args) == 0 {
		fmt.Println("Available tasks:")
		r.ListTasks()
		return 0
	}

	if err := r.Run(args[0]); err != nil {
		fmt.Fprintf(os.Stderr, "error: %v\n", err)
		return 1
	}

	return 0
}

func printHelp() {
	fmt.Println(`kyle - task runner

Usage:
  kyle              List available tasks
  kyle <task>       Run a task
  kyle init         Create a new Kylefile
  kyle config       Configure kyle settings
  kyle help         Show this help

Init:
  kyle init                  Interactive setup
  kyle init <name>           Quick setup with name
  kyle init <name> --yaml    Quick setup with YAML format
  kyle init <name> --toml    Quick setup with TOML format (default)

Config:
  kyle config                Interactive settings
  kyle config list           Show all settings
  kyle config get <key>      Get a config value
  kyle config set <key> <v>  Set a config value

Reserved task names: init, config, help, version`)
}
