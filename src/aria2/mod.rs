use crate::Config;
use anyhow::Result;
use reqwest::Client;
use std::sync::{Arc, RwLock};

#[allow(dead_code)]
#[derive(Clone)]
pub struct Aria2Client {
    config: Arc<RwLock<Config>>,
    client: Client,
}

impl Aria2Client {
    pub fn new(config: Config) -> Self {
        let client = Client::builder()
            .danger_accept_invalid_certs(config.no_verify_ssl)
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            config: Arc::new(RwLock::new(config)),
            client,
        }
    }

    pub fn config(&self) -> Arc<RwLock<Config>> {
        Arc::clone(&self.config)
    }

    pub async fn tell_active(&self, keys: Option<Vec<String>>) -> Result<serde_json::Value> {
        let (rpc_url, rpc_secret) = {
            let config = self
                .config
                .read()
                .map_err(|e| anyhow::anyhow!("Failed to read config: {}", e))?;
            (config.rpc_url.clone(), config.rpc_secret.clone())
        };

        let mut params = Vec::new();
        if let Some(secret) = &rpc_secret {
            params.push(serde_json::json!(format!("token:{}", secret)));
        }
        if let Some(k) = keys {
            params.push(serde_json::json!(k));
        }

        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "aria2-mcp",
            "method": "aria2.tellActive",
            "params": params,
        });

        let resp = self.client.post(&rpc_url).json(&body).send().await?;

        let res: serde_json::Value = resp.json().await?;

        if let Some(err) = res.get("error") {
            return Err(anyhow::anyhow!("aria2 error: {}", err));
        }

        Ok(res["result"].clone())
    }

    pub async fn tell_waiting(
        &self,
        offset: i32,
        num: i32,
        keys: Option<Vec<String>>,
    ) -> Result<serde_json::Value> {
        let (rpc_url, rpc_secret) = {
            let config = self
                .config
                .read()
                .map_err(|e| anyhow::anyhow!("Failed to read config: {}", e))?;
            (config.rpc_url.clone(), config.rpc_secret.clone())
        };
        let mut params = Vec::new();
        if let Some(secret) = &rpc_secret {
            params.push(serde_json::json!(format!("token:{}", secret)));
        }
        params.push(serde_json::json!(offset));
        params.push(serde_json::json!(num));
        if let Some(k) = keys {
            params.push(serde_json::json!(k));
        }

        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "aria2-mcp",
            "method": "aria2.tellWaiting",
            "params": params,
        });

        let resp = self.client.post(&rpc_url).json(&body).send().await?;

        let res: serde_json::Value = resp.json().await?;

        if let Some(err) = res.get("error") {
            return Err(anyhow::anyhow!("aria2 error: {}", err));
        }

        Ok(res["result"].clone())
    }

    pub async fn tell_stopped(
        &self,
        offset: i32,
        num: i32,
        keys: Option<Vec<String>>,
    ) -> Result<serde_json::Value> {
        let (rpc_url, rpc_secret) = {
            let config = self
                .config
                .read()
                .map_err(|e| anyhow::anyhow!("Failed to read config: {}", e))?;
            (config.rpc_url.clone(), config.rpc_secret.clone())
        };
        let mut params = Vec::new();
        if let Some(secret) = &rpc_secret {
            params.push(serde_json::json!(format!("token:{}", secret)));
        }
        params.push(serde_json::json!(offset));
        params.push(serde_json::json!(num));
        if let Some(k) = keys {
            params.push(serde_json::json!(k));
        }

        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "aria2-mcp",
            "method": "aria2.tellStopped",
            "params": params,
        });

        let resp = self.client.post(&rpc_url).json(&body).send().await?;

        let res: serde_json::Value = resp.json().await?;

        if let Some(err) = res.get("error") {
            return Err(anyhow::anyhow!("aria2 error: {}", err));
        }

        Ok(res["result"].clone())
    }

    pub async fn get_global_stat(&self) -> Result<serde_json::Value> {
        let (rpc_url, rpc_secret) = {
            let config = self
                .config
                .read()
                .map_err(|e| anyhow::anyhow!("Failed to read config: {}", e))?;
            (config.rpc_url.clone(), config.rpc_secret.clone())
        };
        let mut params = Vec::new();
        if let Some(secret) = &rpc_secret {
            params.push(serde_json::json!(format!("token:{}", secret)));
        }

        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "aria2-mcp",
            "method": "aria2.getGlobalStat",
            "params": params,
        });

        let resp = self.client.post(&rpc_url).json(&body).send().await?;

        let res: serde_json::Value = resp.json().await?;

        if let Some(err) = res.get("error") {
            return Err(anyhow::anyhow!("aria2 error: {}", err));
        }

        Ok(res["result"].clone())
    }

    pub async fn get_global_option(&self) -> Result<serde_json::Value> {
        let (rpc_url, rpc_secret) = {
            let config = self
                .config
                .read()
                .map_err(|e| anyhow::anyhow!("Failed to read config: {}", e))?;
            (config.rpc_url.clone(), config.rpc_secret.clone())
        };
        let mut params = Vec::new();
        if let Some(secret) = &rpc_secret {
            params.push(serde_json::json!(format!("token:{}", secret)));
        }

        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "aria2-mcp",
            "method": "aria2.getGlobalOption",
            "params": params,
        });

        let resp = self.client.post(&rpc_url).json(&body).send().await?;

        let res: serde_json::Value = resp.json().await?;

        if let Some(err) = res.get("error") {
            return Err(anyhow::anyhow!("aria2 error: {}", err));
        }

        Ok(res["result"].clone())
    }

    pub async fn get_option(&self, gid: &str) -> Result<serde_json::Value> {
        let (rpc_url, rpc_secret) = {
            let config = self
                .config
                .read()
                .map_err(|e| anyhow::anyhow!("Failed to read config: {}", e))?;
            (config.rpc_url.clone(), config.rpc_secret.clone())
        };
        let mut params = Vec::new();
        if let Some(secret) = &rpc_secret {
            params.push(serde_json::json!(format!("token:{}", secret)));
        }
        params.push(serde_json::json!(gid));

        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "aria2-mcp",
            "method": "aria2.getOption",
            "params": params,
        });

        let resp = self.client.post(&rpc_url).json(&body).send().await?;

        let res: serde_json::Value = resp.json().await?;

        if let Some(err) = res.get("error") {
            return Err(anyhow::anyhow!("aria2 error: {}", err));
        }

        Ok(res["result"].clone())
    }

    pub async fn change_global_option(&self, options: serde_json::Value) -> Result<()> {
        let (rpc_url, rpc_secret) = {
            let config = self
                .config
                .read()
                .map_err(|e| anyhow::anyhow!("Failed to read config: {}", e))?;
            (config.rpc_url.clone(), config.rpc_secret.clone())
        };
        let mut params = Vec::new();
        if let Some(secret) = &rpc_secret {
            params.push(serde_json::json!(format!("token:{}", secret)));
        }
        params.push(options);

        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "aria2-mcp",
            "method": "aria2.changeGlobalOption",
            "params": params,
        });

        let resp = self.client.post(&rpc_url).json(&body).send().await?;

        let res: serde_json::Value = resp.json().await?;

        if let Some(err) = res.get("error") {
            return Err(anyhow::anyhow!("aria2 error: {}", err));
        }

        Ok(())
    }

    pub async fn change_option(&self, gid: &str, options: serde_json::Value) -> Result<()> {
        let (rpc_url, rpc_secret) = {
            let config = self
                .config
                .read()
                .map_err(|e| anyhow::anyhow!("Failed to read config: {}", e))?;
            (config.rpc_url.clone(), config.rpc_secret.clone())
        };
        let mut params = Vec::new();
        if let Some(secret) = &rpc_secret {
            params.push(serde_json::json!(format!("token:{}", secret)));
        }
        params.push(serde_json::json!(gid));
        params.push(options);

        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "aria2-mcp",
            "method": "aria2.changeOption",
            "params": params,
        });

        let resp = self.client.post(&rpc_url).json(&body).send().await?;

        let res: serde_json::Value = resp.json().await?;

        if let Some(err) = res.get("error") {
            return Err(anyhow::anyhow!("aria2 error: {}", err));
        }

        Ok(())
    }

    pub async fn pause(&self, gid: &str) -> Result<()> {
        let (rpc_url, rpc_secret) = {
            let config = self
                .config
                .read()
                .map_err(|e| anyhow::anyhow!("Failed to read config: {}", e))?;
            (config.rpc_url.clone(), config.rpc_secret.clone())
        };
        let mut params = Vec::new();
        if let Some(secret) = &rpc_secret {
            params.push(serde_json::json!(format!("token:{}", secret)));
        }
        params.push(serde_json::json!(gid));

        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "aria2-mcp",
            "method": "aria2.pause",
            "params": params,
        });

        let resp = self.client.post(&rpc_url).json(&body).send().await?;

        let res: serde_json::Value = resp.json().await?;

        if let Some(err) = res.get("error") {
            return Err(anyhow::anyhow!("aria2 error: {}", err));
        }

        Ok(())
    }

    pub async fn force_pause(&self, gid: &str) -> Result<()> {
        let (rpc_url, rpc_secret) = {
            let config = self
                .config
                .read()
                .map_err(|e| anyhow::anyhow!("Failed to read config: {}", e))?;
            (config.rpc_url.clone(), config.rpc_secret.clone())
        };
        let mut params = Vec::new();
        if let Some(secret) = &rpc_secret {
            params.push(serde_json::json!(format!("token:{}", secret)));
        }
        params.push(serde_json::json!(gid));

        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "aria2-mcp",
            "method": "aria2.forcePause",
            "params": params,
        });

        let resp = self.client.post(&rpc_url).json(&body).send().await?;

        let res: serde_json::Value = resp.json().await?;

        if let Some(err) = res.get("error") {
            return Err(anyhow::anyhow!("aria2 error: {}", err));
        }

        Ok(())
    }

    pub async fn unpause(&self, gid: &str) -> Result<()> {
        let (rpc_url, rpc_secret) = {
            let config = self
                .config
                .read()
                .map_err(|e| anyhow::anyhow!("Failed to read config: {}", e))?;
            (config.rpc_url.clone(), config.rpc_secret.clone())
        };
        let mut params = Vec::new();
        if let Some(secret) = &rpc_secret {
            params.push(serde_json::json!(format!("token:{}", secret)));
        }
        params.push(serde_json::json!(gid));

        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "aria2-mcp",
            "method": "aria2.unpause",
            "params": params,
        });

        let resp = self.client.post(&rpc_url).json(&body).send().await?;

        let res: serde_json::Value = resp.json().await?;

        if let Some(err) = res.get("error") {
            return Err(anyhow::anyhow!("aria2 error: {}", err));
        }

        Ok(())
    }

    pub async fn remove(&self, gid: &str) -> Result<()> {
        let (rpc_url, rpc_secret) = {
            let config = self
                .config
                .read()
                .map_err(|e| anyhow::anyhow!("Failed to read config: {}", e))?;
            (config.rpc_url.clone(), config.rpc_secret.clone())
        };
        let mut params = Vec::new();
        if let Some(secret) = &rpc_secret {
            params.push(serde_json::json!(format!("token:{}", secret)));
        }
        params.push(serde_json::json!(gid));

        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "aria2-mcp",
            "method": "aria2.remove",
            "params": params,
        });

        let resp = self.client.post(&rpc_url).json(&body).send().await?;

        let res: serde_json::Value = resp.json().await?;

        if let Some(err) = res.get("error") {
            return Err(anyhow::anyhow!("aria2 error: {}", err));
        }

        Ok(())
    }

    pub async fn force_remove(&self, gid: &str) -> Result<()> {
        let (rpc_url, rpc_secret) = {
            let config = self
                .config
                .read()
                .map_err(|e| anyhow::anyhow!("Failed to read config: {}", e))?;
            (config.rpc_url.clone(), config.rpc_secret.clone())
        };
        let mut params = Vec::new();
        if let Some(secret) = &rpc_secret {
            params.push(serde_json::json!(format!("token:{}", secret)));
        }
        params.push(serde_json::json!(gid));

        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "aria2-mcp",
            "method": "aria2.forceRemove",
            "params": params,
        });

        let resp = self.client.post(&rpc_url).json(&body).send().await?;

        let res: serde_json::Value = resp.json().await?;

        if let Some(err) = res.get("error") {
            return Err(anyhow::anyhow!("aria2 error: {}", err));
        }

        Ok(())
    }

    pub async fn move_position(&self, gid: &str, pos: i32, how: &str) -> Result<i32> {
        let (rpc_url, rpc_secret) = {
            let config = self
                .config
                .read()
                .map_err(|e| anyhow::anyhow!("Failed to read config: {}", e))?;
            (config.rpc_url.clone(), config.rpc_secret.clone())
        };
        let mut params = Vec::new();
        if let Some(secret) = &rpc_secret {
            params.push(serde_json::json!(format!("token:{}", secret)));
        }
        params.push(serde_json::json!(gid));
        params.push(serde_json::json!(pos));
        params.push(serde_json::json!(how));

        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "aria2-mcp",
            "method": "aria2.changePosition",
            "params": params,
        });

        let resp = self.client.post(&rpc_url).json(&body).send().await?;

        let res: serde_json::Value = resp.json().await?;

        if let Some(err) = res.get("error") {
            return Err(anyhow::anyhow!("aria2 error: {}", err));
        }

        let new_pos = res["result"]
            .as_i64()
            .ok_or_else(|| anyhow::anyhow!("Failed to get new position from response"))?;

        Ok(new_pos as i32)
    }

    pub async fn tell_status(&self, gid: &str) -> Result<serde_json::Value> {
        let (rpc_url, rpc_secret) = {
            let config = self
                .config
                .read()
                .map_err(|e| anyhow::anyhow!("Failed to read config: {}", e))?;
            (config.rpc_url.clone(), config.rpc_secret.clone())
        };
        let mut params = Vec::new();
        if let Some(secret) = &rpc_secret {
            params.push(serde_json::json!(format!("token:{}", secret)));
        }
        params.push(serde_json::json!(gid));

        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "aria2-mcp",
            "method": "aria2.tellStatus",
            "params": params,
        });

        let resp = self.client.post(&rpc_url).json(&body).send().await?;

        let res: serde_json::Value = resp.json().await?;

        if let Some(err) = res.get("error") {
            return Err(anyhow::anyhow!("aria2 error: {}", err));
        }

        Ok(res["result"].clone())
    }

    pub async fn add_uri(
        &self,
        uris: Vec<String>,
        options: Option<serde_json::Value>,
    ) -> Result<String> {
        let (rpc_url, rpc_secret) = {
            let config = self
                .config
                .read()
                .map_err(|e| anyhow::anyhow!("Failed to read config: {}", e))?;
            (config.rpc_url.clone(), config.rpc_secret.clone())
        };
        let mut params = Vec::new();
        if let Some(secret) = &rpc_secret {
            params.push(serde_json::json!(format!("token:{}", secret)));
        }
        params.push(serde_json::json!(uris));
        if let Some(opts) = options {
            params.push(opts);
        }

        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "aria2-mcp",
            "method": "aria2.addUri",
            "params": params,
        });

        let resp = self.client.post(&rpc_url).json(&body).send().await?;

        let res: serde_json::Value = resp.json().await?;

        if let Some(err) = res.get("error") {
            return Err(anyhow::anyhow!("aria2 error: {}", err));
        }

        let gid = res["result"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Failed to get GID from response"))?;

        Ok(gid.to_string())
    }

    pub async fn get_version(&self) -> Result<String> {
        let (rpc_url, rpc_secret) = {
            let config = self
                .config
                .read()
                .map_err(|e| anyhow::anyhow!("Failed to read config: {}", e))?;
            (config.rpc_url.clone(), config.rpc_secret.clone())
        };
        let mut params = Vec::new();
        if let Some(secret) = &rpc_secret {
            params.push(serde_json::json!(format!("token:{}", secret)));
        }

        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "aria2-mcp",
            "method": "aria2.getVersion",
            "params": params,
        });

        let resp = self.client.post(&rpc_url).json(&body).send().await?;

        let res: serde_json::Value = resp.json().await?;

        if let Some(err) = res.get("error") {
            return Err(anyhow::anyhow!("aria2 error: {}", err));
        }

        let version = res["result"]["version"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Failed to get version from response"))?;

        Ok(version.to_string())
    }

    pub async fn get_files(&self, gid: &str) -> Result<serde_json::Value> {
        let (rpc_url, rpc_secret) = {
            let config = self
                .config
                .read()
                .map_err(|e| anyhow::anyhow!("Failed to read config: {}", e))?;
            (config.rpc_url.clone(), config.rpc_secret.clone())
        };
        let mut params = Vec::new();
        if let Some(secret) = &rpc_secret {
            params.push(serde_json::json!(format!("token:{}", secret)));
        }
        params.push(serde_json::json!(gid));

        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "aria2-mcp",
            "method": "aria2.getFiles",
            "params": params,
        });

        let resp = self.client.post(&rpc_url).json(&body).send().await?;

        let res: serde_json::Value = resp.json().await?;

        if let Some(err) = res.get("error") {
            return Err(anyhow::anyhow!("aria2 error: {}", err));
        }

        Ok(res["result"].clone())
    }

    pub async fn get_uris(&self, gid: &str) -> Result<serde_json::Value> {
        let (rpc_url, rpc_secret) = {
            let config = self
                .config
                .read()
                .map_err(|e| anyhow::anyhow!("Failed to read config: {}", e))?;
            (config.rpc_url.clone(), config.rpc_secret.clone())
        };
        let mut params = Vec::new();
        if let Some(secret) = &rpc_secret {
            params.push(serde_json::json!(format!("token:{}", secret)));
        }
        params.push(serde_json::json!(gid));

        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "aria2-mcp",
            "method": "aria2.getUris",
            "params": params,
        });

        let resp = self.client.post(&rpc_url).json(&body).send().await?;

        let res: serde_json::Value = resp.json().await?;

        if let Some(err) = res.get("error") {
            return Err(anyhow::anyhow!("aria2 error: {}", err));
        }

        Ok(res["result"].clone())
    }

    pub async fn get_peers(&self, gid: &str) -> Result<serde_json::Value> {
        let (rpc_url, rpc_secret) = {
            let config = self
                .config
                .read()
                .map_err(|e| anyhow::anyhow!("Failed to read config: {}", e))?;
            (config.rpc_url.clone(), config.rpc_secret.clone())
        };
        let mut params = Vec::new();
        if let Some(secret) = &rpc_secret {
            params.push(serde_json::json!(format!("token:{}", secret)));
        }
        params.push(serde_json::json!(gid));

        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "aria2-mcp",
            "method": "aria2.getPeers",
            "params": params,
        });

        let resp = self.client.post(&rpc_url).json(&body).send().await?;

        let res: serde_json::Value = resp.json().await?;

        if let Some(err) = res.get("error") {
            return Err(anyhow::anyhow!("aria2 error: {}", err));
        }

        Ok(res["result"].clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Config;

    #[tokio::test]
    async fn test_new_client() {
        let config = Config::default();
        let client = Aria2Client::new(config);

        // Test with invalid URL to verify error handling
        let result = client.get_version().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_tell_active_error() {
        let config = Config::default();
        let client = Aria2Client::new(config);
        let result = client.tell_active(None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_global_stat_error() {
        let config = Config::default();
        let client = Aria2Client::new(config);
        let result = client.get_global_stat().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_tell_waiting_error() {
        let config = Config::default();
        let client = Aria2Client::new(config);
        let result = client.tell_waiting(0, 10, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_tell_stopped_error() {
        let config = Config::default();
        let client = Aria2Client::new(config);
        let result = client.tell_stopped(0, 10, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_global_option_error() {
        let config = Config::default();
        let client = Aria2Client::new(config);
        let result = client.get_global_option().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_option_error() {
        let config = Config::default();
        let client = Aria2Client::new(config);
        let result = client.get_option("dummy").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_change_global_option_error() {
        let config = Config::default();
        let client = Aria2Client::new(config);
        let result = client.change_global_option(serde_json::json!({})).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_change_option_error() {
        let config = Config::default();
        let client = Aria2Client::new(config);
        let result = client.change_option("dummy", serde_json::json!({})).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_pause_error() {
        let config = Config::default();
        let client = Aria2Client::new(config);
        let result = client.pause("dummy").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_force_pause_error() {
        let config = Config::default();
        let client = Aria2Client::new(config);
        let result = client.force_pause("dummy").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_unpause_error() {
        let config = Config::default();
        let client = Aria2Client::new(config);
        let result = client.unpause("dummy").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_remove_error() {
        let config = Config::default();
        let client = Aria2Client::new(config);
        let result = client.remove("dummy").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_force_remove_error() {
        let config = Config::default();
        let client = Aria2Client::new(config);
        let result = client.force_remove("dummy").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_move_position_error() {
        let config = Config::default();
        let client = Aria2Client::new(config);
        let result = client.move_position("dummy", 0, "dummy").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_tell_status_error() {
        let config = Config::default();
        let client = Aria2Client::new(config);
        let result = client.tell_status("dummy").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_add_uri_error() {
        let config = Config::default();
        let client = Aria2Client::new(config);
        let result = client.add_uri(vec!["dummy".to_string()], None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_files_error() {
        let config = Config::default();
        let client = Aria2Client::new(config);
        let result = client.get_files("dummy").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_peers_error() {
        let config = Config::default();
        let client = Aria2Client::new(config);
        let result = client.get_peers("dummy").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_uris_error() {
        let config = Config::default();
        let client = Aria2Client::new(config);
        let result = client.get_uris("dummy").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_new_client_with_secret() {
        let config = Config {
            rpc_secret: Some("secret".to_string()),
            ..Config::default()
        };
        let client = Aria2Client::new(config);
        let result = client.get_version().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_tell_active_with_keys_error() {
        let config = Config::default();
        let client = Aria2Client::new(config);
        let result = client.tell_active(Some(vec!["gid".to_string()])).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_tell_waiting_with_keys_error() {
        let config = Config::default();
        let client = Aria2Client::new(config);
        let result = client
            .tell_waiting(0, 10, Some(vec!["gid".to_string()]))
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_tell_stopped_with_keys_error() {
        let config = Config::default();
        let client = Aria2Client::new(config);
        let result = client
            .tell_stopped(0, 10, Some(vec!["gid".to_string()]))
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_move_position_how_error() {
        let config = Config::default();
        let client = Aria2Client::new(config);
        let result = client.move_position("dummy", 0, "POS_SET").await;
        assert!(result.is_err());
    }
}
