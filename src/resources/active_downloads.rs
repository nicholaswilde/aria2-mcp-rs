use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;

use crate::aria2::Aria2Client;
use crate::resources::McpResource;

pub struct ActiveDownloadsResource;

#[async_trait]
impl McpResource for ActiveDownloadsResource {
    fn uri(&self) -> String {
        "aria2://downloads/active".to_string()
    }

    fn name(&self) -> String {
        "Active Downloads".to_string()
    }

    fn description(&self) -> Option<String> {
        Some("List of currently active downloads.".to_string())
    }

    fn mime_type(&self) -> Option<String> {
        Some("application/json".to_string())
    }

    async fn read(&self, client: &Aria2Client) -> Result<Value> {
        client.tell_active(None).await
    }
}
