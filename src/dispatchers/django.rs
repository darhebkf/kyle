use super::{DispatcherContext, DispatcherExtension, SourceHint, Subcommand};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const SKIP_DIRS: &[&str] = &[
    ".venv",
    "venv",
    "__pycache__",
    "node_modules",
    ".git",
    "target",
    "build",
    "dist",
    "migrations",
];

const FALLBACK_MANAGE_PY_PATHS: &[&str] = &["manage.py", "src/manage.py"];

#[derive(Default)]
pub struct DjangoExtension;

impl DjangoExtension {
    pub fn new() -> Self {
        Self
    }

    fn locate_manage_py(&self, ctx: &DispatcherContext<'_>) -> Option<PathBuf> {
        for token in ctx.command.split_whitespace() {
            if token.ends_with("manage.py") {
                let candidate = ctx.dir.join(token);
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }
        for rel in FALLBACK_MANAGE_PY_PATHS {
            let candidate = ctx.dir.join(rel);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
        None
    }

    fn looks_like_entry_point(cmd: &str) -> bool {
        let cmd = cmd.trim();
        !cmd.is_empty()
            && !cmd.contains(' ')
            && !cmd.contains('/')
            && !cmd.contains('\\')
            && cmd.contains(':')
            && cmd.contains('.')
    }
}

fn entry_point_signals_django(entry_point: &str) -> bool {
    DjangoExtension::looks_like_entry_point(entry_point)
}

impl DispatcherExtension for DjangoExtension {
    fn id(&self) -> &'static str {
        "django"
    }

    fn detect(&self, ctx: &DispatcherContext<'_>) -> bool {
        if ctx.source_hint != SourceHint::PyProject {
            return false;
        }
        // Path 1: task command directly references manage.py (pdm/hatch/rye scripts)
        if ctx.command.contains("manage.py") && self.locate_manage_py(ctx).is_some() {
            return true;
        }
        // Path 2: task is a [project.scripts] entry whose entry_point module
        // looks like a Django CLI wrapper — claim when manage.py is nearby.
        if ctx
            .entry_point
            .map(entry_point_signals_django)
            .unwrap_or(false)
            && self.locate_manage_py(ctx).is_some()
        {
            return true;
        }
        // Path 3 (legacy): command itself looks like an entry point — keeps
        // dispatcher detection working for pdm scripts that happen to be set
        // to a raw entry-point reference rather than a manage.py path.
        if Self::looks_like_entry_point(ctx.command) && self.locate_manage_py(ctx).is_some() {
            return true;
        }
        false
    }

    fn enumerate(&self, ctx: &DispatcherContext<'_>) -> Vec<Subcommand> {
        let Some(manage_py) = self.locate_manage_py(ctx) else {
            return Vec::new();
        };
        let root = manage_py.parent().unwrap_or(ctx.dir);
        scan_commands(root)
    }

    fn exec_prefix(&self, ctx: &DispatcherContext<'_>) -> String {
        match self.locate_manage_py(ctx) {
            Some(manage_py) => manage_py
                .strip_prefix(ctx.dir)
                .unwrap_or(&manage_py)
                .to_string_lossy()
                .into_owned(),
            None => "manage.py".to_string(),
        }
    }
}

fn scan_commands(root: &Path) -> Vec<Subcommand> {
    let mut subs: Vec<Subcommand> = WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            if !e.file_type().is_dir() {
                return true;
            }
            let name = e.file_name().to_str().unwrap_or("");
            !SKIP_DIRS.contains(&name)
        })
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.path().extension().map(|x| x == "py").unwrap_or(false))
        .filter(|e| {
            e.path()
                .parent()
                .map(|p| p.ends_with(Path::new("management/commands")))
                .unwrap_or(false)
        })
        .filter_map(|e| {
            let name = e.path().file_stem().and_then(|s| s.to_str())?.to_string();
            if name.is_empty() || name.starts_with('_') {
                return None;
            }
            let sub = Subcommand::new(name);
            match app_name_from_commands_path(e.path()) {
                Some(group) => Some(sub.with_group(group)),
                None => Some(sub),
            }
        })
        .collect();

    subs.sort_by(|a, b| a.name.cmp(&b.name));
    subs.dedup_by(|a, b| a.name == b.name);
    subs
}

