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

    pub async fn tell_active(&self, keys: Option<Vec<String>>) -> Result<serde_json::Value> {
        let mut params = Vec::new();
        if let Some(secret) = &self.config.rpc_secret {
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

        let resp = self
            .client
            .post(&self.config.rpc_url)
            .json(&body)
            .send()
            .await?;

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
        let mut params = Vec::new();
        if let Some(secret) = &self.config.rpc_secret {
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

        let resp = self
            .client
            .post(&self.config.rpc_url)
            .json(&body)
            .send()
            .await?;

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
        let mut params = Vec::new();
        if let Some(secret) = &self.config.rpc_secret {
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

        let resp = self
            .client
            .post(&self.config.rpc_url)
            .json(&body)
            .send()
            .await?;

        let res: serde_json::Value = resp.json().await?;

        if let Some(err) = res.get("error") {
            return Err(anyhow::anyhow!("aria2 error: {}", err));
        }

        Ok(res["result"].clone())
    }

    pub async fn get_global_stat(&self) -> Result<serde_json::Value> {
        let mut params = Vec::new();
        if let Some(secret) = &self.config.rpc_secret {
            params.push(serde_json::json!(format!("token:{}", secret)));
        }

        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "aria2-mcp",
            "method": "aria2.getGlobalStat",
            "params": params,
        });

        let resp = self
            .client
            .post(&self.config.rpc_url)
            .json(&body)
            .send()
            .await?;

        let res: serde_json::Value = resp.json().await?;

        if let Some(err) = res.get("error") {
            return Err(anyhow::anyhow!("aria2 error: {}", err));
        }

        Ok(res["result"].clone())
    }

    pub async fn get_global_option(&self) -> Result<serde_json::Value> {
        let mut params = Vec::new();
        if let Some(secret) = &self.config.rpc_secret {
            params.push(serde_json::json!(format!("token:{}", secret)));
        }

        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "aria2-mcp",
            "method": "aria2.getGlobalOption",
            "params": params,
        });

        let resp = self
            .client
            .post(&self.config.rpc_url)
            .json(&body)
            .send()
            .await?;

        let res: serde_json::Value = resp.json().await?;

        if let Some(err) = res.get("error") {
            return Err(anyhow::anyhow!("aria2 error: {}", err));
        }

        Ok(res["result"].clone())
    }

    pub async fn get_option(&self, gid: &str) -> Result<serde_json::Value> {
        let mut params = Vec::new();
        if let Some(secret) = &self.config.rpc_secret {
            params.push(serde_json::json!(format!("token:{}", secret)));
        }
        params.push(serde_json::json!(gid));

        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "aria2-mcp",
            "method": "aria2.getOption",
            "params": params,
        });

        let resp = self
            .client
            .post(&self.config.rpc_url)
            .json(&body)
            .send()
            .await?;

        let res: serde_json::Value = resp.json().await?;

        if let Some(err) = res.get("error") {
            return Err(anyhow::anyhow!("aria2 error: {}", err));
        }

        Ok(res["result"].clone())
    }

    pub async fn change_global_option(&self, options: serde_json::Value) -> Result<()> {
        let mut params = Vec::new();
        if let Some(secret) = &self.config.rpc_secret {
            params.push(serde_json::json!(format!("token:{}", secret)));
        }
        params.push(options);

        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "aria2-mcp",
            "method": "aria2.changeGlobalOption",
            "params": params,
        });

        let resp = self
            .client
            .post(&self.config.rpc_url)
            .json(&body)
            .send()
            .await?;

        let res: serde_json::Value = resp.json().await?;

        if let Some(err) = res.get("error") {
            return Err(anyhow::anyhow!("aria2 error: {}", err));
        }

        Ok(())
    }

    pub async fn change_option(&self, gid: &str, options: serde_json::Value) -> Result<()> {
        let mut params = Vec::new();
        if let Some(secret) = &self.config.rpc_secret {
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

        let resp = self
            .client
            .post(&self.config.rpc_url)
            .json(&body)
            .send()
            .await?;

        let res: serde_json::Value = resp.json().await?;

        if let Some(err) = res.get("error") {
            return Err(anyhow::anyhow!("aria2 error: {}", err));
        }

        Ok(())
    }

    pub async fn pause(&self, gid: &str) -> Result<()> {
        let mut params = Vec::new();
        if let Some(secret) = &self.config.rpc_secret {
            params.push(serde_json::json!(format!("token:{}", secret)));
        }
        params.push(serde_json::json!(gid));

        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "aria2-mcp",
            "method": "aria2.pause",
            "params": params,
        });

        let resp = self
            .client
            .post(&self.config.rpc_url)
            .json(&body)
            .send()
            .await?;

        let res: serde_json::Value = resp.json().await?;

        if let Some(err) = res.get("error") {
            return Err(anyhow::anyhow!("aria2 error: {}", err));
        }

        Ok(())
    }

    pub async fn force_pause(&self, gid: &str) -> Result<()> {
        let mut params = Vec::new();
        if let Some(secret) = &self.config.rpc_secret {
            params.push(serde_json::json!(format!("token:{}", secret)));
        }
        params.push(serde_json::json!(gid));

        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "aria2-mcp",
            "method": "aria2.forcePause",
            "params": params,
        });

        let resp = self
            .client
            .post(&self.config.rpc_url)
            .json(&body)
            .send()
            .await?;

        let res: serde_json::Value = resp.json().await?;

        if let Some(err) = res.get("error") {
            return Err(anyhow::anyhow!("aria2 error: {}", err));
        }

        Ok(())
    }

    pub async fn unpause(&self, gid: &str) -> Result<()> {
        let mut params = Vec::new();
        if let Some(secret) = &self.config.rpc_secret {
            params.push(serde_json::json!(format!("token:{}", secret)));
        }
        params.push(serde_json::json!(gid));

        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "aria2-mcp",
            "method": "aria2.unpause",
            "params": params,
        });

        let resp = self
            .client
            .post(&self.config.rpc_url)
            .json(&body)
            .send()
            .await?;

        let res: serde_json::Value = resp.json().await?;

        if let Some(err) = res.get("error") {
            return Err(anyhow::anyhow!("aria2 error: {}", err));
        }

        Ok(())
    }

    pub async fn remove(&self, gid: &str) -> Result<()> {
        let mut params = Vec::new();
        if let Some(secret) = &self.config.rpc_secret {
            params.push(serde_json::json!(format!("token:{}", secret)));
        }
        params.push(serde_json::json!(gid));

        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "aria2-mcp",
            "method": "aria2.remove",
            "params": params,
        });

        let resp = self
            .client
            .post(&self.config.rpc_url)
            .json(&body)
            .send()
            .await?;

        let res: serde_json::Value = resp.json().await?;

        if let Some(err) = res.get("error") {
            return Err(anyhow::anyhow!("aria2 error: {}", err));
        }

        Ok(())
    }

    pub async fn force_remove(&self, gid: &str) -> Result<()> {
        let mut params = Vec::new();
        if let Some(secret) = &self.config.rpc_secret {
            params.push(serde_json::json!(format!("token:{}", secret)));
        }
        params.push(serde_json::json!(gid));

        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "aria2-mcp",
            "method": "aria2.forceRemove",
            "params": params,
        });

        let resp = self
            .client
            .post(&self.config.rpc_url)
            .json(&body)
            .send()
            .await?;

        let res: serde_json::Value = resp.json().await?;

        if let Some(err) = res.get("error") {
            return Err(anyhow::anyhow!("aria2 error: {}", err));
        }

        Ok(())
    }

    pub async fn move_position(&self, gid: &str, pos: i32, how: &str) -> Result<i32> {
        let mut params = Vec::new();
        if let Some(secret) = &self.config.rpc_secret {
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

        let resp = self
            .client
            .post(&self.config.rpc_url)
            .json(&body)
            .send()
            .await?;

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
        let mut params = Vec::new();
        if let Some(secret) = &self.config.rpc_secret {
            params.push(serde_json::json!(format!("token:{}", secret)));
        }
        params.push(serde_json::json!(gid));

        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "aria2-mcp",
            "method": "aria2.tellStatus",
            "params": params,
        });

        let resp = self
            .client
            .post(&self.config.rpc_url)
            .json(&body)
            .send()
            .await?;

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
        let mut params = Vec::new();
        if let Some(secret) = &self.config.rpc_secret {
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

        let resp = self
            .client
            .post(&self.config.rpc_url)
            .json(&body)
            .send()
            .await?;

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

        let resp = self
            .client
            .post(&self.config.rpc_url)
            .json(&body)
            .send()
            .await?;

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
        let mut params = Vec::new();
        if let Some(secret) = &self.config.rpc_secret {
            params.push(serde_json::json!(format!("token:{}", secret)));
        }
        params.push(serde_json::json!(gid));

        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "aria2-mcp",
            "method": "aria2.getFiles",
            "params": params,
        });

        let resp = self
            .client
            .post(&self.config.rpc_url)
            .json(&body)
            .send()
            .await?;

        let res: serde_json::Value = resp.json().await?;

        if let Some(err) = res.get("error") {
            return Err(anyhow::anyhow!("aria2 error: {}", err));
        }

        Ok(res["result"].clone())
    }

    pub async fn get_uris(&self, gid: &str) -> Result<serde_json::Value> {
        let mut params = Vec::new();
        if let Some(secret) = &self.config.rpc_secret {
            params.push(serde_json::json!(format!("token:{}", secret)));
        }
        params.push(serde_json::json!(gid));

        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "aria2-mcp",
            "method": "aria2.getUris",
            "params": params,
        });

        let resp = self
            .client
            .post(&self.config.rpc_url)
            .json(&body)
            .send()
            .await?;

        let res: serde_json::Value = resp.json().await?;

        if let Some(err) = res.get("error") {
            return Err(anyhow::anyhow!("aria2 error: {}", err));
        }

        Ok(res["result"].clone())
    }
}
