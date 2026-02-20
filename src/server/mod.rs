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
