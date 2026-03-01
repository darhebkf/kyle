use super::Error;
use super::kylefile::{Kylefile, Task};
use std::collections::HashMap;

pub fn parse(content: &str) -> Result<Kylefile, Error> {
    let tf: serde_json::Value = serde_yml::from_str(content)?;
    let mut tasks = HashMap::new();

    if let Some(task_map) = tf["tasks"].as_object() {
        for (name, def) in task_map {
            // Support both object format and simple string format
            if let Some(cmd) = def.as_str() {
                tasks.insert(
                    name.clone(),
                    Task {
                        run: cmd.to_string(),
                        ..Default::default()
                    },
                );
                continue;
            }

            let desc = def["desc"].as_str().unwrap_or("").to_string();
            let cmds: Vec<String> = def["cmds"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();
            let deps: Vec<String> = def["deps"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();

            tasks.insert(
                name.clone(),
                Task {
                    desc,
                    run: cmds.join(" && "),
                    deps,
                },
            );
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
    fn parse_taskfile() {
        let content = r#"
version: '3'
tasks:
  build:
    desc: Build the project
    cmds:
      - go build -o main .
    deps:
      - generate
  generate:
    cmds:
      - go generate ./...
"#;
        let kf = parse(content).unwrap();
        assert_eq!(kf.tasks.len(), 2);
        assert_eq!(kf.tasks["build"].desc, "Build the project");
        assert_eq!(kf.tasks["build"].run, "go build -o main .");
        assert_eq!(kf.tasks["build"].deps, vec!["generate"]);
        assert_eq!(kf.tasks["generate"].run, "go generate ./...");
    }

    #[test]
    fn parse_multi_cmds() {
        let content = r#"
version: '3'
tasks:
  build:
    cmds:
      - echo step1
      - echo step2
      - echo step3
"#;
        let kf = parse(content).unwrap();
        assert_eq!(
            kf.tasks["build"].run,
            "echo step1 && echo step2 && echo step3"
        );
    }

    #[test]
    fn parse_simple_string_task() {
        let content = r#"
version: '3'
tasks:
  hello: echo hello
"#;
        let kf = parse(content).unwrap();
        assert_eq!(kf.tasks["hello"].run, "echo hello");
    }
}
