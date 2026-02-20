use anyhow::Result;
use std::sync::Arc;
use mcp_sdk_rs::server::Server;
use mcp_sdk_rs::transport::stdio::StdioTransport;

use crate::aria2::Aria2Client;
use crate::tools::ToolRegistry;
use crate::server::handler::McpHandler;

pub async fn run_server(registry: Arc<ToolRegistry>, client: Arc<Aria2Client>) -> Result<()> {
    let (transport, _sender) = StdioTransport::new();
    let handler = Arc::new(McpHandler::new(registry, client));
    let server = Server::new(Arc::new(transport), handler);

    server.start().await.map_err(|e| anyhow::anyhow!("Server error: {:?}", e))?;
    Ok(())
}
