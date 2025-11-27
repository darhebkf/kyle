package cli

import (
	"bufio"
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"do/internal/settings"
)

func runInit(args []string) int {
	opts := parseInitArgs(args)

	if opts.name == "" {
		opts.name = promptName()
	}

	if opts.format == "" {
		opts.format = settings.Get().DefaultFormat
	}

	tasks := []taskDef{}
	if promptYN("Add a task?", true) {
		for {
			task := promptTask()
			tasks = append(tasks, task)
			if !promptYN("Add another task?", false) {
				break
			}
		}
	}

	filename := "Kylefile"
	content := generateKylefile(opts.format, opts.name, tasks)

	if err := os.WriteFile(filename, []byte(content), 0644); err != nil {
		fmt.Fprintf(os.Stderr, "error: %v\n", err)
		return 1
	}

	fmt.Printf("\n  Created %s\n\n", filename)
	return 0
}

type initOpts struct {
	name   string
	format string
}

type taskDef struct {
	name string
	desc string
	run  string
}

func parseInitArgs(args []string) initOpts {
	opts := initOpts{}
	for i := 0; i < len(args); i++ {
		switch args[i] {
		case "--name", "-n":
			if i+1 < len(args) {
				opts.name = args[i+1]
				i++
			}
		case "--yaml":
			opts.format = "yaml"
		case "--toml":
			opts.format = "toml"
		default:
			// Positional arg: treat as name if not a flag
			if !strings.HasPrefix(args[i], "-") && opts.name == "" {
				opts.name = args[i]
			}
		}
	}
	return opts
}

func promptName() string {
	dir, _ := os.Getwd()
	defaultName := filepath.Base(dir)

	fmt.Printf("  Project name [%s]: ", defaultName)
	reader := bufio.NewReader(os.Stdin)
	input, _ := reader.ReadString('\n')
	input = strings.TrimSpace(input)

	if input == "" {
		return defaultName
	}
	return input
}

func promptYN(question string, defaultYes bool) bool {
	hint := "Y/n"
	if !defaultYes {
		hint = "y/N"
	}

	fmt.Printf("  %s [%s]: ", question, hint)
	reader := bufio.NewReader(os.Stdin)
	input, _ := reader.ReadString('\n')
	input = strings.ToLower(strings.TrimSpace(input))

	if input == "" {
		return defaultYes
	}
	return input == "y" || input == "yes"
}

func promptTask() taskDef {
	reader := bufio.NewReader(os.Stdin)

	fmt.Print("  Task name: ")
	name, _ := reader.ReadString('\n')
	name = strings.TrimSpace(name)

	fmt.Print("  Description (optional): ")
	desc, _ := reader.ReadString('\n')
	desc = strings.TrimSpace(desc)

	fmt.Print("  Command: ")
	run, _ := reader.ReadString('\n')
	run = strings.TrimSpace(run)

	fmt.Println()

	return taskDef{name: name, desc: desc, run: run}
}

func generateKylefile(format, name string, tasks []taskDef) string {
	if format == "yaml" {
		return generateYAML(name, tasks)
	}
	return generateTOML(name, tasks)
}

func generateYAML(name string, tasks []taskDef) string {
	var sb strings.Builder
	sb.WriteString(fmt.Sprintf("name: %s\n\ntasks:\n", name))

	if len(tasks) == 0 {
		sb.WriteString("  # example:\n")
		sb.WriteString("  #   desc: An example task\n")
		sb.WriteString("  #   run: echo hello\n")
	}

	for _, t := range tasks {
		sb.WriteString(fmt.Sprintf("  %s:\n", t.name))
		if t.desc != "" {
			sb.WriteString(fmt.Sprintf("    desc: %s\n", t.desc))
		}
		sb.WriteString(fmt.Sprintf("    run: %s\n", t.run))
	}

	return sb.String()
}

func generateTOML(name string, tasks []taskDef) string {
	var sb strings.Builder
	sb.WriteString(fmt.Sprintf("name = \"%s\"\n", name))

	if len(tasks) == 0 {
		sb.WriteString("\n# [tasks.example]\n")
		sb.WriteString("# desc = \"An example task\"\n")
		sb.WriteString("# run = \"echo hello\"\n")
	}

	for _, t := range tasks {
		sb.WriteString(fmt.Sprintf("\n[tasks.%s]\n", t.name))
		if t.desc != "" {
			sb.WriteString(fmt.Sprintf("desc = \"%s\"\n", t.desc))
		}
		sb.WriteString(fmt.Sprintf("run = \"%s\"\n", t.run))
	}

	return sb.String()
}
