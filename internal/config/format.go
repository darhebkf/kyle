package config

import "io"

type Format interface {
	Name() string
	Extensions() []string
	Parse(r io.Reader) (*Kylefile, error)
}

var registry = make(map[string]Format)

func Register(f Format) {
	registry[f.Name()] = f
}

func GetFormat(name string) (Format, bool) {
	f, ok := registry[name]
	return f, ok
}

func GetFormatByExtension(ext string) (Format, bool) {
	for _, f := range registry {
		for _, e := range f.Extensions() {
			if e == ext {
				return f, true
			}
		}
	}
	return nil, false
}

func ListFormats() []string {
	names := make([]string, 0, len(registry))
	for name := range registry {
		names = append(names, name)
	}
	return names
}
