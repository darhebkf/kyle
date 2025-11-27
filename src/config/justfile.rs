use super::Error;
use super::kylefile::{Kylefile, Task};
use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;

static RECIPE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^([a-zA-Z_][a-zA-Z0-9_\-]*)(?:\s+[^:]+)?:\s*(.*)$").unwrap());

static COMMENT_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^#\s*(.*)$").unwrap());

static SETTING_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^set\s+").unwrap());

static ALIAS_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^alias\s+").unwrap());

pub fn parse(content: &str) -> Result<Kylefile, Error> {
    let mut tasks: HashMap<String, Task> = HashMap::new();
    let mut pending_comment: Option<String> = None;

    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];

        if SETTING_RE.is_match(line) || ALIAS_RE.is_match(line) {
            pending_comment = None;
            i += 1;
            continue;
        }

        if let Some(caps) = COMMENT_RE.captures(line) {
            let comment = caps[1].trim().to_string();
            if !comment.is_empty() && !comment.starts_with('!') {
                pending_comment = Some(comment);
            }
            i += 1;
            continue;
        }

        if let Some(caps) = RECIPE_RE.captures(line) {
            let recipe_name = caps[1].to_string();

            if recipe_name.starts_with('_') {
                pending_comment = None;
                i += 1;
                continue;
            }

            let deps: Vec<String> = caps[2]
                .split_whitespace()
                .filter(|d| !d.starts_with('(') && !d.contains('='))
                .map(|s| s.to_string())
                .collect();

            let mut commands: Vec<String> = Vec::new();
            i += 1;
            while i < lines.len() {
                let cmd_line = lines[i];
                if cmd_line.starts_with("    ") || cmd_line.starts_with('\t') {
                    let cmd = cmd_line.trim_start();
                    let cmd = cmd.strip_prefix('@').unwrap_or(cmd);
                    if !cmd.is_empty() {
                        commands.push(cmd.to_string());
                    }
                    i += 1;
                } else if cmd_line.trim().is_empty() {
                    i += 1;
                } else {
                    break;
                }
            }

            let task = Task {
                desc: pending_comment.take().unwrap_or_default(),
                run: commands.join(" && "),
                deps,
            };

            tasks.insert(recipe_name, task);
            continue;
        }

        if line.trim().is_empty() {
            pending_comment = None;
        }
        i += 1;
    }

    Ok(Kylefile {
        name: String::new(),
        includes: Default::default(),
        tasks,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_recipe() {
        let content = "build:\n    cargo build\n";
        let kf = parse(content).unwrap();
        assert!(kf.tasks.contains_key("build"));
        assert_eq!(kf.tasks["build"].run, "cargo build");
    }

    #[test]
    fn parse_recipe_with_deps() {
        let content = "test: build\n    cargo test\n";
        let kf = parse(content).unwrap();
        assert_eq!(kf.tasks["test"].deps, vec!["build"]);
    }

    #[test]
    fn parse_comment_as_description() {
        let content = "# Build the project\nbuild:\n    cargo build\n";
        let kf = parse(content).unwrap();
        assert_eq!(kf.tasks["build"].desc, "Build the project");
    }

    #[test]
    fn skip_private_recipes() {
        let content = "_helper:\n    echo helper\n\nbuild:\n    echo build\n";
        let kf = parse(content).unwrap();
        assert!(!kf.tasks.contains_key("_helper"));
        assert!(kf.tasks.contains_key("build"));
    }

    #[test]
    fn handle_quiet_prefix() {
        let content = "build:\n    @echo building\n";
        let kf = parse(content).unwrap();
        assert_eq!(kf.tasks["build"].run, "echo building");
    }

    #[test]
    fn multi_line_commands() {
        let content = "build:\n    echo step1\n    echo step2\n";
        let kf = parse(content).unwrap();
        assert_eq!(kf.tasks["build"].run, "echo step1 && echo step2");
    }

    #[test]
    fn skip_settings() {
        let content = "set shell := [\"bash\", \"-c\"]\n\nbuild:\n    echo build\n";
        let kf = parse(content).unwrap();
        assert!(kf.tasks.contains_key("build"));
        assert!(!kf.tasks.contains_key("set"));
    }

    #[test]
    fn skip_aliases() {
        let content = "alias b := build\n\nbuild:\n    echo build\n";
        let kf = parse(content).unwrap();
        assert!(kf.tasks.contains_key("build"));
        assert!(!kf.tasks.contains_key("alias"));
    }

    #[test]
    fn multiple_recipes() {
        let content = "# Build\nbuild:\n    echo build\n\n# Test\ntest: build\n    echo test\n";
        let kf = parse(content).unwrap();
        assert_eq!(kf.tasks.len(), 2);
        assert_eq!(kf.tasks["build"].desc, "Build");
        assert_eq!(kf.tasks["test"].desc, "Test");
        assert_eq!(kf.tasks["test"].deps, vec!["build"]);
    }
}
