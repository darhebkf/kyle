use super::format::Format;
use super::kylefile::Kylefile;
use super::{Error, justfile, makefile};
use crate::settings;
use std::fs;
use std::path::Path;

const DEFAULT_FILENAMES: &[&str] = &["Kylefile", "Kylefile.yaml", "Kylefile.yml", "Kylefile.toml"];
const FALLBACK_FILENAMES: &[&str] = &[
    "Makefile",
    "makefile",
    "GNUmakefile",
    "justfile",
    "Justfile",
];
const HEADER_PREFIX: &str = "kyle:";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    Kylefile,
    Makefile,
    Justfile,
}

pub fn load(path: &str) -> Result<(Kylefile, Source), Error> {
    if path.is_empty() {
        load_from_current_dir()
    } else {
        load_file(Path::new(path))
    }
}

/// Load a Kylefile from a specific directory
/// This is used for namespace resolution
pub fn load_from_dir(dir: &Path) -> Result<(Kylefile, Source), Error> {
    for name in DEFAULT_FILENAMES {
        let path = dir.join(name);
        if path.exists() {
            return load_file(&path);
        }
    }

    for name in FALLBACK_FILENAMES {
        let path = dir.join(name);
        if path.exists() {
            return load_file(&path);
        }
    }

    let all_names: Vec<&'static str> = DEFAULT_FILENAMES
        .iter()
        .chain(FALLBACK_FILENAMES.iter())
        .copied()
        .collect();

    Err(Error::NotFound(all_names))
}

fn load_from_current_dir() -> Result<(Kylefile, Source), Error> {
    for name in DEFAULT_FILENAMES {
        let path = Path::new(name);
        if path.exists() {
            return load_file(path);
        }
    }

    for name in FALLBACK_FILENAMES {
        let path = Path::new(name);
        if path.exists() {
            return load_file(path);
        }
    }

    let all_names: Vec<&'static str> = DEFAULT_FILENAMES
        .iter()
        .chain(FALLBACK_FILENAMES.iter())
        .copied()
        .collect();

    Err(Error::NotFound(all_names))
}

fn load_file(path: &Path) -> Result<(Kylefile, Source), Error> {
    let content = fs::read_to_string(path)?;

    let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

    if matches!(filename, "Makefile" | "makefile" | "GNUmakefile") {
        return Ok((makefile::parse(&content)?, Source::Makefile));
    }

    if matches!(filename, "justfile" | "Justfile") {
        return Ok((justfile::parse(&content)?, Source::Justfile));
    }

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| format!(".{e}"));

    let format = match ext {
        Some(ref e) => {
            Format::from_extension(e).ok_or_else(|| Error::UnsupportedExtension(e.clone()))?
        }
        None => {
            let format_name = detect_format_from_header(&content);
            Format::from_name(&format_name).ok_or(Error::UnknownFormat(format_name))?
        }
    };

    Ok((format.parse(&content)?, Source::Kylefile))
}

fn detect_format_from_header(content: &str) -> String {
    content
        .lines()
        .next()
        .and_then(|line| {
            let line = line.trim().strip_prefix('#')?.trim();
            let format = line.strip_prefix(HEADER_PREFIX)?.trim();
            Some(format.to_ascii_lowercase())
        })
        .unwrap_or_else(|| settings::get().default_format)
}
