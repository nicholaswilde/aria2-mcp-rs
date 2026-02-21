use anyhow::Result;
use async_trait::async_trait;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

use crate::aria2::Aria2Client;

use super::configure_aria2::ConfigureAria2Tool;
use super::inspect_download::InspectDownloadTool;
use super::manage_downloads::ManageDownloadsTool;
use super::monitor_queue::MonitorQueueTool;
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
}

pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn McpeTool>>,
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            tools: HashMap::new(),
        };

        registry.register(Arc::new(ManageDownloadsTool));
        registry.register(Arc::new(MonitorQueueTool));
        registry.register(Arc::new(InspectDownloadTool));
        registry.register(Arc::new(ConfigureAria2Tool));
        registry.register(Arc::new(SearchDownloadsTool));

        registry
    }

    pub fn register(&mut self, tool: Arc<dyn McpeTool>) {
        self.tools.insert(tool.name(), tool);
    }

    pub fn get_tool(&self, name: &str) -> Option<Arc<dyn McpeTool>> {
        self.tools.get(name).cloned()
    }

    pub fn list_tools(&self) -> Vec<Arc<dyn McpeTool>> {
        self.tools.values().cloned().collect()
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
        let registry = ToolRegistry::new();
        let tools = registry.list_tools();
        assert_eq!(tools.len(), 5);
    }

    #[test]
    fn test_registry_get_tool() {
        let registry = ToolRegistry::new();
        assert!(registry.get_tool("manage_downloads").is_some());
        assert!(registry.get_tool("unknown").is_none());
    }

    #[test]
    fn test_registry_default() {
        let _registry = ToolRegistry::default();
    }
}
