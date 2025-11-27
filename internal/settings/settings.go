package settings

import (
	"os"
	"path/filepath"

	"github.com/BurntSushi/toml"
)

type Settings struct {
	DefaultFormat string `toml:"default_format"`
}

var defaultSettings = Settings{
	DefaultFormat: "toml",
}

var configPath string

func init() {
	home, _ := os.UserHomeDir()
	configPath = filepath.Join(home, ".config", "kyle", "config.toml")
}

func Get() Settings {
	s := defaultSettings

	f, err := os.Open(configPath)
	if err != nil {
		return s
	}
	defer f.Close()

	toml.NewDecoder(f).Decode(&s)
	return s
}

func Set(key, value string) error {
	s := Get()

	switch key {
	case "default_format":
		if value != "yaml" && value != "toml" {
			return &InvalidValueError{key, value, "yaml, toml"}
		}
		s.DefaultFormat = value
	default:
		return &UnknownKeyError{key}
	}

	return save(s)
}

func GetValue(key string) (string, error) {
	s := Get()
	switch key {
	case "default_format":
		return s.DefaultFormat, nil
	default:
		return "", &UnknownKeyError{key}
	}
}

func save(s Settings) error {
	dir := filepath.Dir(configPath)
	if err := os.MkdirAll(dir, 0755); err != nil {
		return err
	}

	f, err := os.Create(configPath)
	if err != nil {
		return err
	}
	defer f.Close()

	return toml.NewEncoder(f).Encode(s)
}

func Path() string {
	return configPath
}

func List() map[string]string {
	s := Get()
	return map[string]string{
		"default_format": s.DefaultFormat,
	}
}

type UnknownKeyError struct {
	Key string
}

func (e *UnknownKeyError) Error() string {
	return "unknown config key: " + e.Key
}

type InvalidValueError struct {
	Key     string
	Value   string
	Allowed string
}

func (e *InvalidValueError) Error() string {
	return "invalid value '" + e.Value + "' for " + e.Key + " (allowed: " + e.Allowed + ")"
}
