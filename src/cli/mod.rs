mod completions;
mod config;
mod init;
mod upgrade;

use crate::config::{self as kylefile_config, load_from_dir};
use crate::namespace::discovery::{FileType, discover_namespaces};
use crate::namespace::{parse_task_ref, resolve_namespace};
use crate::output;
use crate::runner::Runner;
use crate::settings;
use crate::suggest;
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

    /// Print task names (legacy — used by older completion scripts)
    #[arg(long, hide = true)]
    summary: bool,

    /// Print full completion candidate set (reserved, tasks, namespaces, dispatcher subs)
    #[arg(long, hide = true)]
    completion_feed: bool,

    /// Print completion candidates for the arg position *after* a given task.
    /// For a dispatcher task, emits its subcommands (bare and qualified).
    /// For a namespace alias, emits the tasks inside that namespace.
    #[arg(long, hide = true, value_name = "TASK")]
    completion_for: Option<String>,

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
    // Fast path: if argv[1] is a task name (not a kyle reserved command or
    // flag), skip clap entirely so task args like `--help`, `--version`,
    // `help`, or any subcommand name pass through transparently without
    // needing `--` as a separator.
    let argv: Vec<String> = std::env::args().collect();

    // --dir escape: `kyle --dir <task> [args...]` forces task-route resolution
    // even when <task> would otherwise be a reserved command or namespace
    // alias that shadows one. Stays before the reserved-command check below.
    if argv.get(1).map(String::as_str) == Some("--dir") {
        let Some(task) = argv.get(2) else {
            anyhow::bail!("--dir requires a task name\n\n  Example: kyle --dir config:list");
        };
        let task_args: Vec<String> = match argv.get(3) {
            Some(s) if s == "--" => argv[4..].to_vec(),
            _ => argv[3..].to_vec(),
        };
        upgrade::check_auto_upgrade();
        return run_tasks(Some(task), &task_args);
    }

    if let Some(task) = argv.get(1)
        && is_user_task_invocation(task)
    {
        // Strip a single leading `--` for back-compat — historically required
        // to push args through clap; now purely optional.
        let task_args: Vec<String> = match argv.get(2) {
            Some(s) if s == "--" => argv[3..].to_vec(),
            _ => argv[2..].to_vec(),
        };
        upgrade::check_auto_upgrade();
        return run_tasks(Some(task), &task_args);
    }

    let cli = Cli::parse();

    if cli.summary {
        return print_summary();
    }

    if cli.completion_feed {
        return print_completion_feed();
    }

    if let Some(ref task) = cli.completion_for {
        return print_completion_for(task);
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

// Is this argument clearly a user task rather than a kyle-level flag or
// reserved command? If yes, clap is bypassed so task args pass through raw.
fn is_user_task_invocation(arg: &str) -> bool {
    if arg.starts_with('-') {
        return false;
    }
    if RESERVED_COMMANDS.contains(&arg) {
        return false;
    }
    true
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

fn print_completion_for(task: &str) -> Result<()> {
    let cwd = std::env::current_dir().context("Failed to get current directory")?;
    let local = kylefile_config::load("").ok().map(|(kf, _)| kf);

    let mut out = std::collections::BTreeSet::new();

    // If `task` is a local task with a dispatcher, emit its subcommands.
    if let Some(ref kf) = local
        && let Some(t) = kf.tasks.get(task)
        && let Some(d) = &t.dispatcher
    {
        for sub_name in d.subcommands.keys() {
            out.insert(sub_name.clone());
        }
    }

    // If `task` is a discovered namespace alias, emit its tasks (and nested
    // dispatcher subs as qualified forms).
    for ns in discover_namespaces(&cwd) {
        if ns.alias != task {
            continue;
        }
        if let Ok((kf, _)) = load_from_dir(&ns.path) {
            for (name, ns_task) in &kf.tasks {
                out.insert(name.clone());
                if let Some(d) = &ns_task.dispatcher {
                    for sub_name in d.subcommands.keys() {
                        out.insert(format!("{name}:{sub_name}"));
                        if !kf.tasks.contains_key(sub_name) {
                            out.insert(sub_name.clone());
                        }
                    }
                }
            }
        }
    }

    for cand in out {
        println!("{cand}");
    }
    Ok(())
}

fn print_completion_feed() -> Result<()> {
    let cwd = std::env::current_dir().context("Failed to get current directory")?;
    let local = kylefile_config::load("").ok().map(|(kf, _)| kf);
    let discovered = discover_namespaces(&cwd);
    let mut discovered_kylefiles = Vec::new();
    for ns in &discovered {
        if let Ok((kf, _)) = load_from_dir(&ns.path) {
            discovered_kylefiles.push((ns.alias.clone(), kf));
        }
    }
    for candidate in build_completion_feed(local.as_ref(), &discovered_kylefiles) {
        println!("{candidate}");
    }
    Ok(())
}

fn build_completion_feed(
    local: Option<&crate::config::Kylefile>,
    discovered: &[(String, crate::config::Kylefile)],
) -> Vec<String> {
    let mut out = std::collections::BTreeSet::new();

    for cmd in RESERVED_COMMANDS {
        out.insert((*cmd).to_string());
    }

    if let Some(kf) = local {
        for (name, task) in &kf.tasks {
            if RESERVED_COMMANDS.contains(&name.as_str()) {
                continue;
            }
            out.insert(name.clone());

            if let Some(d) = &task.dispatcher {
                for sub_name in d.subcommands.keys() {
                    out.insert(format!("{name}:{sub_name}"));
                    if !kf.tasks.contains_key(sub_name) {
                        out.insert(sub_name.clone());
                    }
                }
            }
        }
    }

    for (alias, kf) in discovered {
        out.insert(format!("{alias}:"));
        for (name, task) in &kf.tasks {
            out.insert(format!("{alias}:{name}"));

            if let Some(d) = &task.dispatcher {
                for sub_name in d.subcommands.keys() {
                    out.insert(format!("{alias}:{name}:{sub_name}"));
                    if !kf.tasks.contains_key(sub_name) {
                        out.insert(format!("{alias}:{sub_name}"));
                    }
                }
            }
        }
    }

    out.into_iter().collect()
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
        return run_qualified(&cwd, prefix, &task_ref.task_name, args, true);
    }

    run_flat(&cwd, task_input, args, true)
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

fn run_flat(cwd: &Path, task_input: &str, args: &[String], allow_correct: bool) -> Result<()> {
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
        if allow_correct {
            let candidates = build_suggestion_candidates(local.as_ref(), &discovered);
            match decide_autocorrect(task_input, &candidates) {
                AutocorrectAction::Run(corrected) => {
                    eprintln!("kyle: running '{corrected}' (corrected from '{task_input}')");
                    return run_flat(cwd, &corrected, args, false);
                }
                AutocorrectAction::Suggest(names) => {
                    anyhow::bail!(
                        "task not found: {task_input}\n\n  {}",
                        format_did_you_mean(&names)
                    );
                }
                AutocorrectAction::Plain => {}
            }
        }
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

fn run_qualified(
    cwd: &Path,
    left: &str,
    right: &str,
    args: &[String],
    allow_correct: bool,
) -> Result<()> {
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
            0 => {
                if allow_correct {
                    let candidates = candidates_within_namespace(&kf);
                    match decide_autocorrect(right, &candidates) {
                        AutocorrectAction::Run(corrected) => {
                            eprintln!(
                                "kyle: running '{left}:{corrected}' (corrected from '{left}:{right}')"
                            );
                            return run_qualified(cwd, left, &corrected, args, false);
                        }
                        AutocorrectAction::Suggest(names) => {
                            let qualified: Vec<String> =
                                names.iter().map(|n| format!("{left}:{n}")).collect();
                            anyhow::bail!(
                                "task not found: '{right}' in namespace '{left}'\n\n  {}",
                                format_did_you_mean(&qualified)
                            );
                        }
                        AutocorrectAction::Plain => {}
                    }
                }
                anyhow::bail!("task not found: '{right}' in namespace '{left}'")
            }
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

fn print_deduped_dispatcher_subs(kf: &crate::config::Kylefile) {
    let mut entries: Vec<(&String, &crate::dispatchers::Dispatcher)> = kf
        .tasks
        .iter()
        .filter_map(|(name, task)| {
            task.dispatcher
                .as_ref()
                .filter(|d| !d.subcommands.is_empty())
                .map(|d| (name, d))
        })
        .collect();
    entries.sort_by(|a, b| a.0.cmp(b.0));

    let mut seen_prefixes = std::collections::HashSet::new();
    for (name, d) in entries {
        if !seen_prefixes.insert(d.exec_prefix.as_str()) {
            continue;
        }
        print_dispatcher_subs(name, d);
    }
}

fn print_dispatcher_subs(task_name: &str, d: &crate::dispatchers::Dispatcher) {
    println!(
        "\n{task_name} ({} dispatcher — {} subcommands):",
        d.extension,
        d.subcommands.len()
    );
    let mut by_group: std::collections::BTreeMap<String, Vec<&crate::dispatchers::Subcommand>> =
        std::collections::BTreeMap::new();
    for sub in d.subcommands.values() {
        let key = sub.group.clone().unwrap_or_else(|| "other".to_string());
        by_group.entry(key).or_default().push(sub);
    }
    for (group, subs) in &by_group {
        println!("  [{group}]");
        for sub in subs {
            match &sub.desc {
                Some(desc) => println!("    {} — {}", sub.name, desc),
                None => println!("    {}", sub.name),
            }
        }
    }
}

fn warn_shadowing(
    kf: Option<&crate::config::Kylefile>,
    discovered: &[crate::namespace::discovery::DiscoveredNamespace],
) {
    // Namespace aliases that shadow reserved commands.
    for ns in discovered {
        if RESERVED_COMMANDS.contains(&ns.alias.as_str()) {
            output::warn(&format!(
                "namespace '{0}' shadows built-in '{0}' — use 'kyle --dir {0}:<task>' or the qualified form 'kyle {0}:<task>' to access it",
                ns.alias
            ));
        }
    }

    // Dispatcher subcommands that shadow reserved commands.
    if let Some(kf) = kf {
        let mut seen = std::collections::HashSet::new();
        for (task_name, task) in &kf.tasks {
            if let Some(d) = &task.dispatcher {
                for sub_name in d.subcommands.keys() {
                    if RESERVED_COMMANDS.contains(&sub_name.as_str())
                        && seen.insert(sub_name.clone())
                    {
                        output::warn(&format!(
                            "dispatcher '{task_name}' exposes subcommand '{sub_name}' that shadows built-in '{sub_name}' — use 'kyle {task_name}:{sub_name}' to invoke it"
                        ));
                    }
                }
            }
        }
    }
}

enum AutocorrectAction {
    Run(String),
    Suggest(Vec<String>),
    Plain,
}

fn decide_autocorrect(input: &str, candidates: &[String]) -> AutocorrectAction {
    let mode = settings::get().autocorrect;
    if mode == "off" {
        return AutocorrectAction::Plain;
    }

    let suggestions = suggest::suggest(input, candidates);
    if suggestions.is_empty() {
        return AutocorrectAction::Plain;
    }

    if mode == "autocorrect"
        && let Some(best) = suggest::best_unambiguous(input, candidates)
        && !RESERVED_COMMANDS.contains(&best.candidate.as_str())
    {
        return AutocorrectAction::Run(best.candidate);
    }

    let names: Vec<String> = suggestions
        .into_iter()
        .take(3)
        .map(|s| s.candidate)
        .collect();
    AutocorrectAction::Suggest(names)
}

fn format_did_you_mean(names: &[String]) -> String {
    match names {
        [one] => format!("Did you mean '{one}'? Run: kyle {one}"),
        many => {
            let list = many
                .iter()
                .map(|n| format!("    kyle {n}"))
                .collect::<Vec<_>>()
                .join("\n");
            format!("Did you mean one of:\n{list}")
        }
    }
}

fn build_suggestion_candidates(
    local: Option<&crate::config::Kylefile>,
    discovered: &[crate::namespace::discovery::DiscoveredNamespace],
) -> Vec<String> {
    let mut out = std::collections::BTreeSet::new();

    if let Some(kf) = local {
        for (name, task) in &kf.tasks {
            if RESERVED_COMMANDS.contains(&name.as_str()) {
                continue;
            }
            out.insert(name.clone());
            if let Some(d) = &task.dispatcher {
                for sub_name in d.subcommands.keys() {
                    if !RESERVED_COMMANDS.contains(&sub_name.as_str())
                        && !kf.tasks.contains_key(sub_name)
                    {
                        out.insert(sub_name.clone());
                    }
                    out.insert(format!("{name}:{sub_name}"));
                }
            }
        }
    }

    for ns in discovered {
        let Ok((kf, _)) = load_from_dir(&ns.path) else {
            continue;
        };
        for (name, task) in &kf.tasks {
            if !RESERVED_COMMANDS.contains(&name.as_str()) {
                out.insert(name.clone());
            }
            out.insert(format!("{}:{}", ns.alias, name));
            if let Some(d) = &task.dispatcher {
                for sub_name in d.subcommands.keys() {
                    if !RESERVED_COMMANDS.contains(&sub_name.as_str())
                        && !kf.tasks.contains_key(sub_name)
                    {
                        out.insert(sub_name.clone());
                    }
                    out.insert(format!("{}:{}:{}", ns.alias, name, sub_name));
                }
            }
        }
    }

    out.into_iter().collect()
}

fn candidates_within_namespace(kf: &crate::config::Kylefile) -> Vec<String> {
    let mut out = std::collections::BTreeSet::new();
    for (name, task) in &kf.tasks {
        out.insert(name.clone());
        if let Some(d) = &task.dispatcher {
            for sub_name in d.subcommands.keys() {
                if !kf.tasks.contains_key(sub_name) {
                    out.insert(sub_name.clone());
                }
                out.insert(format!("{name}:{sub_name}"));
            }
        }
    }
    out.into_iter().collect()
}

fn list_all_tasks(cwd: &Path) -> Result<()> {
    // Try to load local Kylefile
    let local_result = kylefile_config::load("");

    match local_result {
        Ok((kf, _source)) => {
            let discovered = discover_namespaces(cwd);
            warn_shadowing(Some(&kf), &discovered);

            println!("Available tasks:");
            let runner = Runner::new(kf.clone());
            runner.list_tasks();

            print_deduped_dispatcher_subs(&kf);

            // Show namespaces from explicit includes
            if !kf.includes.is_empty() {
                println!("\nNamespaces (from includes):");
                for (alias, _path) in kf.includes.iter() {
                    println!("  {alias}:");
                }
            }

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
            warn_shadowing(None, &discovered);

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Kylefile, Task};
    use crate::dispatchers::{Dispatcher, Subcommand};
    use std::collections::{BTreeMap, HashMap};

    fn task(run: &str) -> Task {
        Task {
            run: run.into(),
            ..Default::default()
        }
    }

    fn dispatcher_task(run: &str, exec_prefix: &str, subs: &[&str]) -> Task {
        let mut subcommands = BTreeMap::new();
        for s in subs {
            subcommands.insert((*s).to_string(), Subcommand::new(*s));
        }
        Task {
            run: run.into(),
            dispatcher: Some(Dispatcher {
                extension: "django".into(),
                exec_prefix: exec_prefix.into(),
                subcommands,
            }),
            ..Default::default()
        }
    }

    fn kf(tasks: &[(&str, Task)]) -> Kylefile {
        let mut map = HashMap::new();
        for (name, t) in tasks {
            map.insert((*name).to_string(), t.clone());
        }
        Kylefile {
            tasks: map,
            ..Default::default()
        }
    }

    #[test]
    fn feed_always_includes_reserved_commands() {
        let feed = build_completion_feed(None, &[]);
        for cmd in RESERVED_COMMANDS {
            assert!(feed.contains(&cmd.to_string()), "missing {cmd}");
        }
    }

    #[test]
    fn feed_skips_user_tasks_shadowing_reserved_names() {
        let local = kf(&[
            ("upgrade", task("echo nope")),
            ("build", task("cargo build")),
        ]);
        let feed = build_completion_feed(Some(&local), &[]);
        assert!(feed.contains(&"build".to_string()));
        // `upgrade` appears only as the reserved command — the user task is skipped.
        assert_eq!(
            feed.iter().filter(|s| *s == "upgrade").count(),
            1,
            "feed should dedupe reserved and user-task `upgrade`"
        );
    }

    #[test]
    fn feed_emits_dispatcher_subs_both_bare_and_qualified() {
        let local = kf(&[(
            "ccm-admin",
            dispatcher_task(
                "ccm.__main__:main",
                "src/manage.py",
                &["exportxml", "migrate"],
            ),
        )]);
        let feed = build_completion_feed(Some(&local), &[]);
        assert!(feed.contains(&"ccm-admin".to_string()));
        assert!(feed.contains(&"ccm-admin:exportxml".to_string()));
        assert!(feed.contains(&"ccm-admin:migrate".to_string()));
        assert!(feed.contains(&"exportxml".to_string()));
        assert!(feed.contains(&"migrate".to_string()));
    }

    #[test]
    fn feed_shadowed_dispatcher_sub_not_emitted_bare() {
        // Local task `format` shadows the dispatcher sub of the same name.
        let local = kf(&[
            ("format", task("black .")),
            (
                "dev",
                dispatcher_task(
                    "src/manage.py runserver",
                    "src/manage.py",
                    &["format", "seed"],
                ),
            ),
        ]);
        let feed = build_completion_feed(Some(&local), &[]);
        assert!(feed.contains(&"format".to_string()));
        assert!(feed.contains(&"dev:format".to_string()));
        // Bare `seed` stays (not shadowed); `format` appears once because the
        // user task shadows the dispatcher bare form.
        assert!(feed.contains(&"seed".to_string()));
        assert_eq!(
            feed.iter().filter(|s| *s == "format").count(),
            1,
            "format should appear exactly once (local task, not dispatcher sub)"
        );
    }

    #[test]
    fn feed_namespace_prefixes_and_tasks() {
        let backend = kf(&[("build", task("cargo build"))]);
        let feed = build_completion_feed(None, &[("backend".into(), backend)]);
        assert!(feed.contains(&"backend:".to_string()));
        assert!(feed.contains(&"backend:build".to_string()));
    }

    #[test]
    fn feed_namespace_dispatcher_subs() {
        let backend = kf(&[(
            "ccm-admin",
            dispatcher_task("ccm.__main__:main", "src/manage.py", &["exportxml"]),
        )]);
        let feed = build_completion_feed(None, &[("backend".into(), backend)]);
        assert!(feed.contains(&"backend:".to_string()));
        assert!(feed.contains(&"backend:ccm-admin".to_string()));
        assert!(feed.contains(&"backend:ccm-admin:exportxml".to_string()));
        assert!(feed.contains(&"backend:exportxml".to_string()));
    }

    #[test]
    fn candidates_skip_reserved_local_tasks() {
        let local = kf(&[
            ("build", task("cargo build")),
            ("upgrade", task("echo nope")), // shadows reserved — must be excluded as autocorrect target
        ]);
        let candidates = build_suggestion_candidates(Some(&local), &[]);
        assert!(candidates.contains(&"build".to_string()));
        assert!(!candidates.contains(&"upgrade".to_string()));
    }

    #[test]
    fn candidates_include_local_dispatcher_subs_bare_and_qualified() {
        let local = kf(&[(
            "ccm-admin",
            dispatcher_task(
                "ccm.__main__:main",
                "src/manage.py",
                &["exportxml", "migrate"],
            ),
        )]);
        let candidates = build_suggestion_candidates(Some(&local), &[]);
        assert!(candidates.contains(&"ccm-admin".to_string()));
        assert!(candidates.contains(&"ccm-admin:exportxml".to_string()));
        assert!(candidates.contains(&"exportxml".to_string()));
        assert!(candidates.contains(&"migrate".to_string()));
    }

    #[test]
    fn candidates_within_namespace_covers_tasks_and_subs() {
        let backend = kf(&[
            ("build", task("cargo build")),
            (
                "ccm-admin",
                dispatcher_task("ccm.__main__:main", "src/manage.py", &["exportxml"]),
            ),
        ]);
        let candidates = candidates_within_namespace(&backend);
        assert!(candidates.contains(&"build".to_string()));
        assert!(candidates.contains(&"ccm-admin".to_string()));
        assert!(candidates.contains(&"ccm-admin:exportxml".to_string()));
        assert!(candidates.contains(&"exportxml".to_string()));
    }

    #[test]
    fn did_you_mean_single() {
        let msg = format_did_you_mean(&["build".into()]);
        assert_eq!(msg, "Did you mean 'build'? Run: kyle build");
    }

    #[test]
    fn did_you_mean_multiple_lists_each() {
        let msg = format_did_you_mean(&["build".into(), "bind".into()]);
        assert!(msg.starts_with("Did you mean one of:"));
        assert!(msg.contains("kyle build"));
        assert!(msg.contains("kyle bind"));
    }

    #[test]
    fn feed_is_sorted_and_deduplicated() {
        let local = kf(&[
            ("build", task("cargo build")),
            ("build", task("duplicate")), // won't collide — HashMap dedupes
        ]);
        let feed = build_completion_feed(Some(&local), &[]);
        let mut sorted = feed.clone();
        sorted.sort();
        assert_eq!(feed, sorted, "feed must be sorted");
        let unique: std::collections::HashSet<_> = feed.iter().collect();
        assert_eq!(unique.len(), feed.len(), "feed must not contain duplicates");
    }
}
