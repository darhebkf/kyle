use super::Error;
use super::kylefile::Kylefile;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Yaml,
    Toml,
}

impl Format {
    pub fn name(self) -> &'static str {
        match self {
            Self::Yaml => "yaml",
            Self::Toml => "toml",
        }
    }

    pub fn extensions(self) -> &'static [&'static str] {
        match self {
            Self::Yaml => &[".yaml", ".yml"],
            Self::Toml => &[".toml"],
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        if name.eq_ignore_ascii_case("yaml") {
            Some(Self::Yaml)
        } else if name.eq_ignore_ascii_case("toml") {
            Some(Self::Toml)
        } else {
            None
        }
    }

    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext {
            ".yaml" | ".yml" => Some(Self::Yaml),
            ".toml" => Some(Self::Toml),
            _ => None,
        }
    }

    pub fn parse(self, content: &str) -> Result<Kylefile, Error> {
        match self {
            Self::Yaml => serde_yml::from_str(content).map_err(Error::Yaml),
            Self::Toml => toml::from_str(content).map_err(Error::Toml),
        }
    }
}
