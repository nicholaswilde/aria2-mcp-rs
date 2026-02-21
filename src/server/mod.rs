pub mod handler;
pub mod sse;
pub mod stdio;

use anyhow::Result;
use std::sync::Arc;

use crate::aria2::Aria2Client;
use crate::config::{Config, TransportType};
use crate::tools::ToolRegistry;

pub struct McpServer {
    config: Config,
    registry: Arc<ToolRegistry>,
    client: Arc<Aria2Client>,
}

impl McpServer {
    pub fn new(config: Config, registry: ToolRegistry, client: Aria2Client) -> Self {
        Self {
            config,
            registry: Arc::new(registry),
            client: Arc::new(client),
        }
    }

    pub async fn run(&self) -> Result<()> {
        match self.config.transport {
            TransportType::Stdio => {
                stdio::run_server(Arc::clone(&self.registry), Arc::clone(&self.client)).await
            }
            TransportType::Sse => {
                sse::run_server(
                    self.config.port,
                    Arc::clone(&self.registry),
                    Arc::clone(&self.client),
                )
                .await
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_server() {
        let config = Config::default();
        let registry = ToolRegistry::new();
        let client = Aria2Client::new(config.clone());
        let _server = McpServer::new(config, registry, client);
    }

    #[tokio::test]
    async fn test_server_run_sse_error() {
        let config = Config {
            transport: TransportType::Sse,
            port: 1, // Likely to fail on most systems
            ..Default::default()
        };
        let registry = ToolRegistry::new();
        let client = Aria2Client::new(config.clone());
        let server = McpServer::new(config, registry, client);
        let result = server.run().await;
        assert!(result.is_err());
    }
}
