use super::Error;
use super::kylefile::{Kylefile, Task};
use std::collections::HashMap;

pub fn parse(content: &str) -> Result<Kylefile, Error> {
    let doc: toml::Value = toml::from_str(content)?;
    let mut tasks = HashMap::new();
    let mut found_shortcut_source = false;

    // PEP-621 [project.scripts] — console script entry points installed on
    // PATH by build backends (pdm, hatch, setuptools, ...). Orthogonal to
    // pdm/hatch/rye task shortcuts, so always additive.
    if let Some(scripts) = doc
        .get("project")
        .and_then(|p| p.get("scripts"))
        .and_then(|s| s.as_table())
    {
        for (name, val) in scripts {
            if let Some(cmd) = val.as_str() {
                tasks.insert(
                    name.clone(),
                    Task {
                        run: cmd.to_string(),
                        ..Default::default()
                    },
                );
            }
        }
    }

    // Task shortcut sources — pdm > hatch > rye, first found wins.
    if let Some(scripts) = doc
        .get("tool")
        .and_then(|t| t.get("pdm"))
        .and_then(|p| p.get("scripts"))
        .and_then(|s| s.as_table())
    {
        found_shortcut_source = true;
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

    if !found_shortcut_source
        && let Some(scripts) = doc
            .get("tool")
            .and_then(|t| t.get("hatch"))
            .and_then(|h| h.get("envs"))
            .and_then(|e| e.get("default"))
            .and_then(|d| d.get("scripts"))
            .and_then(|s| s.as_table())
    {
        found_shortcut_source = true;
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

    if !found_shortcut_source
        && let Some(scripts) = doc
            .get("tool")
            .and_then(|t| t.get("rye"))
            .and_then(|r| r.get("scripts"))
            .and_then(|s| s.as_table())
    {
        found_shortcut_source = true;
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

    // Fallback to standard python tasks when no task shortcut source was
    // found. [project.scripts] alone doesn't count — it's console entries,
    // not task commands. Merge (don't replace) so project.scripts entries
    // survive alongside the generated test/lint/format tasks.
    if !found_shortcut_source {
        for (name, task) in standard_python_tasks() {
            tasks.entry(name).or_insert(task);
        }
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
        toml::Value::Table(t) => match t.get("cmd") {
            Some(toml::Value::String(s)) => Some(s.clone()),
            Some(toml::Value::Array(arr)) => {
                let parts: Vec<String> = arr
                    .iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect();
                if parts.is_empty() {
                    None
                } else {
                    Some(parts.join(" "))
                }
            }
            _ => None,
        },
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

    #[test]
    fn parse_pdm_cmd_array_format() {
        let content = r#"[tool.pdm.scripts]
format = {cmd = ["bash", "-c", "isort src ; black src"]}
"#;
        let kf = parse(content).unwrap();
        assert_eq!(kf.tasks["format"].run, "bash -c isort src ; black src");
    }

    #[test]
    fn parse_project_scripts_alongside_pdm() {
        let content = r#"
[project]
name = "demo"

[project.scripts]
"ccm-admin" = "ccm.__main__:main"

[tool.pdm.scripts]
dev = "src/manage.py runserver"
test = "pytest"
"#;
        let kf = parse(content).unwrap();
        assert_eq!(kf.tasks["ccm-admin"].run, "ccm.__main__:main");
        assert_eq!(kf.tasks["dev"].run, "src/manage.py runserver");
        assert_eq!(kf.tasks["test"].run, "pytest");
    }

    #[test]
    fn project_scripts_alone_does_not_skip_standard_fallback() {
        // A pyproject with only [project.scripts] and no task shortcuts
        // should still expose the standard python task set (test/lint/...).
        let content = r#"
[project]
name = "demo"

[project.scripts]
"my-cli" = "my_pkg:main"
"#;
        let kf = parse(content).unwrap();
        assert_eq!(kf.tasks["my-cli"].run, "my_pkg:main");
        assert!(kf.tasks.contains_key("test"));
        assert!(kf.tasks.contains_key("lint"));
    }
}
