use crate::config::{Kylefile, Source, load_from_dir};
use crate::namespace::{parse_task_ref, resolve_namespace};
use std::collections::HashSet;
use std::io;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use thiserror::Error;

const SHELL: &str = "sh";
const SHELL_FLAG: &str = "-c";

pub struct Runner {
    kylefile: Kylefile,
    working_dir: PathBuf,
    root_dir: PathBuf,
    executed: HashSet<String>,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("task not found: {0}")]
    TaskNotFound(String),

    #[error("namespace not found: {0}")]
    NamespaceNotFound(String),

    #[error("failed to load namespace '{namespace}': {source}")]
    NamespaceLoadFailed {
        namespace: String,
        #[source]
        source: crate::config::Error,
    },

    #[error("dependency '{dep}' failed: {source}")]
    DependencyFailed {
        dep: String,
        #[source]
        source: Box<Error>,
    },

    #[error("task '{task}' failed: {source}")]
    ExecutionFailed {
        task: String,
        #[source]
        source: io::Error,
    },
}

impl Runner {
    pub fn new(kylefile: Kylefile) -> Self {
        let cwd = std::env::current_dir().unwrap_or_default();
        Self {
            kylefile,
            working_dir: cwd.clone(),
            root_dir: cwd,
            executed: HashSet::new(),
        }
    }

    pub fn with_working_dir(kylefile: Kylefile, working_dir: PathBuf, root_dir: PathBuf) -> Self {
        Self {
            kylefile,
            working_dir,
            root_dir,
            executed: HashSet::new(),
        }
    }

    pub fn run(&mut self, task_name: &str, args: &[String]) -> Result<(), Error> {
        let task = self
            .kylefile
            .tasks
            .get(task_name)
            .ok_or_else(|| Error::TaskNotFound(task_name.into()))?
            .clone();

        // Run dependencies first (without extra args, args only apply to main task)
        for dep in &task.deps {
            let dep_ref = parse_task_ref(dep);

            // Create a unique key for executed tracking
            let executed_key = if dep_ref.is_namespaced() {
                dep.clone()
            } else {
                task_name.to_string()
            };

            if self.executed.contains(&executed_key) {
                continue;
            }

            if dep_ref.is_namespaced() {
                // Cross-namespace dependency
                self.run_namespaced(
                    &dep_ref
                        .namespace
                        .expect("invariant: is_namespaced() guarantees namespace is Some"),
                    &dep_ref.task_name,
                )
                .map_err(|e| Error::DependencyFailed {
                    dep: dep.clone(),
                    source: Box::new(e),
                })?;
            } else {
                // Local dependency
                self.run(dep, &[]).map_err(|e| Error::DependencyFailed {
                    dep: dep.clone(),
                    source: Box::new(e),
                })?;
            }
        }

        if self.executed.contains(task_name) {
            return Ok(());
        }

        println!("→ {task_name}");

        // Append extra args to command if provided
        let cmd = if args.is_empty() {
            task.run.clone()
        } else {
            format!("{} {}", task.run, args.join(" "))
        };

        let status = Command::new(SHELL)
            .arg(SHELL_FLAG)
            .arg(&cmd)
            .current_dir(&self.working_dir)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .map_err(|e| Error::ExecutionFailed {
                task: task_name.into(),
                source: e,
            })?;

        if !status.success() {
            return Err(Error::ExecutionFailed {
                task: task_name.into(),
                source: io::Error::other(format!("exit code: {}", status.code().unwrap_or(-1))),
            });
        }

        self.executed.insert(task_name.into());
        Ok(())
    }

    /// Run a task in a different namespace
    fn run_namespaced(&mut self, namespace: &str, task_name: &str) -> Result<(), Error> {
        let ns_key = format!("{namespace}:{task_name}");
        if self.executed.contains(&ns_key) {
            return Ok(());
        }

        let ns_dir = resolve_namespace(&self.root_dir, namespace);

        if !ns_dir.exists() {
            return Err(Error::NamespaceNotFound(namespace.into()));
        }

        let (kf, _source) = load_from_dir(&ns_dir).map_err(|e| Error::NamespaceLoadFailed {
            namespace: namespace.into(),
            source: e,
        })?;

        let mut ns_runner = Runner::with_working_dir(kf, ns_dir, self.root_dir.clone());

        println!("→ [{namespace}]");
        ns_runner.run(task_name, &[])?;

        self.executed.insert(ns_key);

        Ok(())
    }

    pub fn kylefile(&self) -> &Kylefile {
        &self.kylefile
    }

    pub fn source(&self) -> Option<Source> {
        None // Will be set by CLI when needed
    }

    pub fn list_tasks(&self) {
        for (name, task) in &self.kylefile.tasks {
            if task.desc.is_empty() {
                println!("  {name}");
            } else {
                println!("  {name} - {}", task.desc);
            }
        }
    }
}
