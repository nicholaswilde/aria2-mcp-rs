use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;

use crate::aria2::Aria2Client;

#[async_trait]
pub trait McpResource: Send + Sync {
    /// The URI template for this resource (e.g., "aria2://status/global")
    fn uri(&self) -> String;

    /// A human-readable name for this resource
    fn name(&self) -> String;

    /// A description of what this resource provides
    fn description(&self) -> Option<String>;

    /// The MIME type of the resource content (e.g., "application/json", "text/plain")
    fn mime_type(&self) -> Option<String>;

    /// Read the resource content
    async fn read(&self, client: &Aria2Client) -> Result<Value>;

    /// Read the resource with context of multiple clients
    /// Default implementation uses the first client
    async fn read_multi(&self, clients: &[Arc<Aria2Client>]) -> Result<Value> {
        let client = clients
            .first()
            .ok_or_else(|| anyhow::anyhow!("No clients provided"))?;
        self.read(client).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    struct MockResource;

    #[async_trait]
    impl McpResource for MockResource {
        fn uri(&self) -> String {
            "mock://uri".to_string()
        }
        fn name(&self) -> String {
            "Mock".to_string()
        }
        fn description(&self) -> Option<String> {
            Some("Mock Desc".to_string())
        }
        fn mime_type(&self) -> Option<String> {
            Some("text/plain".to_string())
        }
        async fn read(&self, _client: &Aria2Client) -> Result<Value> {
            Ok(serde_json::json!({"status": "ok"}))
        }
    }

    #[tokio::test]
    async fn test_resource_trait() {
        let resource = MockResource;
        let config = Config::default();
        let client = Aria2Client::new(config);

        assert_eq!(resource.uri(), "mock://uri");
        assert_eq!(resource.name(), "Mock");

        let result = resource.read(&client).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), serde_json::json!({"status": "ok"}));
    }
}
