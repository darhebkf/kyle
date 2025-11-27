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
