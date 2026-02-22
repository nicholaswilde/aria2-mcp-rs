use async_trait::async_trait;
use mcp_sdk_rs::{
    error::{Error, ErrorCode},
    server::ServerHandler,
    types::{ClientCapabilities, Implementation, ServerCapabilities},
};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::aria2::Aria2Client;
use crate::tools::registry::ToolRegistry;

pub struct McpHandler {
    registry: Arc<RwLock<ToolRegistry>>,
    clients: Vec<Arc<Aria2Client>>,
}

impl McpHandler {
    pub fn new(registry: Arc<RwLock<ToolRegistry>>, clients: Vec<Arc<Aria2Client>>) -> Self {
        Self { registry, clients }
    }

    fn get_client(&self, arguments: &serde_json::Value) -> Result<Arc<Aria2Client>, Error> {
        let instance_idx = arguments
            .get("instance")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;

        self.clients.get(instance_idx).cloned().ok_or_else(|| {
            Error::protocol(
                ErrorCode::InvalidParams,
                format!("Invalid instance index: {}", instance_idx),
            )
        })
    }

    async fn handle_manage_tools(
        &self,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value, Error> {
        let action = arguments
            .get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                Error::protocol(ErrorCode::InvalidParams, "Missing 'action' in manage_tools")
            })?;

        match action {
            "list" => {
                let registry = self.registry.read().await;
                let tools = registry.list_available_tools();
                Ok(serde_json::json!({
                    "content": [{
                        "type": "text",
                        "text": serde_json::to_string_pretty(&tools).unwrap_or_default()
                    }]
                }))
            }
            "enable" => {
                let tools_to_enable = arguments
                    .get("tools")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| {
                        Error::protocol(
                            ErrorCode::InvalidParams,
                            "Missing 'tools' in manage_tools enable",
                        )
                    })?;

                let mut registry = self.registry.write().await;
                let mut enabled_count = 0;
                for t in tools_to_enable {
                    if let Some(name) = t.as_str() {
                        if registry.enable_tool(name) {
                            enabled_count += 1;
                        }
                    }
                }
                Ok(serde_json::json!({
                    "content": [{
                        "type": "text",
                        "text": format!("Enabled {} tools.", enabled_count)
                    }]
                }))
            }
            "disable" => {
                let tools_to_disable = arguments
                    .get("tools")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| {
                        Error::protocol(
                            ErrorCode::InvalidParams,
                            "Missing 'tools' in manage_tools disable",
                        )
                    })?;

                let mut registry = self.registry.write().await;
                let mut disabled_count = 0;
                for t in tools_to_disable {
                    if let Some(name) = t.as_str() {
                        if registry.disable_tool(name) {
                            disabled_count += 1;
                        }
                    }
                }
                Ok(serde_json::json!({
                    "content": [{
                        "type": "text",
                        "text": format!("Disabled {} tools.", disabled_count)
                    }]
                }))
            }
            _ => Err(Error::protocol(
                ErrorCode::InvalidParams,
                format!("Invalid action: {}", action),
            )),
        }
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
                let registry = self.registry.read().await;
                let tools = registry.list_tools();
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

                if registry.is_lazy_mode() {
                    tool_infos.push(serde_json::json!({
                        "name": "manage_tools",
                        "description": "Manage available tools (enable/disable) to save tokens.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "action": {
                                    "type": "string",
                                    "enum": ["list", "enable", "disable"],
                                    "description": "The action to perform."
                                },
                                "tools": {
                                    "type": "array",
                                    "items": { "type": "string" },
                                    "description": "List of tool names to enable or disable."
                                }
                            },
                            "required": ["action"]
                        }
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

                if name == "manage_tools" {
                    let registry = self.registry.read().await;
                    if registry.is_lazy_mode() {
                        drop(registry);
                        return self.handle_manage_tools(arguments).await;
                    }
                }

                let registry = self.registry.read().await;
                let tool = registry.get_tool(name).ok_or_else(|| {
                    Error::protocol(
                        ErrorCode::MethodNotFound,
                        format!("Tool not found: {}", name),
                    )
                })?;
                drop(registry);

                let client = self.get_client(&arguments)?;

                let result = tool
                    .run(&client, arguments)
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
        let registry = Arc::new(RwLock::new(ToolRegistry::new(&Config::default())));
        let client = Arc::new(Aria2Client::new(Config::default()));
        let handler = McpHandler::new(registry, vec![client]);

        let result = handler.handle_method("tools/list", None).await.unwrap();
        let tools = result["tools"].as_array().unwrap();
        assert!(!tools.is_empty());
    }

    #[tokio::test]
    async fn test_handler_unknown_method() {
        let registry = Arc::new(RwLock::new(ToolRegistry::new(&Config::default())));
        let client = Arc::new(Aria2Client::new(Config::default()));
        let handler = McpHandler::new(registry, vec![client]);

        let result = handler.handle_method("unknown", None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_handler_tools_call_missing_params() {
        let registry = Arc::new(RwLock::new(ToolRegistry::new(&Config::default())));
        let client = Arc::new(Aria2Client::new(Config::default()));
        let handler = McpHandler::new(registry, vec![client]);

        let result = handler.handle_method("tools/call", None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_handler_tools_call_missing_name() {
        let registry = Arc::new(RwLock::new(ToolRegistry::new(&Config::default())));
        let client = Arc::new(Aria2Client::new(Config::default()));
        let handler = McpHandler::new(registry, vec![client]);

        let params = serde_json::json!({ "arguments": {} });
        let result = handler.handle_method("tools/call", Some(params)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_handler_tools_call_not_found() {
        let registry = Arc::new(RwLock::new(ToolRegistry::new(&Config::default())));
        let client = Arc::new(Aria2Client::new(Config::default()));
        let handler = McpHandler::new(registry, vec![client]);

        let params = serde_json::json!({ "name": "unknown", "arguments": {} });
        let result = handler.handle_method("tools/call", Some(params)).await;
        assert!(result.is_err());
    }
}
