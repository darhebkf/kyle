use super::Error;
use super::kylefile::{Kylefile, Task};
use std::collections::HashMap;

pub fn parse(content: &str) -> Result<Kylefile, Error> {
    let deno: serde_json::Value = serde_json::from_str(content)?;
    let mut tasks = HashMap::new();

    if let Some(task_map) = deno["tasks"].as_object() {
        for (key, val) in task_map {
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
        tasks,
        ..Default::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_tasks() {
        let content = r#"{"tasks": {"start": "deno run main.ts", "test": "deno test"}}"#;
        let kf = parse(content).unwrap();
        assert_eq!(kf.tasks.len(), 2);
        assert_eq!(kf.tasks["start"].run, "deno run main.ts");
        assert_eq!(kf.tasks["test"].run, "deno test");
    }

    #[test]
    fn parse_no_tasks() {
        let content = r#"{"compilerOptions": {}}"#;
        let kf = parse(content).unwrap();
        assert!(kf.tasks.is_empty());
    }

    #[test]
    fn parse_empty_tasks() {
        let content = r#"{"tasks": {}}"#;
        let kf = parse(content).unwrap();
        assert!(kf.tasks.is_empty());
    }
}
