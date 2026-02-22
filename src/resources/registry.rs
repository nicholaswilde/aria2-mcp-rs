use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

use super::McpResource;

use super::active_downloads::ActiveDownloadsResource;
use super::global_status::GlobalStatusResource;
use super::recent_logs::RecentLogsResource; // Import RecentLogsResource

pub struct ResourceRegistry {
    resources: HashMap<String, Arc<dyn McpResource>>,
}

impl Default for ResourceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ResourceRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            resources: HashMap::new(),
        };

        registry.register(Arc::new(GlobalStatusResource));
        registry.register(Arc::new(ActiveDownloadsResource));
        // Register RecentLogsResource with default values
        registry.register(Arc::new(RecentLogsResource::new(
            "/var/log/aria2-mcp-rs.log".to_string(),
            100,
        )));

        registry
    }

    pub fn register(&mut self, resource: Arc<dyn McpResource>) {
        self.resources.insert(resource.uri(), resource);
    }

    pub fn get_resource(&self, uri: &str) -> Option<Arc<dyn McpResource>> {
        self.resources.get(uri).cloned()
    }

    pub fn list_resources(&self) -> Vec<Value> {
        let mut results = Vec::new();
        for resource in self.resources.values() {
            let mut val = serde_json::json!({
                "uri": resource.uri(),
                "name": resource.name(),
            });
            if let Some(desc) = resource.description() {
                val["description"] = serde_json::json!(desc);
            }
            if let Some(mime) = resource.mime_type() {
                val["mimeType"] = serde_json::json!(mime);
            }
            results.push(val);
        }
        results.sort_by(|a, b| a["uri"].as_str().cmp(&b["uri"].as_str()));
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aria2::Aria2Client;
    use anyhow::Result;
    use async_trait::async_trait;

    struct MockResource {
        uri: String,
    }

    #[async_trait]
    impl McpResource for MockResource {
        fn uri(&self) -> String {
            self.uri.clone()
        }
        fn name(&self) -> String {
            "Mock".to_string()
        }
        fn description(&self) -> Option<String> {
            Some("Desc".to_string())
        }
        fn mime_type(&self) -> Option<String> {
            None
        }
        async fn read(&self, _client: &Aria2Client) -> Result<Value> {
            Ok(Value::Null)
        }
    }

    #[test]
    fn test_registry() {
        let mut registry = ResourceRegistry::new();
        let r1 = Arc::new(MockResource {
            uri: "test://1".to_string(),
        });
        let r2 = Arc::new(MockResource {
            uri: "test://2".to_string(),
        });

        registry.register(r1);
        registry.register(r2);

        let list = registry.list_resources();
        // 3 default resources + 2 mock resources
        assert_eq!(list.len(), 5);

        // Resources are sorted by URI:
        // 1. aria2://downloads/active
        // 2. aria2://logs/recent
        // 3. aria2://status/global
        // 4. test://1
        // 5. test://2
        assert_eq!(list[3]["uri"], "test://1");
        assert_eq!(list[4]["uri"], "test://2");

        assert!(registry.get_resource("test://1").is_some());
        assert!(registry.get_resource("test://3").is_none());
    }
}
