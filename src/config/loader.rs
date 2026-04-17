use super::format::Format;
use super::kylefile::Kylefile;
use super::{
    Error, composer_json, deno_json, justfile, makefile, package_json, pyproject, rakefile,
    standard, taskfile,
};
use crate::cli::RESERVED_COMMANDS;
use crate::dispatchers::{DispatcherContext, DispatcherRegistry, SourceHint};
use crate::output;
use crate::settings;
use std::fs;
use std::path::Path;
use std::sync::LazyLock;

static REGISTRY: LazyLock<DispatcherRegistry> = LazyLock::new(DispatcherRegistry::builtin);

fn expand_dispatchers(kf: &mut Kylefile, dir: &Path, source: Source) {
    let hint = source.hint();
    for (name, task) in kf.tasks.iter_mut() {
        if task.dispatcher.is_some() {
            continue;
        }
        let ctx = DispatcherContext {
            dir,
            task_name: name,
            command: &task.run,
            entry_point: task.entry_point.as_deref(),
            source_hint: hint,
        };
        if let Some(dispatcher) = REGISTRY.try_expand(&ctx) {
            task.dispatcher = Some(dispatcher);
        }
    }
}

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

impl Source {
    fn hint(self) -> SourceHint {
        match self {
            Self::PyProject => SourceHint::PyProject,
            Self::PackageJson => SourceHint::PackageJson,
            Self::ComposerJson => SourceHint::ComposerJson,
            Self::DenoJson => SourceHint::DenoJson,
            Self::Kylefile => SourceHint::Kylefile,
            _ => SourceHint::Other,
        }
    }
}

impl std::fmt::Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Kylefile => write!(f, "Kylefile"),
            Self::Makefile => write!(f, "Makefile"),
            Self::Justfile => write!(f, "justfile"),
            Self::Taskfile => write!(f, "Taskfile.yml"),
            Self::Rakefile => write!(f, "Rakefile"),
            Self::PackageJson => write!(f, "package.json"),
            Self::ComposerJson => write!(f, "composer.json"),
            Self::DenoJson => write!(f, "deno.json"),
            Self::PyProject => write!(f, "pyproject.toml"),
            Self::CargoToml => write!(f, "Cargo.toml"),
            Self::GoMod => write!(f, "go.mod"),
            Self::Pubspec => write!(f, "pubspec.yaml"),
            Self::CSharpProject => write!(f, ".csproj"),
            Self::Gradle => write!(f, "build.gradle"),
            Self::Maven => write!(f, "pom.xml"),
            Self::CMake => write!(f, "CMakeLists.txt"),
        }
    }
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
            let result = load_file(&path)?;
            if !result.0.tasks.is_empty() {
                return Ok(result);
            }
            // Empty Kylefile — try fallback files in same dir
            for fb_name in FALLBACK_FILENAMES {
                let fb_path = dir.join(fb_name);
                if fb_path.exists()
                    && let Ok(fb) = load_file(&fb_path)
                {
                    return Ok(fb);
                }
            }
            return Ok(result);
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
            let result = load_file(path)?;
            if !result.0.tasks.is_empty() {
                return Ok(result);
            }
            // Kylefile exists but has no tasks — try fallback files
            if let Some(fallback) = load_first_fallback() {
                output::warn(&format!(
                    "Kylefile has no tasks, using {}. Run 'kyle init --detect' to auto-populate",
                    fallback.1
                ));
                return Ok(fallback);
            }
            return Ok(result);
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

fn load_first_fallback() -> Option<(Kylefile, Source)> {
    for name in FALLBACK_FILENAMES {
        let path = Path::new(name);
        if path.exists() {
            return load_file(path).ok();
        }
    }
    if let Some(Ok(result)) = find_by_extension(Path::new(".")) {
        return Some(result);
    }
    None
}

