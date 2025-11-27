mod format;
mod justfile;
mod kylefile;
mod loader;
mod makefile;

pub use format::Format;
pub use kylefile::{Includes, Kylefile, Task};
pub use loader::{Source, load, load_from_dir};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[error("yaml parse error: {0}")]
    Yaml(#[from] serde_yml::Error),

    #[error("toml parse error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("unknown format: {0}")]
    UnknownFormat(String),

    #[error("unsupported file format: {0}")]
    UnsupportedExtension(String),

    #[error("no Kylefile found (looked for: {0:?})")]
    NotFound(Vec<&'static str>),
}
