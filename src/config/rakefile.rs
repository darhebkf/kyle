use super::Error;
use super::kylefile::{Kylefile, Task};
use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;

// task :name or task :name => [:dep1, :dep2]
static TASK_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"^task\s+:(\w+)(?:\s+=>\s+\[([^\]]*)\])?"#).unwrap());

// desc "description"
static DESC_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"^desc\s+["'](.+?)["']"#).unwrap());

// sh "command" or system("command") or `command`
static SH_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"^\s+sh\s+["'](.+?)["']"#).unwrap());

static SYSTEM_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"^\s+system\(["'](.+?)["']\)"#).unwrap());

static BACKTICK_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\s+`(.+?)`").unwrap());

pub fn parse(content: &str) -> Result<Kylefile, Error> {
    let mut tasks: HashMap<String, Task> = HashMap::new();
    let mut pending_desc: Option<String> = None;

    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];

        if let Some(caps) = DESC_RE.captures(line) {
            pending_desc = Some(caps[1].to_string());
            i += 1;
            continue;
        }

        if let Some(caps) = TASK_RE.captures(line) {
            let name = caps[1].to_string();
            let deps: Vec<String> = caps
                .get(2)
                .map(|m| {
                    m.as_str()
                        .split(',')
                        .filter_map(|d| {
                            let d = d.trim().trim_start_matches(':');
                            if d.is_empty() {
                                None
                            } else {
                                Some(d.to_string())
                            }
                        })
                        .collect()
                })
                .unwrap_or_default();

            let mut commands: Vec<String> = Vec::new();
            i += 1;
            while i < lines.len() {
                let cmd_line = lines[i];
                if cmd_line.trim() == "end" {
                    i += 1;
                    break;
                }
                if cmd_line.trim().is_empty() || cmd_line.starts_with("task") {
                    break;
                }

                if let Some(caps) = SH_RE.captures(cmd_line) {
                    commands.push(caps[1].to_string());
                } else if let Some(caps) = SYSTEM_RE.captures(cmd_line) {
                    commands.push(caps[1].to_string());
                } else if let Some(caps) = BACKTICK_RE.captures(cmd_line) {
                    commands.push(caps[1].to_string());
                }
                i += 1;
            }

            tasks.insert(
                name,
                Task {
                    desc: pending_desc.take().unwrap_or_default(),
                    run: commands.join(" && "),
                    deps,
                },
            );
            continue;
        }

        if line.trim().is_empty() {
            pending_desc = None;
        }
        i += 1;
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
    fn parse_simple_task() {
        let content = r#"
task :build do
  sh "gcc -o main main.c"
end
"#;
        let kf = parse(content).unwrap();
        assert!(kf.tasks.contains_key("build"));
        assert_eq!(kf.tasks["build"].run, "gcc -o main main.c");
    }

    #[test]
    fn parse_task_with_desc() {
        let content = r#"
desc "Run tests"
task :test do
  sh "rspec"
end
"#;
        let kf = parse(content).unwrap();
        assert_eq!(kf.tasks["test"].desc, "Run tests");
    }

    #[test]
    fn parse_task_with_deps() {
        let content = r#"
task :test => [:build, :lint] do
  sh "rspec"
end
"#;
        let kf = parse(content).unwrap();
        assert_eq!(kf.tasks["test"].deps, vec!["build", "lint"]);
    }

    #[test]
    fn parse_system_command() {
        let content = r#"
task :clean do
  system("rm -rf build/")
end
"#;
        let kf = parse(content).unwrap();
        assert_eq!(kf.tasks["clean"].run, "rm -rf build/");
    }

    #[test]
    fn parse_multiple_commands() {
        let content = r#"
task :deploy do
  sh "make build"
  sh "rsync -a dist/ server:/app/"
end
"#;
        let kf = parse(content).unwrap();
        assert_eq!(
            kf.tasks["deploy"].run,
            "make build && rsync -a dist/ server:/app/"
        );
    }
}
