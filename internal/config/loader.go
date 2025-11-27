package config

import (
	"bufio"
	"bytes"
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"do/internal/settings"
)

var defaultFilenames = []string{"Kylefile", "Kylefile.yaml", "Kylefile.yml", "Kylefile.toml"}

func Load(path string) (*Kylefile, error) {
	if path != "" {
		return loadFile(path)
	}
	return loadFromCurrentDir()
}

func loadFromCurrentDir() (*Kylefile, error) {
	for _, name := range defaultFilenames {
		if _, err := os.Stat(name); err == nil {
			return loadFile(name)
		}
	}
	return nil, fmt.Errorf("no Kylefile found (looked for: %v)", defaultFilenames)
}

func loadFile(path string) (*Kylefile, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return nil, err
	}

	ext := filepath.Ext(path)

	if ext == "" {
		formatName := detectFormatFromHeader(data)
		format, ok := GetFormat(formatName)
		if !ok {
			return nil, fmt.Errorf("unknown format: %s", formatName)
		}
		return format.Parse(bytes.NewReader(data))
	}

	format, ok := GetFormatByExtension(ext)
	if !ok {
		return nil, fmt.Errorf("unsupported file format: %s", ext)
	}

	return format.Parse(bytes.NewReader(data))
}

func detectFormatFromHeader(data []byte) string {
	scanner := bufio.NewScanner(bytes.NewReader(data))
	if scanner.Scan() {
		line := strings.TrimSpace(scanner.Text())
		if strings.HasPrefix(line, "#") {
			line = strings.ToLower(strings.TrimSpace(strings.TrimPrefix(line, "#")))
			if strings.HasPrefix(line, "kyle:") {
				return strings.TrimSpace(strings.TrimPrefix(line, "kyle:"))
			}
		}
	}
	return settings.Get().DefaultFormat
}
