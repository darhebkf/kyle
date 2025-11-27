use crate::settings;
use anyhow::Result;
use std::io::{self, BufRead, Write};

pub fn run(action: Option<super::ConfigAction>) -> Result<()> {
    use super::ConfigAction;
    match action {
        Some(ConfigAction::Get { key }) => config_get(&key),
        Some(ConfigAction::Set { key, value }) => config_set(&key, &value),
        Some(ConfigAction::List) => config_list(),
        Some(ConfigAction::Path) => {
            println!("{}", settings::path().display());
            Ok(())
        }
        None => run_interactive(),
    }
}

fn config_get(key: &str) -> Result<()> {
    let val = settings::get_value(key)?;
    println!("{val}");
    Ok(())
}

fn config_set(key: &str, value: &str) -> Result<()> {
    settings::set(key, value)?;
    println!("  {key} = {value}");
    Ok(())
}

fn config_list() -> Result<()> {
    for (k, v) in settings::list() {
        println!("  {k} = {v}");
    }
    Ok(())
}

fn run_interactive() -> Result<()> {
    println!("\n  Kyle Configuration");
    println!("  ──────────────────");
    println!("  Config file: {}\n", settings::path().display());

    let s = settings::get();

    print!("  Default format [{}]: ", s.default_format);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().lock().read_line(&mut input)?;
    let input = input.trim();

    if !input.is_empty() && input != s.default_format {
        settings::set("default_format", input)?;
        println!("  ✓ Updated");
    }

    println!();
    Ok(())
}
