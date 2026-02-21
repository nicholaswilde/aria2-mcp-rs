use anyhow::Result;
use mcp_sdk_rs::server::Server;
use mcp_sdk_rs::transport::stdio::StdioTransport;
use std::sync::Arc;

use crate::aria2::Aria2Client;
use crate::server::handler::McpHandler;
use crate::tools::ToolRegistry;

pub async fn run_server(registry: Arc<ToolRegistry>, client: Arc<Aria2Client>) -> Result<()> {
    let (transport, _sender) = StdioTransport::new();
    let handler = Arc::new(McpHandler::new(registry, client));
    let server = Server::new(Arc::new(transport), handler);

    server
        .start()
        .await
        .map_err(|e| anyhow::anyhow!("Server error: {:?}", e))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    // We can't easily test run_server as it's blocking
}
