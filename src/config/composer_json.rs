use super::Error;
use super::kylefile::{Kylefile, Task};
use std::collections::HashMap;

pub fn parse(content: &str) -> Result<Kylefile, Error> {
    let pkg: serde_json::Value = serde_json::from_str(content)?;
    let name = pkg["name"].as_str().unwrap_or("").to_string();
    let mut tasks = HashMap::new();

    if let Some(scripts) = pkg["scripts"].as_object() {
        for (key, val) in scripts {
            // Skip composer lifecycle hooks
            if key.starts_with("pre-") || key.starts_with("post-") {
                continue;
            }

            let cmd = match val {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Array(arr) => arr
                    .iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect::<Vec<_>>()
                    .join(" && "),
                _ => continue,
            };

            tasks.insert(
                key.clone(),
                Task {
                    run: cmd,
                    ..Default::default()
                },
            );
        }
    }

    Ok(Kylefile {
        name,
        tasks,
        ..Default::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_scripts() {
        let content =
            r#"{"name": "vendor/pkg", "scripts": {"test": "phpunit", "lint": "phpcs src/"}}"#;
        let kf = parse(content).unwrap();
        assert_eq!(kf.tasks.len(), 2);
        assert_eq!(kf.tasks["test"].run, "phpunit");
    }

    #[test]
    fn skip_lifecycle_hooks() {
        let content = r#"{"scripts": {"pre-install-cmd": "echo pre", "test": "phpunit", "post-update-cmd": "echo post"}}"#;
        let kf = parse(content).unwrap();
        assert_eq!(kf.tasks.len(), 1);
        assert!(kf.tasks.contains_key("test"));
    }

    #[test]
    fn parse_array_scripts() {
        let content = r#"{"scripts": {"check": ["phpcs", "phpstan"]}}"#;
        let kf = parse(content).unwrap();
        assert_eq!(kf.tasks["check"].run, "phpcs && phpstan");
    }
}
