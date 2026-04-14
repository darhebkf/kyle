use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SourceHint {
    PyProject,
    PackageJson,
    ComposerJson,
    DenoJson,
    Kylefile,
    Other,
}

#[derive(Debug)]
pub struct DispatcherContext<'a> {
    pub dir: &'a Path,
    pub task_name: &'a str,
    pub command: &'a str,
    pub source_hint: SourceHint,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Subcommand {
    pub name: String,
    pub desc: Option<String>,
}

impl Subcommand {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            desc: None,
        }
    }

    pub fn with_desc(name: impl Into<String>, desc: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            desc: Some(desc.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expansion {
    pub extension: String,
    pub subcommands: BTreeMap<String, Subcommand>,
}

pub trait DispatcherExtension: Send + Sync {
    fn id(&self) -> &'static str;
    fn detect(&self, ctx: &DispatcherContext<'_>) -> bool;
    fn enumerate(&self, ctx: &DispatcherContext<'_>) -> Vec<Subcommand>;
}

pub struct DispatcherRegistry {
    extensions: Vec<Box<dyn DispatcherExtension>>,
}

impl Default for DispatcherRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl DispatcherRegistry {
    pub fn new() -> Self {
        Self {
            extensions: Vec::new(),
        }
    }

    pub fn builtin() -> Self {
        Self::new()
    }

    pub fn register(&mut self, ext: Box<dyn DispatcherExtension>) {
        self.extensions.push(ext);
    }

    pub fn try_expand(&self, ctx: &DispatcherContext<'_>) -> Option<Expansion> {
        for ext in &self.extensions {
            if ext.detect(ctx) {
                let subcommands = ext
                    .enumerate(ctx)
                    .into_iter()
                    .map(|s| (s.name.clone(), s))
                    .collect();
                return Some(Expansion {
                    extension: ext.id().to_string(),
                    subcommands,
                });
            }
        }
        None
    }

    pub fn len(&self) -> usize {
        self.extensions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.extensions.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct FakeExt {
        id: &'static str,
        claim: &'static str,
        subs: Vec<Subcommand>,
        enumerate_calls: Arc<AtomicUsize>,
    }

    impl FakeExt {
        fn new(id: &'static str, claim: &'static str, subs: Vec<Subcommand>) -> Self {
            Self {
                id,
                claim,
                subs,
                enumerate_calls: Arc::new(AtomicUsize::new(0)),
            }
        }
    }

    impl DispatcherExtension for FakeExt {
        fn id(&self) -> &'static str {
            self.id
        }

        fn detect(&self, ctx: &DispatcherContext<'_>) -> bool {
            ctx.task_name == self.claim
        }

        fn enumerate(&self, _ctx: &DispatcherContext<'_>) -> Vec<Subcommand> {
            self.enumerate_calls.fetch_add(1, Ordering::SeqCst);
            self.subs.clone()
        }
    }

    fn ctx<'a>(task_name: &'a str, dir: &'a Path) -> DispatcherContext<'a> {
        DispatcherContext {
            dir,
            task_name,
            command: "irrelevant",
            source_hint: SourceHint::PyProject,
        }
    }

    #[test]
    fn new_registry_is_empty() {
        let reg = DispatcherRegistry::new();
        assert!(reg.is_empty());
        assert_eq!(reg.len(), 0);
    }

    #[test]
    fn try_expand_returns_none_without_extensions() {
        let reg = DispatcherRegistry::new();
        let dir = PathBuf::from("/tmp/x");
        assert_eq!(reg.try_expand(&ctx("anything", &dir)), None);
    }

    #[test]
    fn try_expand_returns_none_when_no_match() {
        let mut reg = DispatcherRegistry::new();
        reg.register(Box::new(FakeExt::new("django", "manage", vec![])));
        let dir = PathBuf::from("/tmp/x");
        assert_eq!(reg.try_expand(&ctx("rails", &dir)), None);
    }

    #[test]
    fn try_expand_returns_first_match() {
        let mut reg = DispatcherRegistry::new();
        reg.register(Box::new(FakeExt::new(
            "django",
            "ccm-admin",
            vec![Subcommand::new("migrate"), Subcommand::new("exportxml")],
        )));
        let dir = PathBuf::from("/tmp/x");
        let exp = reg.try_expand(&ctx("ccm-admin", &dir)).unwrap();

        assert_eq!(exp.extension, "django");
        assert_eq!(exp.subcommands.len(), 2);
        assert!(exp.subcommands.contains_key("migrate"));
        assert!(exp.subcommands.contains_key("exportxml"));
    }

    #[test]
    fn registration_order_determines_precedence() {
        let first = FakeExt::new("first", "claim-me", vec![Subcommand::new("a")]);
        let second = FakeExt::new("second", "claim-me", vec![Subcommand::new("b")]);
        let second_enum = second.enumerate_calls.clone();

        let mut reg = DispatcherRegistry::new();
        reg.register(Box::new(first));
        reg.register(Box::new(second));

        let dir = PathBuf::from("/tmp/x");
        let exp = reg.try_expand(&ctx("claim-me", &dir)).unwrap();

        assert_eq!(exp.extension, "first");
        assert_eq!(second_enum.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn empty_subcommands_is_a_valid_claim() {
        let mut reg = DispatcherRegistry::new();
        reg.register(Box::new(FakeExt::new("empty", "boss", vec![])));
        let dir = PathBuf::from("/tmp/x");
        let exp = reg.try_expand(&ctx("boss", &dir)).unwrap();
        assert_eq!(exp.extension, "empty");
        assert!(exp.subcommands.is_empty());
    }

    #[test]
    fn builtin_registry_constructs() {
        let _ = DispatcherRegistry::builtin();
    }
}
