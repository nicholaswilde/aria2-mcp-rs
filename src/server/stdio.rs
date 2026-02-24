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

pub fn create_server(
    registry: Arc<RwLock<ToolRegistry>>,
    resource_registry: Arc<RwLock<ResourceRegistry>>,
    prompt_registry: Arc<RwLock<PromptRegistry>>,
    clients: Vec<Arc<Aria2Client>>,
) -> Server {
    let (transport, _sender) = StdioTransport::new();
    let handler = Arc::new(McpHandler::new(
        registry,
        resource_registry,
        prompt_registry,
        clients,
    ));
    Server::new(Arc::new(transport), handler)
}

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
    use super::*;
    use crate::aria2::notifications::{Aria2Event, Aria2EventParams};
    use crate::config::Config;

    #[tokio::test]
    async fn test_create_server() {
        let registry = Arc::new(RwLock::new(ToolRegistry::new(&Config::default())));
        let resource_registry = Arc::new(RwLock::new(ResourceRegistry::default()));
        let prompt_registry = Arc::new(RwLock::new(PromptRegistry::default()));
        let client = Arc::new(Aria2Client::new(Config::default()));

        let _server = create_server(registry, resource_registry, prompt_registry, vec![client]);
    }

    #[tokio::test]
    async fn test_notification_loop_logic() {
        let (tx, rx) = tokio::sync::mpsc::channel::<Aria2Notification>(1);
        let (transport_tx, mut transport_rx) = tokio::sync::mpsc::unbounded_channel::<
            Result<mcp_sdk_rs::transport::Message, String>,
        >();

        let notification = Aria2Notification {
            jsonrpc: "2.0".to_string(),
            method: Aria2Event::DownloadStart,
            params: vec![Aria2EventParams {
                gid: "123".to_string(),
            }],
        };
        tx.send(notification).await.unwrap();
        drop(tx);

        let mut rx = rx;
        if let Some(n) = rx.recv().await {
            let mcp_n = n.to_mcp_notification();
            let msg = serde_json::from_value::<mcp_sdk_rs::transport::Message>(mcp_n).unwrap();
            transport_tx.send(Ok(msg)).unwrap();
        }

        let received = transport_rx.recv().await.unwrap().unwrap();
        assert!(format!("{:?}", received).contains("notifications/aria2/event"));
    }
}
