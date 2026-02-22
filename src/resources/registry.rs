use std::collections::HashMap;
use std::sync::Arc;
use serde_json::Value;
use super::McpResource;

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
        Self {
            resources: HashMap::new(),
        }
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
        fn uri(&self) -> String { self.uri.clone() }
        fn name(&self) -> String { "Mock".to_string() }
        fn description(&self) -> Option<String> { Some("Desc".to_string()) }
        fn mime_type(&self) -> Option<String> { None }
        async fn read(&self, _client: &Aria2Client) -> Result<Value> { Ok(Value::Null) }
    }

    #[test]
    fn test_registry() {
        let mut registry = ResourceRegistry::new();
        let r1 = Arc::new(MockResource { uri: "test://1".to_string() });
        let r2 = Arc::new(MockResource { uri: "test://2".to_string() });

        registry.register(r1);
        registry.register(r2);

        let list = registry.list_resources();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0]["uri"], "test://1");
        assert_eq!(list[1]["uri"], "test://2");

        assert!(registry.get_resource("test://1").is_some());
        assert!(registry.get_resource("test://3").is_none());
    }
}
