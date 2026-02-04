mod config;
mod init;
mod upgrade;

use crate::config::{self as kylefile_config, Source, load_from_dir};
use crate::namespace::discovery::{FileType, discover_namespaces};
use crate::namespace::{parse_task_ref, resolve_namespace};
use crate::output;
use crate::runner::Runner;
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::Path;

const VERSION: &str = concat!("v", env!("CARGO_PKG_VERSION"));

#[derive(Parser)]
#[command(name = "kyle", about = "kyle - task runner")]
#[command(version = VERSION)]
#[command(arg_required_else_help = false, disable_version_flag = true)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    /// Task to run
    #[arg(value_name = "TASK")]
    task: Option<String>,

    /// Arguments to pass to the task
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<String>,

    /// Print version
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    version: (),
}

#[derive(Subcommand)]
enum Command {
    /// Create a new Kylefile
    Init {
        /// Project name
        #[arg(value_name = "NAME")]
        name: Option<String>,

        /// Use YAML format
        #[arg(long)]
        yaml: bool,

        /// Use TOML format (default)
        #[arg(long)]
        toml: bool,
    },

    /// Configure kyle settings
    Config {
        #[command(subcommand)]
        action: Option<ConfigAction>,
    },

    /// Print version
    Version,

    /// Upgrade kyle to the latest version (duh)
    Upgrade,
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Show all settings
    List,

    /// Get a config value
    Get {
        /// Config key
        key: String,
    },

    /// Set a config value
    Set {
        /// Config key
        key: String,
        /// Config value
        value: String,
    },

    /// Show config file path
    Path,
}

pub fn run() -> Result<()> {
    upgrade::check_auto_upgrade();

    let cli = Cli::parse();

    match cli.command {
        Some(Command::Init { name, yaml, toml }) => {
            let format = if yaml {
                Some("yaml")
            } else if toml {
                Some("toml")
            } else {
                None
            };
            init::run(name.as_deref(), format)
        }
        Some(Command::Config { action }) => config::run(action),
        Some(Command::Version) => {
            println!("kyle {VERSION}");
            Ok(())
        }
        Some(Command::Upgrade) => upgrade::run(),
        None => run_tasks(cli.task.as_deref(), &cli.args),
    }
}

fn run_tasks(task: Option<&str>, args: &[String]) -> Result<()> {
    let cwd = std::env::current_dir().context("Failed to get current directory")?;

    match task {
        Some(task_input) => {
            let task_ref = parse_task_ref(task_input);

            if let Some(namespace) = &task_ref.namespace {
                // Explicit namespace - load from namespace directory (no warning)
                run_namespaced_task(&cwd, namespace, &task_ref.task_name, args)
            } else {
                // No namespace - load from current directory (may warn)
                run_local_task(&cwd, &task_ref.task_name, args)
            }
        }
        None => {
            // List tasks from current directory + discovered namespaces
            list_all_tasks(&cwd)
        }
    }
}

fn run_local_task(cwd: &Path, task_name: &str, args: &[String]) -> Result<()> {
    let (kf, source) = kylefile_config::load("")
        .context("No Kylefile found in current directory.\n\n  Run 'kyle init' to create one.")?;

    if source != Source::Kylefile {
        output::warn("no Kylefile found, run 'kyle init' to create one");
    }

    let mut runner = Runner::with_working_dir(kf, cwd.to_path_buf(), cwd.to_path_buf());
    runner.run(task_name, args)?;
    Ok(())
}

fn run_namespaced_task(
    root: &Path,
    namespace: &str,
    task_name: &str,
    args: &[String],
) -> Result<()> {
    let ns_dir = resolve_namespace(root, namespace);

    if !ns_dir.exists() {
        anyhow::bail!("Namespace directory not found: {}", ns_dir.display());
    }

    let (kf, _source) = load_from_dir(&ns_dir)
        .with_context(|| format!("Failed to load Kylefile from namespace '{namespace}'"))?;

    let mut runner = Runner::with_working_dir(kf, ns_dir, root.to_path_buf());
    runner.run(task_name, args)?;
    Ok(())
}

fn list_all_tasks(cwd: &Path) -> Result<()> {
    // Try to load local Kylefile
    let local_result = kylefile_config::load("");

    match local_result {
        Ok((kf, source)) => {
            if source != Source::Kylefile {
                output::warn("no Kylefile found, run 'kyle init' to create one");
            }

            println!("Available tasks:");
            let runner = Runner::new(kf.clone());
            runner.list_tasks();

            // Show namespaces from explicit includes
            if !kf.includes.is_empty() {
                println!("\nNamespaces (from includes):");
                for (alias, _path) in kf.includes.iter() {
                    println!("  {alias}:");
                }
            }

            // Discover additional namespaces
            let discovered = discover_namespaces(cwd);
            if !discovered.is_empty() {
                println!("\nDiscovered namespaces:");
                for ns in &discovered {
                    let type_indicator = match ns.file_type {
                        FileType::Kylefile => "",
                        FileType::Makefile => " (Makefile)",
                        FileType::Justfile => " (justfile)",
                    };
                    println!("  {}:{type_indicator}", ns.alias);
                }
            }
        }
        Err(_) => {
            // No local Kylefile, just show discovered namespaces
            let discovered = discover_namespaces(cwd);
            if discovered.is_empty() {
                anyhow::bail!(
                    "No Kylefile found in current directory.\n\n  Run 'kyle init' to create one."
                );
            }

            println!("Discovered namespaces:");
            for ns in &discovered {
                let type_indicator = match ns.file_type {
                    FileType::Kylefile => "",
                    FileType::Makefile => " (Makefile)",
                    FileType::Justfile => " (justfile)",
                };
                println!("  {}:{type_indicator}", ns.alias);
            }
        }
    }

    Ok(())
}