/// Load tasks from project files only (skip Kylefiles).
/// Used by `kyle init --detect` to find tasks to populate a new Kylefile.
pub fn detect_project_tasks(dir: &Path) -> Option<(Kylefile, Source)> {
    for name in FALLBACK_FILENAMES {
        let path = dir.join(name);
        if path.exists()
            && let Ok(result) = load_file(&path)
            && !result.0.tasks.is_empty()
        {
            return Some(result);
        }
    }
    if let Some(Ok(result)) = find_by_extension(dir)
        && !result.0.tasks.is_empty()
    {
        return Some(result);
    }
    None
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
    let (mut kf, source) = load_file_inner(path)?;
    let dir = path.parent().unwrap_or(Path::new("."));
    expand_dispatchers(&mut kf, dir, source);
    Ok((kf, source))
}

fn load_file_inner(path: &Path) -> Result<(Kylefile, Source), Error> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn write(path: &Path, content: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, content).unwrap();
    }

    #[test]
    fn pyproject_task_gains_django_dispatcher() {
        let tmp = TempDir::new().unwrap();
        write(
            &tmp.path().join("pyproject.toml"),
            r#"[project]
name = "demo"

[tool.pdm.scripts]
dev = "src/manage.py runserver"
test = "pytest"
"#,
        );
        write(&tmp.path().join("src/manage.py"), "#!/usr/bin/env python\n");
        write(
            &tmp.path().join("src/app/management/commands/exportxml.py"),
            "",
        );
        write(
            &tmp.path().join("src/app/management/commands/migrate.py"),
            "",
        );

        let (kf, source) = load_from_dir(tmp.path()).unwrap();
        assert_eq!(source, Source::PyProject);

        let dev = kf.tasks.get("dev").expect("dev task missing");
        let disp = dev.dispatcher.as_ref().expect("dev should be a dispatcher");
        assert_eq!(disp.extension, "django");
        let names: Vec<_> = disp.subcommands.keys().cloned().collect();
        assert_eq!(names, vec!["exportxml".to_string(), "migrate".to_string()]);

        let test = kf.tasks.get("test").expect("test task missing");
        assert!(
            test.dispatcher.is_none(),
            "pytest should not be a dispatcher"
        );
    }

    #[test]
    fn pyproject_without_manage_py_has_no_dispatcher() {
        let tmp = TempDir::new().unwrap();
        write(
            &tmp.path().join("pyproject.toml"),
            r#"[project]
name = "demo"

[tool.pdm.scripts]
test = "pytest"
lint = "ruff check ."
"#,
        );

        let (kf, _) = load_from_dir(tmp.path()).unwrap();
        for (name, task) in &kf.tasks {
            assert!(
                task.dispatcher.is_none(),
                "{name} unexpectedly has dispatcher"
            );
        }
    }

    #[test]
    fn pyproject_entry_point_task_gains_dispatcher() {
        let tmp = TempDir::new().unwrap();
        write(
            &tmp.path().join("pyproject.toml"),
            r#"[project]
name = "demo"

[tool.pdm.scripts]
ccm-admin = "ccm.__main__:main"
"#,
        );
        write(&tmp.path().join("src/manage.py"), "");
        write(&tmp.path().join("src/ccm/management/commands/doit.py"), "");

        let (kf, _) = load_from_dir(tmp.path()).unwrap();
        let task = kf.tasks.get("ccm-admin").unwrap();
        let disp = task.dispatcher.as_ref().unwrap();
        assert_eq!(disp.extension, "django");
        assert!(disp.subcommands.contains_key("doit"));
    }

    #[test]
    fn package_json_task_gets_no_django_dispatcher() {
        let tmp = TempDir::new().unwrap();
        // Even if there's a file named manage.py lying around, Django detection
        // is gated on SourceHint::PyProject, so package.json tasks stay clean.
        write(&tmp.path().join("manage.py"), "");
        write(
            &tmp.path().join("package.json"),
            r#"{"scripts": {"start": "node src/manage.py"}}"#,
        );

        let (kf, source) = load_from_dir(tmp.path()).unwrap();
        assert_eq!(source, Source::PackageJson);
        assert!(kf.tasks.get("start").unwrap().dispatcher.is_none());
    }
}
