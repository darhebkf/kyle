mod completions;
mod config;
mod init;
mod upgrade;

use crate::config::{self as kylefile_config, load_from_dir};
use crate::namespace::discovery::{FileType, discover_namespaces};
use crate::namespace::{parse_task_ref, resolve_namespace};
use crate::runner::Runner;
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};

const VERSION: &str = concat!("v", env!("CARGO_PKG_VERSION"));

pub const RESERVED_COMMANDS: &[&str] = &[
    "init",
    "config",
    "version",
    "upgrade",
    "mcp",
    "completions",
    "help",
];

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

    /// Print task names (used by completion scripts)
    #[arg(long, hide = true)]
    summary: bool,

    /// Internal: run a throttled auto-upgrade check in the background
    #[arg(long, hide = true)]
    upgrade_check: bool,
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

        /// Auto-detect tasks from project files (Cargo.toml, package.json, etc.)
        #[arg(long)]
        detect: bool,
    },

    /// Configure kyle settings
    Config {
        #[command(subcommand)]
        action: Option<ConfigAction>,
    },

    /// Print version
    Version,

    /// Upgrade kyle to the latest version (duh)
    Upgrade {
        /// Show recent auto-upgrade activity instead of upgrading
        #[arg(long)]
        status: bool,
    },

    /// MCP server for AI tools
    Mcp {
        /// Print MCP config JSON for AI clients
        #[arg(long)]
        config: bool,
    },

    /// Generate shell completions
    Completions {
        /// Shell type (bash, zsh, fish)
        shell: String,
    },
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
    let cli = Cli::parse();

    if cli.summary {
        return print_summary();
    }

    if cli.upgrade_check {
        upgrade::background_check();
        return Ok(());
    }

    upgrade::check_auto_upgrade();

    match cli.command {
        Some(Command::Init {
            name,
            yaml,
            toml,
            detect,
        }) => {
            let format = if yaml {
                Some("yaml")
            } else if toml {
                Some("toml")
            } else {
                None
            };
            if detect {
                init::run_detect(name.as_deref(), format)
            } else {
                init::run(name.as_deref(), format)
            }
        }
        Some(Command::Config { action }) => config::run(action),
        Some(Command::Version) => {
            println!("kyle {VERSION}");
            Ok(())
        }
        Some(Command::Upgrade { status }) => {
            if status {
                upgrade::print_status()
            } else {
                upgrade::run()
            }
        }
        Some(Command::Mcp { config }) => {
            if config {
                crate::mcp::print_config()
            } else {
                tokio::runtime::Runtime::new()?.block_on(crate::mcp::serve())
            }
        }
        Some(Command::Completions { shell }) => completions::run(&shell),
        None => run_tasks(cli.task.as_deref(), &cli.args),
    }
}

fn print_summary() -> Result<()> {
    if let Ok((kf, _)) = kylefile_config::load("") {
        for name in kf.tasks.keys() {
            if !RESERVED_COMMANDS.contains(&name.as_str()) {
                println!("{name}");
            }
        }
    }
    Ok(())
}

fn run_tasks(task: Option<&str>, args: &[String]) -> Result<()> {
    let cwd = std::env::current_dir().context("Failed to get current directory")?;

    let Some(task_input) = task else {
        return list_all_tasks(&cwd);
    };

    // Short-circuit: if the input contains `:` or `.` AND a local task has
    // that literal name (e.g. `test:e2e`, `build.debug`), run it directly.
    // This preserves literal-name semantics without hiding ambiguities in
    // the flat path (where the input has no separator).
    if (task_input.contains(':') || task_input.contains('.'))
        && let Ok((kf, _)) = kylefile_config::load("")
        && kf.tasks.contains_key(task_input)
    {
        let mut runner = Runner::with_working_dir(kf, cwd.to_path_buf(), cwd.to_path_buf());
        return runner.run(task_input, args).map_err(Into::into);
    }

    let task_ref = parse_task_ref(task_input);
    if let Some(prefix) = &task_ref.namespace {
        return run_qualified(&cwd, prefix, &task_ref.task_name, args);
    }

    run_flat(&cwd, task_input, args)
}

