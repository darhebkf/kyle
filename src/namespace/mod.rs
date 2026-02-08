pub mod discovery;

use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskRef {
    pub namespace: Option<String>,
    pub task_name: String,
}

impl TaskRef {
    pub fn new(task_name: impl Into<String>) -> Self {
        Self {
            namespace: None,
            task_name: task_name.into(),
        }
    }

    pub fn with_namespace(namespace: impl Into<String>, task_name: impl Into<String>) -> Self {
        Self {
            namespace: Some(namespace.into()),
            task_name: task_name.into(),
        }
    }

    pub fn is_namespaced(&self) -> bool {
        self.namespace.is_some()
    }
}

/// Parse a task reference like "backend:build", "backend.build", or "apps/frontend:test"
/// Returns TaskRef with namespace and task name separated
pub fn parse_task_ref(input: &str) -> TaskRef {
    // Find the rightmost separator (either ':' or '.')
    let colon_pos = input.rfind(':');
    let dot_pos = input.rfind('.');

    let split_pos = match (colon_pos, dot_pos) {
        (Some(c), Some(d)) => Some(c.max(d)), // Use rightmost
        (Some(c), None) => Some(c),
        (None, Some(d)) => Some(d),
        (None, None) => None,
    };

    match split_pos {
        Some(pos) if pos > 0 && pos < input.len() - 1 => {
            let namespace = &input[..pos];
            let task = &input[pos + 1..];
            TaskRef::with_namespace(namespace, task)
        }
        _ => TaskRef::new(input),
    }
}

/// Resolve a namespace path relative to a base directory
/// Returns the absolute path to the namespace directory
pub fn resolve_namespace(base: &Path, namespace: &str) -> PathBuf {
    base.join(namespace)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_task() {
        let task_ref = parse_task_ref("build");
        assert_eq!(task_ref.namespace, None);
        assert_eq!(task_ref.task_name, "build");
        assert!(!task_ref.is_namespaced());
    }

    #[test]
    fn test_parse_namespaced_task_colon() {
        let task_ref = parse_task_ref("backend:build");
        assert_eq!(task_ref.namespace, Some("backend".into()));
        assert_eq!(task_ref.task_name, "build");
        assert!(task_ref.is_namespaced());
    }

    #[test]
    fn test_parse_namespaced_task_dot() {
        let task_ref = parse_task_ref("backend.build");
        assert_eq!(task_ref.namespace, Some("backend".into()));
        assert_eq!(task_ref.task_name, "build");
        assert!(task_ref.is_namespaced());
    }

    #[test]
    fn test_parse_nested_namespace_colon() {
        let task_ref = parse_task_ref("apps/frontend:test");
        assert_eq!(task_ref.namespace, Some("apps/frontend".into()));
        assert_eq!(task_ref.task_name, "test");
    }

    #[test]
    fn test_parse_nested_namespace_dot() {
        let task_ref = parse_task_ref("apps/frontend.test");
        assert_eq!(task_ref.namespace, Some("apps/frontend".into()));
        assert_eq!(task_ref.task_name, "test");
    }

    #[test]
    fn test_parse_mixed_separators_colon_last() {
        // "backend.sub:build" -> namespace="backend.sub", task="build"
        let task_ref = parse_task_ref("backend.sub:build");
        assert_eq!(task_ref.namespace, Some("backend.sub".into()));
        assert_eq!(task_ref.task_name, "build");
    }

    #[test]
    fn test_parse_mixed_separators_dot_last() {
        // "backend:sub.build" -> namespace="backend:sub", task="build"
        let task_ref = parse_task_ref("backend:sub.build");
        assert_eq!(task_ref.namespace, Some("backend:sub".into()));
        assert_eq!(task_ref.task_name, "build");
    }

    #[test]
    fn test_parse_empty_namespace_colon() {
        let task_ref = parse_task_ref(":build");
        assert_eq!(task_ref.namespace, None);
        assert_eq!(task_ref.task_name, ":build");
    }

    #[test]
    fn test_parse_empty_namespace_dot() {
        let task_ref = parse_task_ref(".build");
        assert_eq!(task_ref.namespace, None);
        assert_eq!(task_ref.task_name, ".build");
    }

    #[test]
    fn test_parse_empty_task_colon() {
        let task_ref = parse_task_ref("backend:");
        assert_eq!(task_ref.namespace, None);
        assert_eq!(task_ref.task_name, "backend:");
    }

    #[test]
    fn test_parse_empty_task_dot() {
        let task_ref = parse_task_ref("backend.");
        assert_eq!(task_ref.namespace, None);
        assert_eq!(task_ref.task_name, "backend.");
    }

    #[test]
    fn test_resolve_namespace() {
        let base = Path::new("/project");
        let resolved = resolve_namespace(base, "backend");
        assert_eq!(resolved, PathBuf::from("/project/backend"));
    }

    #[test]
    fn test_resolve_nested_namespace() {
        let base = Path::new("/project");
        let resolved = resolve_namespace(base, "apps/frontend");
        assert_eq!(resolved, PathBuf::from("/project/apps/frontend"));
    }
}
