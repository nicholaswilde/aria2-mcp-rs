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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_active_downloads_resource_read() -> Result<()> {
        let server = MockServer::start().await;

        let mock_response = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "aria2-mcp",
            "result": [
                {
                    "gid": "123",
                    "status": "active",
                    "totalLength": "100",
                    "completedLength": "50"
                }
            ]
        });

        Mock::given(method("POST"))
            .and(path("/"))
            .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
            .mount(&server)
            .await;

        let config = Config {
            rpc_url: format!("http://{}", server.address()),
            ..Config::default()
        };
        let client = Aria2Client::new(config);

        let resource = ActiveDownloadsResource;
        let result = resource.read(&client).await?;

        assert!(result.is_array());
        assert_eq!(result.as_array().unwrap().len(), 1);
        assert_eq!(result[0]["gid"], "123");

        Ok(())
    }

    #[tokio::test]
    async fn test_active_downloads_resource_read_empty() -> Result<()> {
        let server = MockServer::start().await;

        let mock_response = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "aria2-mcp",
            "result": []
        });

        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
            .mount(&server)
            .await;

        let config = Config {
            rpc_url: format!("http://{}", server.address()),
            ..Config::default()
        };
        let client = Aria2Client::new(config);

        let resource = ActiveDownloadsResource;
        let result = resource.read(&client).await?;

        assert!(result.is_array());
        assert_eq!(result.as_array().unwrap().len(), 0);

        Ok(())
    }
}
