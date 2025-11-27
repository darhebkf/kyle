use crate::settings;
use anyhow::Result;
use std::env;
use std::fs;
use std::io::{self, BufRead, Write};

const KYLEFILE: &str = "Kylefile";
const DEFAULT_PROJECT_NAME: &str = "project";

struct TaskDef {
    name: String,
    desc: String,
    run: String,
}

pub fn run(name: Option<&str>, format: Option<&str>) -> Result<()> {
    let name = match name {
        Some(n) => n.to_string(),
        None => prompt_name()?,
    };

    let format = format
        .map(String::from)
        .unwrap_or_else(|| settings::get().default_format);

    let mut tasks = Vec::new();
    if prompt_yn("Add a task?", true)? {
        loop {
            tasks.push(prompt_task()?);
            if !prompt_yn("Add another task?", false)? {
                break;
            }
        }
    }

    let content = generate_kylefile(&format, &name, &tasks);
    fs::write(KYLEFILE, &content)?;

    println!("\n  Created {KYLEFILE}\n");
    Ok(())
}

fn prompt_name() -> Result<String> {
    let default_name = env::current_dir()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .unwrap_or_else(|| DEFAULT_PROJECT_NAME.to_string());

    print!("  Project name [{default_name}]: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().lock().read_line(&mut input)?;
    let input = input.trim();

    if input.is_empty() {
        Ok(default_name)
    } else {
        Ok(input.to_string())
    }
}

fn prompt_yn(question: &str, default_yes: bool) -> Result<bool> {
    let hint = if default_yes { "Y/n" } else { "y/N" };

    print!("  {question} [{hint}]: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().lock().read_line(&mut input)?;
    let input = input.trim();

    if input.is_empty() {
        Ok(default_yes)
    } else {
        Ok(input.eq_ignore_ascii_case("y") || input.eq_ignore_ascii_case("yes"))
    }
}

fn prompt_task() -> Result<TaskDef> {
    let mut name = String::new();
    let mut desc = String::new();
    let mut run = String::new();

    print!("  Task name: ");
    io::stdout().flush()?;
    io::stdin().lock().read_line(&mut name)?;

    print!("  Description (optional): ");
    io::stdout().flush()?;
    io::stdin().lock().read_line(&mut desc)?;

    print!("  Command: ");
    io::stdout().flush()?;
    io::stdin().lock().read_line(&mut run)?;

    println!();

    Ok(TaskDef {
        name: name.trim().to_string(),
        desc: desc.trim().to_string(),
        run: run.trim().to_string(),
    })
}

fn generate_kylefile(format: &str, name: &str, tasks: &[TaskDef]) -> String {
    if format == "yaml" {
        generate_yaml(name, tasks)
    } else {
        generate_toml(name, tasks)
    }
}

fn generate_yaml(name: &str, tasks: &[TaskDef]) -> String {
    let mut out = String::new();

    // Format header tells kyle how to parse extensionless Kylefile
    out.push_str("# kyle: yaml\n");
    out.push_str(&format!("version: \"{}\"\n", env!("CARGO_PKG_VERSION")));
    out.push_str(&format!("name: {name}\n\ntasks:\n"));

    if tasks.is_empty() {
        out.push_str("  # example:\n");
        out.push_str("  #   desc: An example task\n");
        out.push_str("  #   run: echo hello\n");
    }

    for t in tasks {
        out.push_str(&format!("  {}:\n", t.name));
        if !t.desc.is_empty() {
            out.push_str(&format!("    desc: {}\n", t.desc));
        }
        out.push_str(&format!("    run: {}\n", t.run));
    }

    out
}

fn generate_toml(name: &str, tasks: &[TaskDef]) -> String {
    let mut out = String::new();

    // Format header tells kyle how to parse extensionless Kylefile
    out.push_str("# kyle: toml\n");
    out.push_str(&format!("version = \"{}\"\n", env!("CARGO_PKG_VERSION")));
    out.push_str(&format!("name = \"{name}\"\n"));

    if tasks.is_empty() {
        out.push_str("\n# [tasks.example]\n");
        out.push_str("# desc = \"An example task\"\n");
        out.push_str("# run = \"echo hello\"\n");
    }

    for t in tasks {
        out.push_str(&format!("\n[tasks.{}]\n", t.name));
        if !t.desc.is_empty() {
            out.push_str(&format!("desc = \"{}\"\n", t.desc));
        }
        out.push_str(&format!("run = \"{}\"\n", t.run));
    }

    out
}
