use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const SKIP_DIRS: &[&str] = &[
    "node_modules",
    "target",
    ".git",
    ".hg",
    ".svn",
    "vendor",
    "__pycache__",
    ".venv",
    "venv",
    "dist",
    "build",
    ".next",
    ".nuxt",
];

const PROJECT_FILES: &[&str] = &[
    "Kylefile",
    "Kylefile.yaml",
    "Kylefile.yml",
    "Kylefile.toml",
    "Makefile",
    "makefile",
    "GNUmakefile",
    "justfile",
    "Justfile",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileType {
    Kylefile,
    Makefile,
    Justfile,
}

#[derive(Debug, Clone)]
pub struct DiscoveredNamespace {
    pub alias: String,
    pub path: PathBuf,
    pub file_type: FileType,
}

/// Recursively scan for project files starting from root
/// Returns a list of discovered namespaces with their aliases and paths
pub fn discover_namespaces(root: &Path) -> Vec<DiscoveredNamespace> {
    let mut namespaces = Vec::new();

    let walker = WalkDir::new(root)
        .min_depth(1) // Skip the root directory itself
        .into_iter()
        .filter_entry(|e| {
            if !e.file_type().is_dir() {
                return true;
            }
            let name = e.file_name().to_str().unwrap_or("");
            !SKIP_DIRS.contains(&name)
        });

    for entry in walker.filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }

        let filename = entry.file_name().to_str().unwrap_or("");
        if !PROJECT_FILES.contains(&filename) {
            continue;
        }

        let file_type = detect_file_type(filename);
        let dir = entry.path().parent().unwrap_or(entry.path());

        // Create alias from relative path
        if let Ok(relative) = dir.strip_prefix(root) {
            let alias = relative.to_string_lossy().replace('\\', "/");
            if !alias.is_empty() {
                namespaces.push(DiscoveredNamespace {
                    alias,
                    path: dir.to_path_buf(),
                    file_type,
                });
            }
        }
    }

    // Sort by alias for consistent output
    namespaces.sort_by(|a, b| a.alias.cmp(&b.alias));

    // Deduplicate - keep only first file found per directory (Kylefile takes precedence)
    namespaces.dedup_by(|a, b| a.alias == b.alias);

    namespaces
}

fn detect_file_type(filename: &str) -> FileType {
    match filename {
        "Makefile" | "makefile" | "GNUmakefile" => FileType::Makefile,
        "justfile" | "Justfile" => FileType::Justfile,
        _ => FileType::Kylefile,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_discover_namespaces_empty_dir() {
        let temp = TempDir::new().unwrap();
        let namespaces = discover_namespaces(temp.path());
        assert!(namespaces.is_empty());
    }

    #[test]
    fn test_discover_single_namespace() {
        let temp = TempDir::new().unwrap();
        let backend = temp.path().join("backend");
        fs::create_dir(&backend).unwrap();
        fs::write(backend.join("Kylefile"), "").unwrap();

        let namespaces = discover_namespaces(temp.path());
        assert_eq!(namespaces.len(), 1);
        assert_eq!(namespaces[0].alias, "backend");
        assert_eq!(namespaces[0].file_type, FileType::Kylefile);
    }

    #[test]
    fn test_discover_nested_namespace() {
        let temp = TempDir::new().unwrap();
        let apps = temp.path().join("apps").join("frontend");
        fs::create_dir_all(&apps).unwrap();
        fs::write(apps.join("Kylefile.toml"), "").unwrap();

        let namespaces = discover_namespaces(temp.path());
        assert_eq!(namespaces.len(), 1);
        assert_eq!(namespaces[0].alias, "apps/frontend");
    }

    #[test]
    fn test_skip_node_modules() {
        let temp = TempDir::new().unwrap();
        let node_modules = temp.path().join("node_modules").join("some-pkg");
        fs::create_dir_all(&node_modules).unwrap();
        fs::write(node_modules.join("Makefile"), "").unwrap();

        let namespaces = discover_namespaces(temp.path());
        assert!(namespaces.is_empty());
    }

    #[test]
    fn test_detect_makefile_type() {
        let temp = TempDir::new().unwrap();
        let backend = temp.path().join("backend");
        fs::create_dir(&backend).unwrap();
        fs::write(backend.join("Makefile"), "").unwrap();

        let namespaces = discover_namespaces(temp.path());
        assert_eq!(namespaces.len(), 1);
        assert_eq!(namespaces[0].file_type, FileType::Makefile);
    }

    #[test]
    fn test_detect_justfile_type() {
        let temp = TempDir::new().unwrap();
        let backend = temp.path().join("backend");
        fs::create_dir(&backend).unwrap();
        fs::write(backend.join("justfile"), "").unwrap();

        let namespaces = discover_namespaces(temp.path());
        assert_eq!(namespaces.len(), 1);
        assert_eq!(namespaces[0].file_type, FileType::Justfile);
    }

    #[test]
    fn test_kylefile_takes_precedence() {
        let temp = TempDir::new().unwrap();
        let backend = temp.path().join("backend");
        fs::create_dir(&backend).unwrap();
        fs::write(backend.join("Kylefile"), "").unwrap();
        fs::write(backend.join("Makefile"), "").unwrap();

        let namespaces = discover_namespaces(temp.path());
        assert_eq!(namespaces.len(), 1);
        assert_eq!(namespaces[0].file_type, FileType::Kylefile);
    }
}
