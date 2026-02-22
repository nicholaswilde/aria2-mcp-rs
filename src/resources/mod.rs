use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;

use crate::aria2::Aria2Client;
use crate::resources::McpResource;

pub mod active_downloads;
pub mod registry;

pub use active_downloads::ActiveDownloadsResource;
pub use registry::ResourceRegistry;

#[async_trait]
impl McpResource for MockResource {
    fn uri(&self) -> String { "mock://uri".to_string() }
    fn name(&self) -> String { "Mock".to_string() }
    fn description(&self) -> Option<String> { Some("Mock Desc".to_string()) }
    fn mime_type(&self) -> Option<String> { Some("text/plain".to_string()) }
    async fn read(&self, _client: &Aria2Client) -> Result<Value> {
        Ok(serde_json::json!({"status": "ok"}))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aria2::Aria2Client;
    use crate::config::Config;
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