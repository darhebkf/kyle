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
