use async_trait::async_trait;
use mcp_sdk_rs::{
    error::{Error, ErrorCode},
    server::ServerHandler,
    types::{ClientCapabilities, Implementation, ServerCapabilities},
};
use std::sync::Arc;

use crate::aria2::Aria2Client;
use crate::tools::registry::ToolRegistry;

pub struct McpHandler {
    registry: Arc<ToolRegistry>,
    client: Arc<Aria2Client>,
}

impl McpHandler {
    pub fn new(registry: Arc<ToolRegistry>, client: Arc<Aria2Client>) -> Self {
        Self { registry, client }
    }
}

#[async_trait]
impl ServerHandler for McpHandler {
    async fn initialize(
        &self,
        _implementation: Implementation,
        _capabilities: ClientCapabilities,
    ) -> Result<ServerCapabilities, Error> {
        Ok(ServerCapabilities {
            tools: Some(serde_json::json!({})),
            ..Default::default()
        })
    }

    async fn shutdown(&self) -> Result<(), Error> {
        Ok(())
    }

    async fn handle_method(
        &self,
        method: &str,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, Error> {
        match method {
            "tools/list" => {
                let tools = self.registry.list_tools();
                let mut tool_infos = Vec::new();
                for t in tools {
                    let schema = t.schema().map_err(|e| {
                        Error::protocol(ErrorCode::InternalError, format!("Schema error: {}", e))
                    })?;
                    tool_infos.push(serde_json::json!({
                        "name": t.name(),
                        "description": t.description(),
                        "inputSchema": schema,
                    }));
                }

                Ok(serde_json::json!({ "tools": tool_infos }))
            }
            "tools/call" => {
                let params = params.ok_or_else(|| {
                    Error::protocol(
                        ErrorCode::InvalidParams,
                        "Missing parameters for tools/call",
                    )
                })?;

                let name = params.get("name").and_then(|v| v.as_str()).ok_or_else(|| {
                    Error::protocol(ErrorCode::InvalidParams, "Missing 'name' in tools/call")
                })?;

                let arguments = params
                    .get("arguments")
                    .cloned()
                    .unwrap_or(serde_json::json!({}));

                let tool = self.registry.get_tool(name).ok_or_else(|| {
                    Error::protocol(
                        ErrorCode::MethodNotFound,
                        format!("Tool not found: {}", name),
                    )
                })?;

                let result = tool
                    .run(&self.client, arguments)
                    .await
                    .map_err(|e| Error::protocol(ErrorCode::InternalError, e.to_string()))?;

                Ok(serde_json::json!({
                    "content": [{
                        "type": "text",
                        "text": result.to_string()
                    }]
                }))
            }
            _ => Err(Error::protocol(
                ErrorCode::MethodNotFound,
                format!("Unknown method: {}", method),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aria2::Aria2Client;
    use crate::config::Config;
    use crate::tools::registry::ToolRegistry;

    #[tokio::test]
    async fn test_handler_tools_list() {
        let registry = Arc::new(ToolRegistry::new());
        let client = Arc::new(Aria2Client::new(Config::default()));
        let handler = McpHandler::new(registry, client);

        let result = handler.handle_method("tools/list", None).await.unwrap();
        let tools = result["tools"].as_array().unwrap();
        assert!(!tools.is_empty());
    }

    #[tokio::test]
    async fn test_handler_unknown_method() {
        let registry = Arc::new(ToolRegistry::new());
        let client = Arc::new(Aria2Client::new(Config::default()));
        let handler = McpHandler::new(registry, client);

        let result = handler.handle_method("unknown", None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_handler_tools_call_missing_params() {
        let registry = Arc::new(ToolRegistry::new());
        let client = Arc::new(Aria2Client::new(Config::default()));
        let handler = McpHandler::new(registry, client);

        let result = handler.handle_method("tools/call", None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_handler_tools_call_missing_name() {
        let registry = Arc::new(ToolRegistry::new());
        let client = Arc::new(Aria2Client::new(Config::default()));
        let handler = McpHandler::new(registry, client);

        let params = serde_json::json!({ "arguments": {} });
        let result = handler.handle_method("tools/call", Some(params)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_handler_tools_call_not_found() {
        let registry = Arc::new(ToolRegistry::new());
        let client = Arc::new(Aria2Client::new(Config::default()));
        let handler = McpHandler::new(registry, client);

        let params = serde_json::json!({ "name": "unknown", "arguments": {} });
        let result = handler.handle_method("tools/call", Some(params)).await;
        assert!(result.is_err());
    }
}
