use super::Error;
use super::kylefile::{Kylefile, Task};
use std::collections::HashMap;

pub fn parse(content: &str) -> Result<Kylefile, Error> {
    let pkg: serde_json::Value = serde_json::from_str(content)?;
    let name = pkg["name"].as_str().unwrap_or("").to_string();
    let mut tasks = HashMap::new();

    if let Some(scripts) = pkg["scripts"].as_object() {
        for (key, val) in scripts {
            if let Some(cmd) = val.as_str() {
                tasks.insert(
                    key.clone(),
                    Task {
                        run: cmd.to_string(),
                        ..Default::default()
                    },
                );
            }
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
        let content = r#"{"name": "my-app", "scripts": {"build": "tsc", "test": "vitest"}}"#;
        let kf = parse(content).unwrap();
        assert_eq!(kf.name, "my-app");
        assert_eq!(kf.tasks.len(), 2);
        assert_eq!(kf.tasks["build"].run, "tsc");
        assert_eq!(kf.tasks["test"].run, "vitest");
    }

    #[test]
    fn parse_no_scripts() {
        let content = r#"{"name": "my-app", "version": "1.0.0"}"#;
        let kf = parse(content).unwrap();
        assert!(kf.tasks.is_empty());
    }

    #[test]
    fn parse_empty_scripts() {
        let content = r#"{"name": "my-app", "scripts": {}}"#;
        let kf = parse(content).unwrap();
        assert!(kf.tasks.is_empty());
    }

    #[test]
    fn parse_compound_command() {
        let content = r#"{"scripts": {"build": "tsc && vite build"}}"#;
        let kf = parse(content).unwrap();
        assert_eq!(kf.tasks["build"].run, "tsc && vite build");
    }
}
