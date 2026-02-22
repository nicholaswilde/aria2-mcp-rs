use anyhow::Result;
use async_trait::async_trait;
use serde::Serialize;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::aria2::Aria2Client;
use crate::Config;

use super::bulk_manage_downloads::BulkManageDownloadsTool;
use super::check_health::CheckHealthTool;
use super::configure_aria2::ConfigureAria2Tool;
use super::inspect_download::InspectDownloadTool;
use super::manage_all_instances::ManageAllInstancesTool;
use super::manage_downloads::ManageDownloadsTool;
use super::manage_torrent::ManageTorrentTool;
use super::monitor_queue::MonitorQueueTool;
use super::organize_completed::OrganizeCompletedTool;
use super::schedule_limits::ScheduleLimitsTool;
use super::search_downloads::SearchDownloadsTool;

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn input_schema(&self) -> Value;
    async fn execute(&self, client: &Aria2Client, args: Value) -> Result<Value>;
}

// Rename to McpeTool to match usage in other files
#[async_trait]
pub trait McpeTool: Send + Sync {
    fn name(&self) -> String;
    fn description(&self) -> String;
    fn schema(&self) -> Result<Value>;
    async fn run(&self, client: &Aria2Client, args: Value) -> Result<Value>;

    /// Run the tool with multiple clients.
    /// Default implementation just uses the first client or returns an error if no clients.
    async fn run_multi(&self, clients: &[Arc<Aria2Client>], args: Value) -> Result<Value> {
        let client = clients
            .first()
            .ok_or_else(|| anyhow::anyhow!("No clients provided"))?;
        self.run(client, args).await
    }
}

pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn McpeTool>>,
    enabled_tools: HashSet<String>,
    lazy_mode: bool,
}

struct ToolWrapper {
    tool: Arc<dyn McpeTool>,
}

#[async_trait]
impl McpeTool for ToolWrapper {
    fn name(&self) -> String {
        self.tool.name()
    }

    fn description(&self) -> String {
        self.tool.description()
    }

    fn schema(&self) -> Result<Value> {
        let mut schema = self.tool.schema()?;
        if let Some(properties) = schema.get_mut("properties") {
            if let Some(props_obj) = properties.as_object_mut() {
                props_obj.insert(
                    "instance".to_string(),
                    serde_json::json!({
                        "type": "integer",
                        "description": "The index of the aria2 instance to target (0, 1, etc.). Defaults to 0."
                    }),
                );
            }
        } else {
            // If no properties, create them
            schema["properties"] = serde_json::json!({
                "instance": {
                    "type": "integer",
                    "description": "The index of the aria2 instance to target (0, 1, etc.). Defaults to 0."
                }
            });
            if schema["type"].is_null() {
                schema["type"] = serde_json::json!("object");
            }
        }
        Ok(schema)
    }

    async fn run(&self, client: &Aria2Client, args: Value) -> Result<Value> {
        self.tool.run(client, args).await
    }

    async fn run_multi(&self, clients: &[Arc<Aria2Client>], args: Value) -> Result<Value> {
        // If this is a global tool (like manage_all_instances), it will override run_multi in its implementation.
        // But since we are wrapping it, we need to decide whether to route or let it handle all.

        // Strategy:
        // 1. If 'instance' is provided, route to specific client.
        // 2. If 'instance' is NOT provided, call the underlying tool's run_multi.

        if let Some(instance_idx) = args.get("instance").and_then(|v| v.as_u64()) {
            let client = clients
                .get(instance_idx as usize)
                .ok_or_else(|| anyhow::anyhow!("Invalid instance index: {}", instance_idx))?;
            self.tool.run(client, args).await
        } else {
            self.tool.run_multi(clients, args).await
        }
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new(&Config::default())
    }
}

