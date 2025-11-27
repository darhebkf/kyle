package config

type Task struct {
	Desc string   `yaml:"desc" toml:"desc" json:"desc"`
	Run  string   `yaml:"run" toml:"run" json:"run"`
	Deps []string `yaml:"deps" toml:"deps" json:"deps"`
}

type Kylefile struct {
	Name  string          `yaml:"name" toml:"name" json:"name"`
	Tasks map[string]Task `yaml:"tasks" toml:"tasks" json:"tasks"`
}
