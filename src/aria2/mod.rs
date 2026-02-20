use crate::Config;
use anyhow::Result;
use reqwest::Client;

#[allow(dead_code)]
pub struct Aria2Client {
    config: Config,
    client: Client,
}

impl Aria2Client {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }

    pub async fn get_version(&self) -> Result<String> {
        let mut params = Vec::new();
        if let Some(secret) = &self.config.rpc_secret {
            params.push(serde_json::json!(format!("token:{}", secret)));
        }

        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "aria2-mcp",
            "method": "aria2.getVersion",
            "params": params,
        });

        let resp = self.client.post(&self.config.rpc_url)
            .json(&body)
            .send()
            .await?;

        let res: serde_json::Value = resp.json().await?;
        
        if let Some(err) = res.get("error") {
            return Err(anyhow::anyhow!("aria2 error: {}", err));
        }

        let version = res["result"]["version"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Failed to get version from response"))?;

        Ok(version.to_string())
    }
}
