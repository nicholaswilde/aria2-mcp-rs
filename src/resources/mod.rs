use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;

use crate::aria2::Aria2Client;

pub mod active_downloads;
pub mod global_status;
pub mod recent_logs;
pub mod registry;

pub use active_downloads::ActiveDownloadsResource;
pub use global_status::GlobalStatusResource;
pub use recent_logs::RecentLogsResource;
pub use registry::ResourceRegistry;

#[async_trait]
pub trait McpResource: Send + Sync {
    fn uri(&self) -> String;
    fn name(&self) -> String;
    fn description(&self) -> Option<String>;
    fn mime_type(&self) -> Option<String>;
    async fn read(&self, client: &Aria2Client) -> Result<Value>;
    async fn read_multi(&self, clients: &[Arc<Aria2Client>]) -> Result<Value> {
        let mut results = Vec::new();
        for client in clients {
            match self.read(client).await {
                Ok(data) => {
                    results.push(serde_json::json!({
                        "instance": client.name,
                        "status": "ok",
                        "data": data
                    }));
                }
                Err(e) => {
                    results.push(serde_json::json!({
                        "instance": client.name,
                        "status": "error",
                        "error": e.to_string()
                    }));
                }
            }
        }
        Ok(serde_json::json!(results))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aria2::Aria2Client;

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
        assert_eq!(list.len(), 2 + 3); // 2 mocks + 3 default resources (GlobalStatus, ActiveDownloads, RecentLogs)
                                       // GlobalStatus is aria2://status/global
                                       // ActiveDownloads is aria2://downloads/active
                                       // RecentLogs is aria2://logs/recent
                                       // Mocks are test://1 and test://2

        assert!(registry.get_resource("test://1").is_some());
        assert!(registry.get_resource("test://3").is_none());
    }
}