#[derive(Debug)]
enum FlatMatch {
    Local {
        task_name: String,
    },
    LocalSub {
        dispatcher_task: String,
        sub_name: String,
        exec_prefix: String,
    },
    Namespace {
        alias: String,
        path: PathBuf,
        task_name: String,
    },
    NamespaceSub {
        alias: String,
        path: PathBuf,
        dispatcher_task: String,
        sub_name: String,
        exec_prefix: String,
    },
}

impl FlatMatch {
    fn qualified_syntax(&self) -> String {
        match self {
            FlatMatch::Local { task_name } => task_name.clone(),
            FlatMatch::LocalSub {
                dispatcher_task,
                sub_name,
                ..
            } => format!("{dispatcher_task}:{sub_name}"),
            FlatMatch::Namespace {
                alias, task_name, ..
            } => format!("{alias}:{task_name}"),
            FlatMatch::NamespaceSub {
                alias,
                dispatcher_task,
                sub_name,
                ..
            } => format!("{alias}:{dispatcher_task}:{sub_name}"),
        }
    }
}

fn run_flat(cwd: &Path, task_input: &str, args: &[String]) -> Result<()> {
    let local = kylefile_config::load("").ok().map(|(kf, _)| kf);
    let discovered = discover_namespaces(cwd);

    // Level 1: local matches. Direct tasks shadow dispatcher subs within the
    // same file — user-authored pyproject.toml entries always win over
    // Django-discovered management commands with the same name.
    let mut local_matches: Vec<FlatMatch> = Vec::new();
    if let Some(ref kf) = local {
        if kf.tasks.contains_key(task_input) {
            local_matches.push(FlatMatch::Local {
                task_name: task_input.to_string(),
            });
        } else {
            for (name, task) in &kf.tasks {
                if let Some(d) = &task.dispatcher
                    && d.subcommands.contains_key(task_input)
                {
                    local_matches.push(FlatMatch::LocalSub {
                        dispatcher_task: name.clone(),
                        sub_name: task_input.to_string(),
                        exec_prefix: d.exec_prefix.clone(),
                    });
                }
            }
        }
    }

    if !local_matches.is_empty() {
        return resolve_level(cwd, task_input, dedupe_matches(local_matches), args);
    }

    // Level 2: discovered namespaces. Same shadowing rule per namespace.
    let mut ns_matches: Vec<FlatMatch> = Vec::new();
    for ns in &discovered {
        let Ok((kf, _)) = load_from_dir(&ns.path) else {
            continue;
        };
        if kf.tasks.contains_key(task_input) {
            ns_matches.push(FlatMatch::Namespace {
                alias: ns.alias.clone(),
                path: ns.path.clone(),
                task_name: task_input.to_string(),
            });
            continue;
        }
        for (name, task) in &kf.tasks {
            if let Some(d) = &task.dispatcher
                && d.subcommands.contains_key(task_input)
            {
                ns_matches.push(FlatMatch::NamespaceSub {
                    alias: ns.alias.clone(),
                    path: ns.path.clone(),
                    dispatcher_task: name.clone(),
                    sub_name: task_input.to_string(),
                    exec_prefix: d.exec_prefix.clone(),
                });
            }
        }
    }

    if ns_matches.is_empty() {
        return bail_not_found(cwd, task_input, local.is_some(), &discovered);
    }
    resolve_level(cwd, task_input, dedupe_matches(ns_matches), args)
}

