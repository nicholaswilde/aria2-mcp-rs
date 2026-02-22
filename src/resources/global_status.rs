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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_global_status_resource_read() -> Result<()> {
        let server = MockServer::start().await;

        let mock_response = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "aria2-mcp",
            "result": {
                "downloadSpeed": "1024",
                "uploadSpeed": "512",
                "numActive": "1",
                "numWaiting": "2",
                "numStopped": "3"
            }
        });

        Mock::given(method("POST"))
            .and(path("/"))
            .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
            .mount(&server)
            .await;

        let mut config = Config::default();
        config.rpc_url = format!("http://{}", server.address());
        let client = Aria2Client::new(config);

        let resource = GlobalStatusResource;
        let result = resource.read(&client).await?;

        assert_eq!(result["downloadSpeed"], "1024");
        assert_eq!(result["numActive"], "1");

        Ok(())
    }

    #[tokio::test]
    async fn test_global_status_resource_read_multi() -> Result<()> {
        let server1 = MockServer::start().await;
        let server2 = MockServer::start().await;

        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "jsonrpc": "2.0", "id": "aria2-mcp", "result": {"numActive": "1"}
            })))
            .mount(&server1)
            .await;

        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "jsonrpc": "2.0", "id": "aria2-mcp", "result": {"numActive": "2"}
            })))
            .mount(&server2)
            .await;

        let mut config1 = Config::default();
        config1.rpc_url = format!("http://{}", server1.address());
        let client1 = Arc::new(Aria2Client::new(config1));

        let mut config2 = Config::default();
        config2.rpc_url = format!("http://{}", server2.address());
        let mut client2 = Aria2Client::new(config2);
        client2.name = "box2".to_string();
        let client2 = Arc::new(client2);

        let resource = GlobalStatusResource;
        let results = resource.read_multi(&[client1, client2]).await?;

        assert_eq!(results.as_array().unwrap().len(), 2);
        assert_eq!(results[0]["instance"], "default");
        assert_eq!(results[0]["stats"]["numActive"], "1");
        assert_eq!(results[1]["instance"], "box2");
        assert_eq!(results[1]["stats"]["numActive"], "2");

        Ok(())
    }
}