// Extract the Django app name from a management command path.
// Path shape: `.../<app>/management/commands/<cmd>.py`
// Returns `<app>` — the directory immediately before `management/`.
fn app_name_from_commands_path(path: &Path) -> Option<String> {
    let commands_dir = path.parent()?;
    let management_dir = commands_dir.parent()?;
    let app_dir = management_dir.parent()?;
    app_dir.file_name()?.to_str().map(String::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn ctx<'a>(
        dir: &'a Path,
        task_name: &'a str,
        command: &'a str,
        source_hint: SourceHint,
    ) -> DispatcherContext<'a> {
        DispatcherContext {
            dir,
            task_name,
            command,
            entry_point: None,
            source_hint,
        }
    }

    fn ctx_with_entry_point<'a>(
        dir: &'a Path,
        task_name: &'a str,
        command: &'a str,
        entry_point: &'a str,
        source_hint: SourceHint,
    ) -> DispatcherContext<'a> {
        DispatcherContext {
            dir,
            task_name,
            command,
            entry_point: Some(entry_point),
            source_hint,
        }
    }

    fn write_manage_py(root: &Path, rel: &str) {
        let path = root.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, "#!/usr/bin/env python\n").unwrap();
    }

    fn write_command(root: &Path, app: &str, name: &str) {
        let dir = root.join(app).join("management").join("commands");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join(format!("{name}.py")), "").unwrap();
    }

    #[test]
    fn rejects_non_pyproject_source() {
        let tmp = TempDir::new().unwrap();
        write_manage_py(tmp.path(), "manage.py");
        let ext = DjangoExtension::new();
        assert!(!ext.detect(&ctx(
            tmp.path(),
            "dev",
            "src/manage.py runserver",
            SourceHint::PackageJson,
        )));
    }

    #[test]
    fn rejects_command_with_no_manage_py_file() {
        let tmp = TempDir::new().unwrap();
        let ext = DjangoExtension::new();
        assert!(!ext.detect(&ctx(
            tmp.path(),
            "dev",
            "src/manage.py runserver",
            SourceHint::PyProject,
        )));
    }

    #[test]
    fn accepts_direct_manage_py_reference() {
        let tmp = TempDir::new().unwrap();
        write_manage_py(tmp.path(), "src/manage.py");
        let ext = DjangoExtension::new();
        assert!(ext.detect(&ctx(
            tmp.path(),
            "dev",
            "src/manage.py runserver",
            SourceHint::PyProject,
        )));
    }

    #[test]
    fn accepts_python_wrapped_manage_py() {
        let tmp = TempDir::new().unwrap();
        write_manage_py(tmp.path(), "src/manage.py");
        let ext = DjangoExtension::new();
        assert!(ext.detect(&ctx(
            tmp.path(),
            "dev",
            "python src/manage.py runserver --noreload",
            SourceHint::PyProject,
        )));
    }

    #[test]
    fn accepts_entry_point_when_manage_py_nearby() {
        let tmp = TempDir::new().unwrap();
        write_manage_py(tmp.path(), "src/manage.py");
        let ext = DjangoExtension::new();
        assert!(ext.detect(&ctx(
            tmp.path(),
            "ccm-admin",
            "ccm.__main__:main",
            SourceHint::PyProject,
        )));
    }

    #[test]
    fn accepts_project_scripts_task_via_entry_point_metadata() {
        // Simulates a [project.scripts] parse result: run is the script name
        // (so the bare task actually executes), entry_point carries the ref.
        let tmp = TempDir::new().unwrap();
        write_manage_py(tmp.path(), "src/manage.py");
        let ext = DjangoExtension::new();
        assert!(ext.detect(&ctx_with_entry_point(
            tmp.path(),
            "ccm-admin",
            "ccm-admin",
            "ccm.__main__:main",
            SourceHint::PyProject,
        )));
    }

    #[test]
    fn rejects_project_scripts_task_without_manage_py() {
        let tmp = TempDir::new().unwrap();
        let ext = DjangoExtension::new();
        assert!(!ext.detect(&ctx_with_entry_point(
            tmp.path(),
            "my-cli",
            "my-cli",
            "my_pkg:main",
            SourceHint::PyProject,
        )));
    }

    #[test]
    fn rejects_entry_point_without_manage_py() {
        let tmp = TempDir::new().unwrap();
        let ext = DjangoExtension::new();
        assert!(!ext.detect(&ctx(
            tmp.path(),
            "ccm-admin",
            "ccm.__main__:main",
            SourceHint::PyProject,
        )));
    }

    #[test]
    fn looks_like_entry_point_examples() {
        assert!(DjangoExtension::looks_like_entry_point("ccm.__main__:main"));
        assert!(DjangoExtension::looks_like_entry_point("foo.bar.baz:run"));
        assert!(!DjangoExtension::looks_like_entry_point("pytest"));
        assert!(!DjangoExtension::looks_like_entry_point("src/manage.py"));
        assert!(!DjangoExtension::looks_like_entry_point(""));
        assert!(!DjangoExtension::looks_like_entry_point("python -m foo"));
    }

    #[test]
    fn enumerate_empty_when_no_commands_dir() {
        let tmp = TempDir::new().unwrap();
        write_manage_py(tmp.path(), "manage.py");
        let ext = DjangoExtension::new();
        let subs = ext.enumerate(&ctx(tmp.path(), "dev", "manage.py", SourceHint::PyProject));
        assert!(subs.is_empty());
    }

    #[test]
    fn enumerate_finds_commands_in_single_app() {
        let tmp = TempDir::new().unwrap();
        write_manage_py(tmp.path(), "manage.py");
        write_command(tmp.path(), "app", "migrate");
        write_command(tmp.path(), "app", "shell");
        let ext = DjangoExtension::new();
        let subs = ext.enumerate(&ctx(tmp.path(), "dev", "manage.py", SourceHint::PyProject));
        let names: Vec<_> = subs.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(names, vec!["migrate", "shell"]);
    }

    #[test]
    fn enumerate_ignores_underscore_files() {
        let tmp = TempDir::new().unwrap();
        write_manage_py(tmp.path(), "manage.py");
        write_command(tmp.path(), "app", "__init__");
        write_command(tmp.path(), "app", "_private");
        write_command(tmp.path(), "app", "runjob");
        let ext = DjangoExtension::new();
        let subs = ext.enumerate(&ctx(tmp.path(), "dev", "manage.py", SourceHint::PyProject));
        let names: Vec<_> = subs.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(names, vec!["runjob"]);
    }

    #[test]
    fn enumerate_tags_each_sub_with_its_app_group() {
        let tmp = TempDir::new().unwrap();
        write_manage_py(tmp.path(), "manage.py");
        write_command(tmp.path(), "billing", "invoice");
        write_command(tmp.path(), "dfs", "cleanup");
        let ext = DjangoExtension::new();
        let subs = ext.enumerate(&ctx(tmp.path(), "dev", "manage.py", SourceHint::PyProject));
        let invoice = subs.iter().find(|s| s.name == "invoice").unwrap();
        let cleanup = subs.iter().find(|s| s.name == "cleanup").unwrap();
        assert_eq!(invoice.group.as_deref(), Some("billing"));
        assert_eq!(cleanup.group.as_deref(), Some("dfs"));
    }

    #[test]
    fn enumerate_tags_group_when_manage_py_in_src_and_app_nested() {
        let tmp = TempDir::new().unwrap();
        write_manage_py(tmp.path(), "src/manage.py");
        let dir = tmp
            .path()
            .join("src/surevoice/surevoice/management/commands");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("exportxml.py"), "").unwrap();
        let ext = DjangoExtension::new();
        let subs = ext.enumerate(&ctx(
            tmp.path(),
            "dev",
            "src/manage.py runserver",
            SourceHint::PyProject,
        ));
        assert_eq!(subs.len(), 1);
        assert_eq!(subs[0].group.as_deref(), Some("surevoice"));
    }

    #[test]
    fn enumerate_merges_commands_from_multiple_apps() {
        let tmp = TempDir::new().unwrap();
        write_manage_py(tmp.path(), "manage.py");
        write_command(tmp.path(), "billing", "invoice");
        write_command(tmp.path(), "users", "cleanup");
        write_command(tmp.path(), "nested/pkg/app", "exportxml");
        let ext = DjangoExtension::new();
        let subs = ext.enumerate(&ctx(tmp.path(), "dev", "manage.py", SourceHint::PyProject));
        let names: Vec<_> = subs.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(names, vec!["cleanup", "exportxml", "invoice"]);
    }

    #[test]
    fn enumerate_scans_from_src_when_manage_py_in_src() {
        let tmp = TempDir::new().unwrap();
        write_manage_py(tmp.path(), "src/manage.py");
        let dir = tmp.path().join("src/ccm/management/commands");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("exportxml.py"), "").unwrap();
        let ext = DjangoExtension::new();
        let subs = ext.enumerate(&ctx(
            tmp.path(),
            "dev",
            "src/manage.py runserver",
            SourceHint::PyProject,
        ));
        let names: Vec<_> = subs.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(names, vec!["exportxml"]);
    }

    #[test]
    fn enumerate_skips_venv_noise() {
        let tmp = TempDir::new().unwrap();
        write_manage_py(tmp.path(), "manage.py");
        write_command(tmp.path(), "app", "legit");
        // Simulate a venv copy of Django that would otherwise pollute results.
        let venv_cmds = tmp.path().join(".venv/lib/django/core/management/commands");
        fs::create_dir_all(&venv_cmds).unwrap();
        fs::write(venv_cmds.join("runserver.py"), "").unwrap();
        let ext = DjangoExtension::new();
        let subs = ext.enumerate(&ctx(tmp.path(), "dev", "manage.py", SourceHint::PyProject));
        let names: Vec<_> = subs.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(names, vec!["legit"]);
    }
}
