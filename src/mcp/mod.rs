mod tools;

use rmcp::ServiceExt;
use rmcp::transport::io;
use tools::KyleMcp;

pub async fn serve() -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let server = KyleMcp::new(cwd);
    let transport = io::stdio();
    let service = server.serve(transport).await?;
    service.waiting().await?;
    Ok(())
}

pub fn print_config() -> anyhow::Result<()> {
    let kyle_path = std::env::current_exe()
        .unwrap_or_else(|_| std::path::PathBuf::from("kyle"))
        .to_string_lossy()
        .to_string();

    let config = serde_json::json!({
        "mcpServers": {
            "kyle": {
                "command": kyle_path,
                "args": ["mcp"]
            }
        }
    });

    println!("{}", serde_json::to_string_pretty(&config)?);
    Ok(())
}
