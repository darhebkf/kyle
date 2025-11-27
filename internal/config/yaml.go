package config

import (
	"io"

	"gopkg.in/yaml.v3"
)

type yamlFormat struct{}

func init() {
	Register(&yamlFormat{})
}

func (y *yamlFormat) Name() string {
	return "yaml"
}

func (y *yamlFormat) Extensions() []string {
	return []string{".yaml", ".yml"}
}

func (y *yamlFormat) Parse(r io.Reader) (*Kylefile, error) {
	var kf Kylefile
	decoder := yaml.NewDecoder(r)
	if err := decoder.Decode(&kf); err != nil {
		return nil, err
	}
	return &kf, nil
}
