package config

import (
	"io"

	"github.com/BurntSushi/toml"
)

type tomlFormat struct{}

func init() {
	Register(&tomlFormat{})
}

func (t *tomlFormat) Name() string {
	return "toml"
}

func (t *tomlFormat) Extensions() []string {
	return []string{".toml"}
}

func (t *tomlFormat) Parse(r io.Reader) (*Kylefile, error) {
	var kf Kylefile
	_, err := toml.NewDecoder(r).Decode(&kf)
	if err != nil {
		return nil, err
	}
	return &kf, nil
}
