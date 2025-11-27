package runner

import (
	"fmt"
	"os"
	"os/exec"

	"do/internal/config"
)

type Runner struct {
	kylefile *config.Kylefile
	executed map[string]bool
}

func New(kf *config.Kylefile) *Runner {
	return &Runner{
		kylefile: kf,
		executed: make(map[string]bool),
	}
}

func (r *Runner) Run(taskName string) error {
	task, ok := r.kylefile.Tasks[taskName]
	if !ok {
		return fmt.Errorf("task not found: %s", taskName)
	}

	// Run dependencies first
	for _, dep := range task.Deps {
		if r.executed[dep] {
			continue
		}
		if err := r.Run(dep); err != nil {
			return fmt.Errorf("dependency '%s' failed: %w", dep, err)
		}
	}

	if r.executed[taskName] {
		return nil
	}

	fmt.Printf("→ %s\n", taskName)

	cmd := exec.Command("sh", "-c", task.Run)
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	cmd.Stdin = os.Stdin

	if err := cmd.Run(); err != nil {
		return fmt.Errorf("task '%s' failed: %w", taskName, err)
	}

	r.executed[taskName] = true
	return nil
}

func (r *Runner) ListTasks() {
	for name, task := range r.kylefile.Tasks {
		if task.Desc != "" {
			fmt.Printf("  %s - %s\n", name, task.Desc)
		} else {
			fmt.Printf("  %s\n", name)
		}
	}
}
