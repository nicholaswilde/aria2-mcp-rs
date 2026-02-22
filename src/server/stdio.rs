use anyhow::Result;
use mcp_sdk_rs::server::Server;
use mcp_sdk_rs::transport::stdio::StdioTransport;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::aria2::notifications::Aria2Notification;
use crate::aria2::Aria2Client;
use crate::prompts::PromptRegistry;
use crate::resources::ResourceRegistry;
use crate::server::handler::McpHandler;
use crate::tools::ToolRegistry;

pub async fn run_server(
    registry: Arc<RwLock<ToolRegistry>>,
    resource_registry: Arc<RwLock<ResourceRegistry>>,
    prompt_registry: Arc<RwLock<PromptRegistry>>,
    clients: Vec<Arc<Aria2Client>>,
    mut notification_rx: tokio::sync::mpsc::Receiver<Aria2Notification>,
) -> Result<()> {
    let (transport, sender) = StdioTransport::new();
    let handler = Arc::new(McpHandler::new(
        registry,
        resource_registry,
        prompt_registry,
        clients,
    ));
    let server = Server::new(Arc::new(transport), handler);

    tokio::spawn(async move {
        while let Some(notification) = notification_rx.recv().await {
            let mcp_notification = notification.to_mcp_notification();
            // Try to deserialize into the transport's Message type
            match serde_json::from_value::<mcp_sdk_rs::transport::Message>(mcp_notification) {
                Ok(msg) => {
                    if let Err(e) = sender.send(Ok(msg)) {
                        log::error!("Failed to send notification to client: {}", e);
                    }
                }
                Err(e) => {
                    log::error!("Failed to serialize notification for transport: {}", e);
                }
            }
        }
    });

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