impl ToolRegistry {
    pub fn new(config: &Config) -> Self {
        let mut registry = Self {
            tools: HashMap::new(),
            enabled_tools: HashSet::new(),
            lazy_mode: config.lazy_mode,
        };

        registry.register(Arc::new(ManageDownloadsTool));
        registry.register(Arc::new(MonitorQueueTool));
        registry.register(Arc::new(InspectDownloadTool));
        registry.register(Arc::new(ConfigureAria2Tool));
        registry.register(Arc::new(SearchDownloadsTool));
        registry.register(Arc::new(BulkManageDownloadsTool));
        registry.register(Arc::new(CheckHealthTool));
        registry.register(Arc::new(ManageAllInstancesTool));
        registry.register(Arc::new(ManageTorrentTool));
        registry.register(Arc::new(OrganizeCompletedTool));
        registry.register(Arc::new(ScheduleLimitsTool));

        // In lazy mode, only enable basic tools by default.
        // register() already enables all tools if !lazy_mode.
        if config.lazy_mode {
            registry.enabled_tools.insert("monitor_queue".to_string());
        }

        registry
    }

    pub fn register(&mut self, tool: Arc<dyn McpeTool>) {
        let wrapped = Arc::new(ToolWrapper { tool });
        let name = wrapped.name();
        self.tools.insert(name.clone(), wrapped);
        if !self.lazy_mode {
            self.enabled_tools.insert(name);
        }
    }

    pub fn get_tool(&self, name: &str) -> Option<Arc<dyn McpeTool>> {
        self.tools.get(name).cloned()
    }

    pub fn list_tools(&self) -> Vec<Arc<dyn McpeTool>> {
        self.tools
            .iter()
            .filter(|(name, _)| self.enabled_tools.contains(*name))
            .map(|(_, tool)| tool.clone())
            .collect()
    }

    pub fn enable_tool(&mut self, name: &str) -> bool {
        if self.tools.contains_key(name) {
            self.enabled_tools.insert(name.to_string());
            true
        } else {
            false
        }
    }

    pub fn disable_tool(&mut self, name: &str) -> bool {
        self.enabled_tools.remove(name)
    }

    pub fn is_tool_enabled(&self, name: &str) -> bool {
        self.enabled_tools.contains(name)
    }

    pub fn list_available_tools(&self) -> Vec<Value> {
        let mut result = Vec::new();
        for (name, tool) in &self.tools {
            result.push(serde_json::json!({
                "name": name,
                "description": tool.description(),
                "enabled": self.enabled_tools.contains(name)
            }));
        }
        result.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
        result
    }

    pub fn is_lazy_mode(&self) -> bool {
        self.lazy_mode
    }
}

#[derive(Serialize)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_new() {
        let registry = ToolRegistry::new(&Config::default());
        let tools = registry.list_tools();
        assert_eq!(tools.len(), 11);
    }

    #[test]
    fn test_registry_get_tool() {
        let registry = ToolRegistry::new(&Config::default());
        assert!(registry.get_tool("manage_downloads").is_some());
        assert!(registry.get_tool("unknown").is_none());
    }

    #[test]
    fn test_registry_lazy_mode() {
        let config = Config {
            lazy_mode: true,
            ..Default::default()
        };
        let registry = ToolRegistry::new(&config);
        let tools = registry.list_tools();
        assert_eq!(tools.len(), 1); // Only monitor_queue
        assert_eq!(tools[0].name(), "monitor_queue");
    }

    #[test]
    fn test_registry_enable_disable() {
        let config = Config {
            lazy_mode: true,
            ..Default::default()
        };
        let mut registry = ToolRegistry::new(&config);

        assert!(!registry.is_tool_enabled("manage_downloads"));

        assert!(registry.enable_tool("manage_downloads"));
        assert!(registry.is_tool_enabled("manage_downloads"));
        assert_eq!(registry.list_tools().len(), 2);

        assert!(registry.disable_tool("manage_downloads"));
        assert!(!registry.is_tool_enabled("manage_downloads"));
        assert_eq!(registry.list_tools().len(), 1);
    }

    #[test]
    fn test_registry_list_available() {
        let config = Config::default();
        let registry = ToolRegistry::new(&config);
        let available = registry.list_available_tools();
        assert_eq!(available.len(), 11);
        for tool in available {
            assert!(tool["enabled"].as_bool().unwrap());
        }
    }

    #[test]
    fn test_registry_tool_schema_has_instance() {
        let registry = ToolRegistry::new(&Config::default());
        let tool = registry.get_tool("manage_downloads").unwrap();
        let schema = tool.schema().unwrap();
        assert!(schema["properties"]["instance"].is_object());
        assert_eq!(schema["properties"]["instance"]["type"], "integer");
    }

    #[test]
    fn test_registry_default() {
        let _registry = ToolRegistry::default();
    }
}
