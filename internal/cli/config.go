package cli

import (
	"bufio"
	"fmt"
	"os"
	"strings"

	"do/internal/settings"
)

func runConfig(args []string) int {
	if len(args) == 0 {
		return runConfigInteractive()
	}

	switch args[0] {
	case "get":
		if len(args) < 2 {
			fmt.Fprintln(os.Stderr, "usage: kyle config get <key>")
			return 1
		}
		return configGet(args[1])
	case "set":
		if len(args) < 3 {
			fmt.Fprintln(os.Stderr, "usage: kyle config set <key> <value>")
			return 1
		}
		return configSet(args[1], args[2])
	case "list":
		return configList()
	case "path":
		fmt.Println(settings.Path())
		return 0
	default:
		fmt.Fprintf(os.Stderr, "unknown config command: %s\n", args[0])
		return 1
	}
}

func configGet(key string) int {
	val, err := settings.GetValue(key)
	if err != nil {
		fmt.Fprintf(os.Stderr, "error: %v\n", err)
		return 1
	}
	fmt.Println(val)
	return 0
}

func configSet(key, value string) int {
	if err := settings.Set(key, value); err != nil {
		fmt.Fprintf(os.Stderr, "error: %v\n", err)
		return 1
	}
	fmt.Printf("  %s = %s\n", key, value)
	return 0
}

func configList() int {
	for k, v := range settings.List() {
		fmt.Printf("  %s = %s\n", k, v)
	}
	return 0
}

func runConfigInteractive() int {
	fmt.Println("\n  Kyle Configuration")
	fmt.Println("  ──────────────────")
	fmt.Printf("  Config file: %s\n\n", settings.Path())

	s := settings.Get()
	reader := bufio.NewReader(os.Stdin)

	// Default format
	fmt.Printf("  Default format [%s]: ", s.DefaultFormat)
	input, _ := reader.ReadString('\n')
	input = strings.TrimSpace(input)
	if input != "" && input != s.DefaultFormat {
		if err := settings.Set("default_format", input); err != nil {
			fmt.Fprintf(os.Stderr, "  error: %v\n", err)
		} else {
			fmt.Println("  ✓ Updated")
		}
	}

	fmt.Println()
	return 0
}
