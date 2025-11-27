use super::Error;
use super::kylefile::{Kylefile, Task};
use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;

static TARGET_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^([a-zA-Z_][a-zA-Z0-9_\-\.]*)[ \t]*:([^=].*)?$").unwrap());

static PHONY_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\.PHONY\s*:\s*(.+)$").unwrap());

static COMMENT_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^#\s*(.*)$").unwrap());

pub fn parse(content: &str) -> Result<Kylefile, Error> {
    let mut tasks: HashMap<String, Task> = HashMap::new();
    let mut pending_comment: Option<String> = None;

    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];

        if PHONY_RE.is_match(line) {
            pending_comment = None;
            i += 1;
            continue;
        }

        if let Some(caps) = COMMENT_RE.captures(line) {
            let comment = caps[1].trim().to_string();
            if !comment.is_empty() {
                pending_comment = Some(comment);
            }
            i += 1;
            continue;
        }

        if let Some(caps) = TARGET_RE.captures(line) {
            let target_name = caps[1].to_string();

            if target_name.starts_with('.') || target_name.contains('%') {
                pending_comment = None;
                i += 1;
                continue;
            }

            let deps_str = caps.get(2).map(|m| m.as_str()).unwrap_or("");
            let deps: Vec<String> = deps_str
                .split_whitespace()
                .filter(|d| !d.starts_with('$') && !d.contains('%'))
                .map(|s| s.to_string())
                .collect();

            let mut commands: Vec<String> = Vec::new();
            i += 1;
            while i < lines.len() {
                let cmd_line = lines[i];
                if cmd_line.starts_with('\t') {
                    let cmd = cmd_line.trim_start_matches('\t');
                    let cmd = cmd.strip_prefix('@').unwrap_or(cmd);
                    let cmd = cmd.strip_prefix('-').unwrap_or(cmd);
                    if !cmd.trim().is_empty() {
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

            tasks.insert(target_name, task);
            continue;
        }

        if line.trim().is_empty() || !line.starts_with('\t') {
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
    fn parse_simple_target() {
        let content = "build:\n\tgcc -o main main.c\n";
        let kf = parse(content).unwrap();
        assert!(kf.tasks.contains_key("build"));
        assert_eq!(kf.tasks["build"].run, "gcc -o main main.c");
    }

    #[test]
    fn parse_target_with_deps() {
        let content = "test: build\n\t./run_tests.sh\n";
        let kf = parse(content).unwrap();
        assert_eq!(kf.tasks["test"].deps, vec!["build"]);
    }

    #[test]
    fn parse_comment_as_description() {
        let content = "# Build the project\nbuild:\n\tmake all\n";
        let kf = parse(content).unwrap();
        assert_eq!(kf.tasks["build"].desc, "Build the project");
    }

    #[test]
    fn parse_phony_targets() {
        let content = ".PHONY: build test\n\nbuild:\n\techo build\n";
        let kf = parse(content).unwrap();
        assert!(kf.tasks.contains_key("build"));
    }

    #[test]
    fn skip_pattern_rules() {
        let content = "%.o: %.c\n\tgcc -c $<\n\nbuild:\n\techo build\n";
        let kf = parse(content).unwrap();
        assert!(!kf.tasks.contains_key("%.o"));
        assert!(kf.tasks.contains_key("build"));
    }

    #[test]
    fn multi_line_commands() {
        let content = "build:\n\techo step1\n\techo step2\n";
        let kf = parse(content).unwrap();
        assert_eq!(kf.tasks["build"].run, "echo step1 && echo step2");
    }

    #[test]
    fn skip_variable_assignments() {
        let content = "CC := gcc\n\nbuild:\n\t$(CC) main.c\n";
        let kf = parse(content).unwrap();
        assert!(kf.tasks.contains_key("build"));
        assert!(!kf.tasks.contains_key("CC"));
    }

    #[test]
    fn handle_silent_prefix() {
        let content = "build:\n\t@echo building\n";
        let kf = parse(content).unwrap();
        assert_eq!(kf.tasks["build"].run, "echo building");
    }

    #[test]
    fn handle_ignore_errors_prefix() {
        let content = "clean:\n\t-rm -rf build/\n";
        let kf = parse(content).unwrap();
        assert_eq!(kf.tasks["clean"].run, "rm -rf build/");
    }

    #[test]
    fn multiple_targets() {
        let content = "# Build\nbuild:\n\techo build\n\n# Test\ntest: build\n\techo test\n";
        let kf = parse(content).unwrap();
        assert_eq!(kf.tasks.len(), 2);
        assert_eq!(kf.tasks["build"].desc, "Build");
        assert_eq!(kf.tasks["test"].desc, "Test");
        assert_eq!(kf.tasks["test"].deps, vec!["build"]);
    }
}
