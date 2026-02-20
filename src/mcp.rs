use crate::Config;
use anyhow::Result;

pub struct McpServer {
    config: Config,
}

impl McpServer {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub async fn run(&self) -> Result<()> {
        // Simple skeleton for now that doesn't rely on specific SDK structures
        // until we decide on the exact server pattern (stdio vs http)

        println!(
            "MCP Server initialized for aria2 at {}",
            self.config.rpc_url
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mcp_server_new() {
        let config = Config::default();
        let server = McpServer::new(config);
        assert_eq!(server.config.rpc_url, "http://localhost:6800/jsonrpc");
    }

    #[tokio::test]
    async fn test_mcp_server_run_skeleton() {
        let config = Config::default();
        let server = McpServer::new(config);
        let result = server.run().await;
        assert!(result.is_ok());
    }
}
