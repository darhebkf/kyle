use super::format::Format;
use super::kylefile::Kylefile;
use super::{
    Error, composer_json, deno_json, justfile, makefile, package_json, pyproject, rakefile,
    standard, taskfile,
};
use crate::cli::RESERVED_COMMANDS;
use crate::output;
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
    "Taskfile.yml",
    "Taskfile.yaml",
    "Rakefile",
    "rakefile",
    "package.json",
    "composer.json",
    "deno.json",
    "deno.jsonc",
    "pyproject.toml",
    "Cargo.toml",
    "go.mod",
    "pubspec.yaml",
    "build.gradle",
    "build.gradle.kts",
    "pom.xml",
    "CMakeLists.txt",
];
const HEADER_PREFIX: &str = "kyle:";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    Kylefile,
    Makefile,
    Justfile,
    Taskfile,
    Rakefile,
    PackageJson,
    ComposerJson,
    DenoJson,
    PyProject,
    CargoToml,
    GoMod,
    Pubspec,
    CSharpProject,
    Gradle,
    Maven,
    CMake,
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

    if let Some(result) = find_by_extension(dir) {
        return result;
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

    if let Some(result) = find_by_extension(Path::new(".")) {
        return result;
    }

    let all_names: Vec<&'static str> = DEFAULT_FILENAMES
        .iter()
        .chain(FALLBACK_FILENAMES.iter())
        .copied()
        .collect();

    Err(Error::NotFound(all_names))
}

const EXTENSION_MAP: &[(&str, Source)] = &[(".csproj", Source::CSharpProject)];

fn find_by_extension(dir: &Path) -> Option<Result<(Kylefile, Source), Error>> {
    let entries = fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_str().unwrap_or("");
        for (ext, source) in EXTENSION_MAP {
            if name.ends_with(ext) {
                return Some(match *source {
                    Source::CSharpProject => Ok((standard::dotnet(), Source::CSharpProject)),
                    _ => Ok((standard::dotnet(), *source)),
                });
            }
        }
    }
    None
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

    if matches!(filename, "Taskfile.yml" | "Taskfile.yaml") {
        return Ok((taskfile::parse(&content)?, Source::Taskfile));
    }

    if matches!(filename, "Rakefile" | "rakefile") {
        return Ok((rakefile::parse(&content)?, Source::Rakefile));
    }

    if filename == "package.json" {
        return Ok((package_json::parse(&content)?, Source::PackageJson));
    }

    if filename == "composer.json" {
        return Ok((composer_json::parse(&content)?, Source::ComposerJson));
    }

    if matches!(filename, "deno.json" | "deno.jsonc") {
        return Ok((deno_json::parse(&content)?, Source::DenoJson));
    }

    if filename == "pyproject.toml" {
        return Ok((pyproject::parse(&content)?, Source::PyProject));
    }

    if filename == "Cargo.toml" {
        return Ok((standard::cargo(), Source::CargoToml));
    }

    if filename == "go.mod" {
        return Ok((standard::go_mod(), Source::GoMod));
    }

    if filename == "pubspec.yaml" {
        return Ok((standard::pubspec(), Source::Pubspec));
    }

    if filename.ends_with(".csproj") {
        return Ok((standard::dotnet(), Source::CSharpProject));
    }

    if matches!(filename, "build.gradle" | "build.gradle.kts") {
        return Ok((standard::gradle(), Source::Gradle));
    }

    if filename == "pom.xml" {
        return Ok((standard::maven(), Source::Maven));
    }

    if filename == "CMakeLists.txt" {
        return Ok((standard::cmake(), Source::CMake));
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

    let kylefile = format.parse(&content)?;
    warn_reserved_tasks(&kylefile);
    Ok((kylefile, Source::Kylefile))
}

fn warn_reserved_tasks(kylefile: &Kylefile) {
    for name in kylefile.tasks.keys() {
        if RESERVED_COMMANDS.contains(&name.as_str()) {
            output::warn(&format!(
                "task '{name}' shadows a built-in command and will be ignored — rename it or use a namespace"
            ));
        }
    }
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
