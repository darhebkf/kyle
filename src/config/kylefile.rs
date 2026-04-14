use crate::dispatchers::Dispatcher;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Task {
    #[serde(default)]
    pub desc: String,
    #[serde(default)]
    pub run: String,
    #[serde(default)]
    pub deps: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dispatcher: Option<Dispatcher>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(untagged)]
pub enum Includes {
    #[default]
    None,
    List(Vec<String>),
    Map(HashMap<String, String>),
}

impl Includes {
    pub fn is_empty(&self) -> bool {
        match self {
            Includes::None => true,
            Includes::List(list) => list.is_empty(),
            Includes::Map(map) => map.is_empty(),
        }
    }

    pub fn iter(&self) -> Box<dyn Iterator<Item = (&str, &str)> + '_> {
        match self {
            Includes::None => Box::new(std::iter::empty()),
            Includes::List(list) => Box::new(list.iter().map(|path| {
                // Extract alias from path (last component)
                let alias = std::path::Path::new(path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(path);
                (alias, path.as_str())
            })),
            Includes::Map(map) => Box::new(
                map.iter()
                    .map(|(alias, path)| (alias.as_str(), path.as_str())),
            ),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Kylefile {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub includes: Includes,
    #[serde(default)]
    pub tasks: HashMap<String, Task>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dispatchers::Subcommand;

    #[test]
    fn default_task_has_no_dispatcher() {
        let task = Task::default();
        assert!(task.dispatcher.is_none());
    }

    #[test]
    fn dispatcher_field_omitted_when_none_in_toml() {
        let task = Task {
            run: "echo hi".into(),
            ..Default::default()
        };
        let out = toml::to_string(&task).unwrap();
        assert!(!out.contains("dispatcher"), "got: {out}");
    }

    #[test]
    fn existing_toml_without_dispatcher_still_parses() {
        let src = r#"
run = "cargo build"
desc = "build"
"#;
        let task: Task = toml::from_str(src).unwrap();
        assert_eq!(task.run, "cargo build");
        assert_eq!(task.desc, "build");
        assert!(task.dispatcher.is_none());
    }

    #[test]
    fn task_with_dispatcher_roundtrips_yaml() {
        let mut subs = std::collections::BTreeMap::new();
        subs.insert(
            "migrate".to_string(),
            Subcommand::with_desc("migrate", "apply migrations"),
        );
        subs.insert("shell".to_string(), Subcommand::new("shell"));

        let task = Task {
            run: "src/manage.py".into(),
            dispatcher: Some(Dispatcher {
                extension: "django".into(),
                subcommands: subs,
            }),
            ..Default::default()
        };

        let yaml = serde_yml::to_string(&task).unwrap();
        let parsed: Task = serde_yml::from_str(&yaml).unwrap();
        let disp = parsed.dispatcher.unwrap();
        assert_eq!(disp.extension, "django");
        assert_eq!(disp.subcommands.len(), 2);
        assert_eq!(
            disp.subcommands["migrate"].desc.as_deref(),
            Some("apply migrations")
        );
    }
}
