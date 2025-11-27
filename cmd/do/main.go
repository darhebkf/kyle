package main

import (
	"os"

	"do/internal/cli"
)

func main() {
	os.Exit(cli.Run(os.Args[1:]))
}
