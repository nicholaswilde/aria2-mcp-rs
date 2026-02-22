use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;

use crate::aria2::Aria2Client;
use crate::resources::McpResource;

pub struct GlobalStatusResource;

#[async_trait]
impl McpResource for GlobalStatusResource {
    fn uri(&self) -> String {
        "aria2://status/global".to_string()
    }

    fn name(&self) -> String {
        "Global Status".to_string()
    }

    fn description(&self) -> Option<String> {
        Some("Global statistics including download/upload speed and number of active/waiting/stopped downloads.".to_string())
    }

    fn mime_type(&self) -> Option<String> {
        Some("application/json".to_string())
    }

    async fn read(&self, client: &Aria2Client) -> Result<Value> {
        client.get_global_stat().await
    }

    async fn read_multi(&self, clients: &[Arc<Aria2Client>]) -> Result<Value> {
        let mut results = Vec::new();
        for client in clients {
            match client.get_global_stat().await {
                Ok(stats) => {
                    results.push(serde_json::json!({
                        "instance": client.name,
                        "status": "ok",
                        "stats": stats
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
