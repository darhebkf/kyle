use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use thiserror::Error;

// Constants
const CONFIG_DIR: &str = "kyle";
const CONFIG_FILE: &str = "config.toml";
const DEFAULT_FORMAT: &str = "toml";
const ALLOWED_FORMATS: &[&str] = &["yaml", "toml"];
const ALLOWED_BOOLS: &[&str] = &["true", "false"];

static CONFIG_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".config")
        .join(CONFIG_DIR)
        .join(CONFIG_FILE)
});

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Settings {
    #[serde(default = "default_format")]
    pub default_format: String,
    #[serde(default)]
    pub auto_upgrade: bool,
}

fn default_format() -> String {
    DEFAULT_FORMAT.into()
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            default_format: default_format(),
            auto_upgrade: false,
        }
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("unknown config key: {0}")]
    UnknownKey(String),

    #[error("invalid value '{value}' for {key} (allowed: {allowed})")]
    InvalidValue {
        key: String,
        value: String,
        allowed: String,
    },

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("parse error: {0}")]
    Parse(#[from] toml::de::Error),

    #[error("serialize error: {0}")]
    Serialize(#[from] toml::ser::Error),
}

pub fn get() -> Settings {
    fs::read_to_string(CONFIG_PATH.as_path())
        .ok()
        .and_then(|content| toml::from_str(&content).ok())
        .unwrap_or_default()
}

pub fn set(key: &str, value: &str) -> Result<(), Error> {
    let mut settings = get();

    match key {
        "default_format" => {
            if !ALLOWED_FORMATS.contains(&value) {
                return Err(Error::InvalidValue {
                    key: key.into(),
                    value: value.into(),
                    allowed: ALLOWED_FORMATS.join(", "),
                });
            }
            settings.default_format = value.into();
        }
        "auto_upgrade" => {
            if !ALLOWED_BOOLS.contains(&value) {
                return Err(Error::InvalidValue {
                    key: key.into(),
                    value: value.into(),
                    allowed: ALLOWED_BOOLS.join(", "),
                });
            }
            settings.auto_upgrade = value == "true";
        }
        _ => return Err(Error::UnknownKey(key.into())),
    }

    save(&settings)
}

pub fn get_value(key: &str) -> Result<String, Error> {
    let settings = get();

    match key {
        "default_format" => Ok(settings.default_format),
        "auto_upgrade" => Ok(settings.auto_upgrade.to_string()),
        _ => Err(Error::UnknownKey(key.into())),
    }
}

fn save(settings: &Settings) -> Result<(), Error> {
    let path = CONFIG_PATH.as_path();

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let content = toml::to_string(settings)?;
    fs::write(path, content)?;

    Ok(())
}

pub fn path() -> &'static Path {
    CONFIG_PATH.as_path()
}

pub fn list() -> HashMap<&'static str, String> {
    let settings = get();
    HashMap::from([
        ("default_format", settings.default_format),
        ("auto_upgrade", settings.auto_upgrade.to_string()),
    ])
}
