use std::path::PathBuf;
use std::process::Command as ProcessCommand;

use rmcp::ServerHandler;
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content, ErrorData as McpError, ServerCapabilities, ServerInfo};
use rmcp::{schemars, tool, tool_handler, tool_router};

use crate::config::load_from_dir;
use crate::namespace::discovery::discover_namespaces;

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct RunTaskParams {
    #[schemars(description = "Task name to run, e.g. 'build' or 'backend:build'")]
    name: String,
}

#[derive(Debug, Clone)]
pub struct KyleMcp {
    root: PathBuf,
    tool_router: ToolRouter<Self>,
}

impl KyleMcp {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            tool_router: Self::tool_router(),
        }
    }
}

#[tool_router]
impl KyleMcp {
    #[tool(
        description = "List all available tasks in the current project. Returns task names, descriptions, dependencies, and source file type. Also discovers tasks from subdirectory namespaces."
    )]
    async fn list_tasks(&self) -> Result<CallToolResult, McpError> {
        let mut output = String::new();

        match load_from_dir(&self.root) {
            Ok((kf, source)) => {
                output.push_str(&format!("Source: {source}\n\nTasks:\n"));
                let mut names: Vec<_> = kf.tasks.keys().collect();
                names.sort();
                for name in names {
                    let task = &kf.tasks[name];
                    output.push_str(&format!("  {name}"));
                    if !task.desc.is_empty() {
                        output.push_str(&format!(" — {}", task.desc));
                    }
                    if !task.deps.is_empty() {
                        output.push_str(&format!(" [deps: {}]", task.deps.join(", ")));
                    }
                    output.push('\n');
                }
            }
            Err(e) => {
                output.push_str(&format!("No task file found: {e}\n"));
            }
        }

        let discovered = discover_namespaces(&self.root);
        if !discovered.is_empty() {
            output.push_str("\nNamespaces:\n");
            for ns in &discovered {
                output.push_str(&format!("  {} ({})\n", ns.alias, ns.file_type));
                if let Ok((kf, _)) = load_from_dir(&ns.path) {
                    let mut names: Vec<_> = kf.tasks.keys().collect();
                    names.sort();
                    for name in names {
                        let task = &kf.tasks[name];
                        output.push_str(&format!("    {}:{name}", ns.alias));
                        if !task.desc.is_empty() {
                            output.push_str(&format!(" — {}", task.desc));
                        }
                        output.push('\n');
                    }
                }
            }
        }

        Ok(CallToolResult::success(vec![Content::text(output)]))
    }

    #[tool(
        description = "Run a task by name. Supports namespaced tasks like 'backend:build'. Returns the combined stdout and stderr output."
    )]
    async fn run_task(
        &self,
        Parameters(params): Parameters<RunTaskParams>,
    ) -> Result<CallToolResult, McpError> {
        let kyle_bin = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("kyle"));

        let result = ProcessCommand::new(&kyle_bin)
            .arg(&params.name)
            .current_dir(&self.root)
            .output();

        match result {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let stderr = String::from_utf8_lossy(&out.stderr);
                let mut text = String::new();
                if !stdout.is_empty() {
                    text.push_str(&stdout);
                }
                if !stderr.is_empty() {
                    if !text.is_empty() {
                        text.push('\n');
                    }
                    text.push_str(&stderr);
                }
                if text.is_empty() {
                    text = "Task completed with no output.".to_string();
                }

                if out.status.success() {
                    Ok(CallToolResult::success(vec![Content::text(text)]))
                } else {
                    let code = out.status.code().unwrap_or(-1);
                    Ok(CallToolResult::error(vec![Content::text(format!(
                        "Task '{}' failed (exit {code}):\n{text}",
                        params.name
                    ))]))
                }
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Failed to execute: {e}"
            ))])),
        }
    }
}

#[tool_handler]
impl ServerHandler for KyleMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Kyle is a universal task runner. Use list_tasks to discover all available tasks \
                 in the project (from Kylefiles, Makefiles, package.json, Cargo.toml, and more). \
                 Use run_task to execute any task by name."
                    .into(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}