// Collapse dispatcher-sub matches whose underlying exec_prefix is identical
// (within the same level — local-only, or per-namespace for namespace subs).
// Sorted alphabetically by dispatcher task name so the winner is stable.
fn dedupe_matches(mut matches: Vec<FlatMatch>) -> Vec<FlatMatch> {
    matches.sort_by(|a, b| match (a, b) {
        (
            FlatMatch::LocalSub {
                dispatcher_task: an,
                ..
            },
            FlatMatch::LocalSub {
                dispatcher_task: bn,
                ..
            },
        ) => an.cmp(bn),
        (
            FlatMatch::NamespaceSub {
                alias: aa,
                dispatcher_task: an,
                ..
            },
            FlatMatch::NamespaceSub {
                alias: ba,
                dispatcher_task: bn,
                ..
            },
        ) => aa.cmp(ba).then_with(|| an.cmp(bn)),
        _ => std::cmp::Ordering::Equal,
    });
    let mut seen: std::collections::HashSet<(String, String)> = std::collections::HashSet::new();
    matches.retain(|m| match m {
        FlatMatch::LocalSub { exec_prefix, .. } => {
            seen.insert((String::new(), exec_prefix.clone()))
        }
        FlatMatch::NamespaceSub {
            alias, exec_prefix, ..
        } => seen.insert((alias.clone(), exec_prefix.clone())),
        _ => true,
    });
    matches
}

fn resolve_level(
    cwd: &Path,
    task_input: &str,
    matches: Vec<FlatMatch>,
    args: &[String],
) -> Result<()> {
    match matches.len() {
        1 => execute_flat(cwd, matches.into_iter().next().unwrap(), args),
        _ => {
            let list: String = matches
                .iter()
                .map(|m| format!("  kyle {}", m.qualified_syntax()))
                .collect::<Vec<_>>()
                .join("\n");
            anyhow::bail!(
                "'{task_input}' is ambiguous — matches multiple tasks:\n{list}\n\n  Use the qualified form to pick one."
            );
        }
    }
}

fn execute_flat(cwd: &Path, m: FlatMatch, args: &[String]) -> Result<()> {
    match m {
        FlatMatch::Local { task_name } => run_local_task(cwd, &task_name, args),
        FlatMatch::LocalSub {
            dispatcher_task,
            sub_name,
            ..
        } => {
            let (kf, _) = kylefile_config::load("").context("No Kylefile found")?;
            run_dispatcher_sub(cwd, cwd, kf, &dispatcher_task, &sub_name, args, None)
        }
        FlatMatch::Namespace {
            alias,
            path,
            task_name,
        } => {
            let (kf, _) = load_from_dir(&path)
                .with_context(|| format!("Failed to load Kylefile from namespace '{alias}'"))?;
            let mut runner = Runner::with_working_dir(kf, path, cwd.to_path_buf());
            runner.run(&task_name, args).map_err(Into::into)
        }
        FlatMatch::NamespaceSub {
            alias,
            path,
            dispatcher_task,
            sub_name,
            ..
        } => {
            let (kf, _) = load_from_dir(&path)
                .with_context(|| format!("Failed to load Kylefile from namespace '{alias}'"))?;
            run_dispatcher_sub(
                cwd,
                &path,
                kf,
                &dispatcher_task,
                &sub_name,
                args,
                Some(&alias),
            )
        }
    }
}

fn run_qualified(cwd: &Path, left: &str, right: &str, args: &[String]) -> Result<()> {
    // Case 1: left is a real namespace directory — resolve right inside it
    let ns_dir = resolve_namespace(cwd, left);
    if ns_dir.exists() {
        let (kf, _) = load_from_dir(&ns_dir)
            .with_context(|| format!("Failed to load Kylefile from namespace '{left}'"))?;

        if kf.tasks.contains_key(right) {
            let mut runner = Runner::with_working_dir(kf, ns_dir, cwd.to_path_buf());
            return runner.run(right, args).map_err(Into::into);
        }

        let subs: Vec<String> = kf
            .tasks
            .iter()
            .filter_map(|(n, t)| {
                t.dispatcher
                    .as_ref()
                    .and_then(|d| d.subcommands.contains_key(right).then(|| n.clone()))
            })
            .collect();

        match subs.len() {
            0 => anyhow::bail!("task not found: '{right}' in namespace '{left}'"),
            1 => {
                let dispatcher_task = subs.into_iter().next().unwrap();
                return run_dispatcher_sub(
                    cwd,
                    &ns_dir,
                    kf,
                    &dispatcher_task,
                    right,
                    args,
                    Some(left),
                );
            }
            _ => {
                let list: String = subs
                    .iter()
                    .map(|t| format!("  kyle {left}:{t}:{right}"))
                    .collect::<Vec<_>>()
                    .join("\n");
                anyhow::bail!(
                    "'{right}' is ambiguous in namespace '{left}':\n{list}\n\n  Use the qualified form to pick one."
                );
            }
        }
    }

    // Case 2: left is a local dispatcher task name, right is one of its subcommands
    if let Ok((kf, _)) = kylefile_config::load("")
        && let Some(task) = kf.tasks.get(left)
        && let Some(d) = &task.dispatcher
        && d.subcommands.contains_key(right)
    {
        return run_dispatcher_sub(cwd, cwd, kf.clone(), left, right, args, None);
    }

    anyhow::bail!("Namespace directory not found: {}", ns_dir.display())
}

