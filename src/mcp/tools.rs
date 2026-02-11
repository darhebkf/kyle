use std::path::PathBuf;

use rmcp::ServerHandler;
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::model::{ServerCapabilities, ServerInfo};
use rmcp::{schemars, tool, tool_router};

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
    // TODO
}

impl ServerHandler for KyleMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Kyle is a task runner that reads Kylefiles, Makefiles, and justfiles. \
                 Use list_tasks to discover available tasks, then run_task to execute them."
                    .into(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}
