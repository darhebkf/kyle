use super::Error;
use super::kylefile::{Kylefile, Task};
use std::collections::HashMap;

pub fn parse(content: &str) -> Result<Kylefile, Error> {
    let doc: toml::Value = toml::from_str(content)?;
    let mut tasks = HashMap::new();

    // Try PDM scripts: [tool.pdm.scripts]
    if let Some(scripts) = doc
        .get("tool")
        .and_then(|t| t.get("pdm"))
        .and_then(|p| p.get("scripts"))
        .and_then(|s| s.as_table())
    {
        for (name, val) in scripts {
            if let Some(cmd) = extract_script_cmd(val) {
                tasks.insert(
                    name.clone(),
                    Task {
                        run: cmd,
                        ..Default::default()
                    },
                );
            }
        }
    }

    // Try Hatch scripts: [tool.hatch.envs.default.scripts]
    if tasks.is_empty()
        && let Some(scripts) = doc
            .get("tool")
            .and_then(|t| t.get("hatch"))
            .and_then(|h| h.get("envs"))
            .and_then(|e| e.get("default"))
            .and_then(|d| d.get("scripts"))
            .and_then(|s| s.as_table())
    {
        for (name, val) in scripts {
            if let Some(cmd) = extract_script_cmd(val) {
                tasks.insert(
                    name.clone(),
                    Task {
                        run: cmd,
                        ..Default::default()
                    },
                );
            }
        }
    }

    if tasks.is_empty()
        && let Some(scripts) = doc
            .get("tool")
            .and_then(|t| t.get("rye"))
            .and_then(|r| r.get("scripts"))
            .and_then(|s| s.as_table())
    {
        for (name, val) in scripts {
            if let Some(cmd) = extract_script_cmd(val) {
                tasks.insert(
                    name.clone(),
                    Task {
                        run: cmd,
                        ..Default::default()
                    },
                );
            }
        }
    }

    // Fallback: generate standard Python tasks
    if tasks.is_empty() {
        let name = doc
            .get("project")
            .and_then(|p| p.get("name"))
            .and_then(|n| n.as_str())
            .or_else(|| {
                doc.get("tool")
                    .and_then(|t| t.get("poetry"))
                    .and_then(|p| p.get("name"))
                    .and_then(|n| n.as_str())
            })
            .unwrap_or("");

        return Ok(Kylefile {
            name: name.to_string(),
            tasks: standard_python_tasks(),
            ..Default::default()
        });
    }

    let name = doc
        .get("project")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .unwrap_or("")
        .to_string();

    Ok(Kylefile {
        name,
        tasks,
        ..Default::default()
    })
}

fn extract_script_cmd(val: &toml::Value) -> Option<String> {
    match val {
        toml::Value::String(s) => Some(s.clone()),
        toml::Value::Array(arr) => {
            let cmds: Vec<String> = arr
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect();
            if cmds.is_empty() {
                None
            } else {
                Some(cmds.join(" && "))
            }
        }
        // PDM supports {cmd = "..."} format
        toml::Value::Table(t) => t.get("cmd").and_then(|c| c.as_str()).map(String::from),
        _ => None,
    }
}

fn standard_python_tasks() -> HashMap<String, Task> {
    let mut tasks = HashMap::new();
    let standard = [
        ("test", "pytest", "Run tests"),
        ("lint", "ruff check .", "Run linter"),
        ("format", "ruff format .", "Format code"),
        ("typecheck", "mypy .", "Run type checker"),
        ("install", "pip install -e .", "Install package"),
    ];
    for (name, cmd, desc) in standard {
        tasks.insert(
            name.to_string(),
            Task {
                desc: desc.to_string(),
                run: cmd.to_string(),
                ..Default::default()
            },
        );
    }
    tasks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_pdm_scripts() {
        let content = "[project]\nname = \"my-app\"\n\n[tool.pdm.scripts]\ntest = \"pytest\"\nlint = \"ruff check .\"";
        let kf = parse(content).unwrap();
        assert_eq!(kf.name, "my-app");
        assert_eq!(kf.tasks["test"].run, "pytest");
        assert_eq!(kf.tasks["lint"].run, "ruff check .");
    }

    #[test]
    fn parse_hatch_scripts() {
        let content =
            "[tool.hatch.envs.default.scripts]\ntest = \"pytest\"\ncov = \"pytest --cov\"";
        let kf = parse(content).unwrap();
        assert_eq!(kf.tasks["test"].run, "pytest");
    }

    #[test]
    fn parse_rye_scripts() {
        let content = "[tool.rye.scripts]\ntest = \"pytest\"\nserve = \"python -m http.server\"";
        let kf = parse(content).unwrap();
        assert_eq!(kf.tasks["test"].run, "pytest");
    }

    #[test]
    fn fallback_to_standard() {
        let content = "[project]\nname = \"my-app\"\nversion = \"1.0.0\"";
        let kf = parse(content).unwrap();
        assert_eq!(kf.name, "my-app");
        assert!(kf.tasks.contains_key("test"));
        assert!(kf.tasks.contains_key("lint"));
    }

    #[test]
    fn parse_pdm_cmd_format() {
        let content = "[tool.pdm.scripts]\nserve = {cmd = \"python -m http.server\"}";
        let kf = parse(content).unwrap();
        assert_eq!(kf.tasks["serve"].run, "python -m http.server");
    }
}