fn run_dispatcher_sub(
    cwd: &Path,
    working_dir: &Path,
    kf: crate::config::Kylefile,
    dispatcher_task: &str,
    sub_name: &str,
    args: &[String],
    namespace: Option<&str>,
) -> Result<()> {
    let task = kf
        .tasks
        .get(dispatcher_task)
        .with_context(|| format!("dispatcher task '{dispatcher_task}' vanished"))?;
    let d = task
        .dispatcher
        .as_ref()
        .with_context(|| format!("task '{dispatcher_task}' is not a dispatcher"))?;
    let label = match namespace {
        Some(ns) => format!("{ns}:{dispatcher_task}:{sub_name}"),
        None => format!("{dispatcher_task}:{sub_name}"),
    };
    let command = format!("{} {}", d.exec_prefix, sub_name);
    let runner = Runner::with_working_dir(kf, working_dir.to_path_buf(), cwd.to_path_buf());
    runner
        .run_command(&label, &command, args)
        .map_err(Into::into)
}

fn bail_not_found(
    _cwd: &Path,
    task_input: &str,
    has_local: bool,
    discovered: &[crate::namespace::discovery::DiscoveredNamespace],
) -> Result<()> {
    if !has_local && discovered.is_empty() {
        anyhow::bail!(
            "No Kylefile found in current directory.\n\n  Run 'kyle init' to create one."
        );
    }
    if discovered.is_empty() {
        anyhow::bail!("task not found: {task_input}");
    }
    let ns_list: String = discovered
        .iter()
        .map(|ns| {
            if ns.file_type == FileType::Kylefile {
                format!("  {}", ns.alias)
            } else {
                format!("  {} ({})", ns.alias, ns.file_type)
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    anyhow::bail!(
        "task not found: {task_input}\n\nDiscovered namespaces:\n{ns_list}\n\n  Use 'kyle <namespace>:{task_input}' to run a namespaced task."
    );
}

fn run_local_task(cwd: &Path, task_name: &str, args: &[String]) -> Result<()> {
    let (kf, _source) = kylefile_config::load("")
        .context("No Kylefile found in current directory.\n\n  Run 'kyle init' to create one.")?;

    let mut runner = Runner::with_working_dir(kf, cwd.to_path_buf(), cwd.to_path_buf());
    runner.run(task_name, args)?;
    Ok(())
}

fn list_all_tasks(cwd: &Path) -> Result<()> {
    // Try to load local Kylefile
    let local_result = kylefile_config::load("");

    match local_result {
        Ok((kf, _source)) => {
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
                    if ns.file_type == FileType::Kylefile {
                        println!("  {}:", ns.alias);
                    } else {
                        println!("  {}: ({})", ns.alias, ns.file_type);
                    }
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
                if ns.file_type == FileType::Kylefile {
                    println!("  {}:", ns.alias);
                } else {
                    println!("  {}: ({})", ns.alias, ns.file_type);
                }
            }
        }
    }

    Ok(())
}
