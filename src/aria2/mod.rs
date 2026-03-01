use crate::Config;
use anyhow::Result;
use futures_util::StreamExt;
use reqwest::Client;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::RwLock;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

pub mod notifications;
pub mod recovery;

pub use notifications::{Aria2Event, Aria2Notification};

#[allow(dead_code)]
#[derive(Clone)]
pub struct Aria2Client {
    config: Arc<RwLock<Config>>,
    client: Client,
    pub name: String,
    pub state_manager: Arc<crate::state::StateManager>,
}

impl Aria2Client {
    pub async fn connect_notifications(
        &self,
    ) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>> {
        let url = self.ws_url().await?;
        let timeout_secs = {
            let config = self.config.read().await;
            config.rpc_timeout_secs
        };

        let (ws_stream, _) = tokio::time::timeout(
            tokio::time::Duration::from_secs(timeout_secs),
            connect_async(url),
        )
        .await
        .map_err(|_| anyhow::anyhow!("Connection timeout"))??;

        Ok(ws_stream)
    }

    pub fn start_notifications(
        &self,
        tx: tokio::sync::mpsc::Sender<Aria2Notification>,
    ) -> Result<()> {
        let client = self.clone();
        tokio::spawn(async move {
            let mut backoff = tokio::time::Duration::from_secs(1);
            let max_backoff = tokio::time::Duration::from_secs(60);

            loop {
                match client.connect_notifications().await {
                    Ok(mut ws_stream) => {
                        log::info!("Connected to aria2 WebSocket for instance: {}", client.name);
                        backoff = tokio::time::Duration::from_secs(1); // Reset backoff on success
                        while let Some(msg) = ws_stream.next().await {
                            match msg {
                                Ok(tokio_tungstenite::tungstenite::Message::Text(text)) => {
                                    if let Ok(notification) =
                                        serde_json::from_str::<Aria2Notification>(&text)
                                    {
                                        if tx.send(notification).await.is_err() {
                                            log::error!("Notification channel closed");
                                            return;
                                        }
                                    }
                                }
                                Ok(tokio_tungstenite::tungstenite::Message::Close(_)) => break,
                                Err(e) => {
                                    log::error!("WebSocket error: {e}");
                                    break;
                                }
                                _ => {}
                            }
                        }
                    }
                    Err(e) => {
                        log::error!(
                            "Failed to connect to aria2 WebSocket for {}: {e}. Retrying in {backoff:?}...",
                            client.name
                        );
                    }
                }
                tokio::time::sleep(backoff).await;
                backoff = std::cmp::min(backoff * 2, max_backoff);
            }
        });
        Ok(())
    }

    #[must_use]
    pub fn new(config: Config) -> Self {
        let client = Client::builder()
            .danger_accept_invalid_certs(config.no_verify_ssl)
            .timeout(std::time::Duration::from_secs(config.rpc_timeout_secs))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            config: Arc::new(RwLock::new(config)),
            client,
            name: "default".to_string(),
            state_manager: Arc::new(crate::state::StateManager::new(std::path::PathBuf::from(
                "aria2_mcp_state.json",
            ))),
        }
    }

    #[must_use]
    pub fn new_with_instance(config: Config, instance: crate::config::Aria2Instance) -> Self {
        let mut config = config;
        config.rpc_url = instance.rpc_url;
        config.rpc_secret = instance.rpc_secret;
        let name = instance.name.clone();

        let client = Client::builder()
            .danger_accept_invalid_certs(config.no_verify_ssl)
            .timeout(std::time::Duration::from_secs(config.rpc_timeout_secs))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            config: Arc::new(RwLock::new(config)),
            client,
            name,
            state_manager: Arc::new(crate::state::StateManager::new(std::path::PathBuf::from(
                "aria2_mcp_state.json",
            ))),
        }
    }

    #[must_use]
    pub fn config(&self) -> Arc<RwLock<Config>> {
        Arc::clone(&self.config)
    }

    #[must_use]
    pub fn state_manager(&self) -> Arc<crate::state::StateManager> {
        Arc::clone(&self.state_manager)
    }

    pub async fn save_state(&self) -> Result<()> {
        let state_data = {
            let config_guard = self.config.read().await;
            crate::state::StateData {
                bandwidth_profiles: config_guard.bandwidth_profiles.clone(),
                bandwidth_schedules: config_guard.bandwidth_schedules.clone(),
                organize_rules: config_guard.organize_rules.clone(),
                rules: std::collections::HashMap::new(),
            }
        };
        self.state_manager
            .save(&state_data)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to save state: {e}"))?;
        Ok(())
    }

    pub async fn ws_url(&self) -> Result<String> {
        let rpc_url = {
            let config = self.config.read().await;
            config.rpc_url.clone()
        };

        if rpc_url.starts_with("http://") {
            Ok(rpc_url.replace("http://", "ws://"))
        } else if rpc_url.starts_with("https://") {
            Ok(rpc_url.replace("https://", "wss://"))
        } else {
            Err(anyhow::anyhow!("Invalid RPC URL protocol: {rpc_url}"))
        }
    }

    pub async fn tell_active(&self, keys: Option<Vec<String>>) -> Result<serde_json::Value> {
        let (rpc_url, rpc_secret) = {
            let config = self.config.read().await;
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
            return Err(anyhow::anyhow!("aria2 error: {err}"));
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
            let config = self.config.read().await;
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
            return Err(anyhow::anyhow!("aria2 error: {err}"));
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
            let config = self.config.read().await;
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
            return Err(anyhow::anyhow!("aria2 error: {err}"));
        }

        Ok(res["result"].clone())
    }

    pub async fn get_global_stat(&self) -> Result<serde_json::Value> {
        let (rpc_url, rpc_secret) = {
            let config = self.config.read().await;
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
            return Err(anyhow::anyhow!("aria2 error: {err}"));
        }

        Ok(res["result"].clone())
    }

    pub async fn get_global_option(&self) -> Result<serde_json::Value> {
        let (rpc_url, rpc_secret) = {
            let config = self.config.read().await;
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
            return Err(anyhow::anyhow!("aria2 error: {err}"));
        }

        Ok(res["result"].clone())
    }

    pub async fn get_option(&self, gid: &str) -> Result<serde_json::Value> {
        let (rpc_url, rpc_secret) = {
            let config = self.config.read().await;
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
            return Err(anyhow::anyhow!("aria2 error: {err}"));
        }

        Ok(res["result"].clone())
    }

    pub async fn change_global_option(&self, options: serde_json::Value) -> Result<()> {
        let (rpc_url, rpc_secret) = {
            let config = self.config.read().await;
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
            return Err(anyhow::anyhow!("aria2 error: {err}"));
        }

        Ok(())
    }

    pub async fn change_option(&self, gid: &str, options: serde_json::Value) -> Result<()> {
        let (rpc_url, rpc_secret) = {
            let config = self.config.read().await;
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
            return Err(anyhow::anyhow!("aria2 error: {err}"));
        }

        Ok(())
    }

    pub async fn pause(&self, gid: &str) -> Result<()> {
        let (rpc_url, rpc_secret) = {
            let config = self.config.read().await;
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
            return Err(anyhow::anyhow!("aria2 error: {err}"));
        }

        Ok(())
    }

    pub async fn force_pause(&self, gid: &str) -> Result<()> {
        let (rpc_url, rpc_secret) = {
            let config = self.config.read().await;
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
            return Err(anyhow::anyhow!("aria2 error: {err}"));
        }

        Ok(())
    }

    pub async fn unpause(&self, gid: &str) -> Result<()> {
        let (rpc_url, rpc_secret) = {
            let config = self.config.read().await;
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
            return Err(anyhow::anyhow!("aria2 error: {err}"));
        }

        Ok(())
    }

    pub async fn remove(&self, gid: &str) -> Result<()> {
        let (rpc_url, rpc_secret) = {
            let config = self.config.read().await;
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
            return Err(anyhow::anyhow!("aria2 error: {err}"));
        }

        Ok(())
    }

    pub async fn force_remove(&self, gid: &str) -> Result<()> {
        let (rpc_url, rpc_secret) = {
            let config = self.config.read().await;
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
            return Err(anyhow::anyhow!("aria2 error: {err}"));
        }

        Ok(())
    }

    pub async fn move_position(&self, gid: &str, pos: i32, how: &str) -> Result<i32> {
        let (rpc_url, rpc_secret) = {
            let config = self.config.read().await;
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
            return Err(anyhow::anyhow!("aria2 error: {err}"));
        }

        let new_pos = res["result"]
            .as_i64()
            .ok_or_else(|| anyhow::anyhow!("Failed to get new position from response"))?;

        Ok(new_pos as i32)
    }

    pub async fn tell_status(&self, gid: &str) -> Result<serde_json::Value> {
        let (rpc_url, rpc_secret) = {
            let config = self.config.read().await;
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
            return Err(anyhow::anyhow!("aria2 error: {err}"));
        }

        Ok(res["result"].clone())
    }

    pub async fn add_uri(
        &self,
        uris: Vec<String>,
        options: Option<serde_json::Value>,
    ) -> Result<String> {
        let (rpc_url, rpc_secret) = {
            let config = self.config.read().await;
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
            return Err(anyhow::anyhow!("aria2 error: {err}"));
        }

        let gid = res["result"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Failed to get GID from response"))?;

        Ok(gid.to_string())
    }

    pub async fn get_version(&self) -> Result<String> {
        let (rpc_url, rpc_secret) = {
            let config = self.config.read().await;
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
            return Err(anyhow::anyhow!("aria2 error: {err}"));
        }

        let version = res["result"]["version"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Failed to get version from response"))?;

        Ok(version.to_string())
    }

    pub async fn get_files(&self, gid: &str) -> Result<serde_json::Value> {
        let (rpc_url, rpc_secret) = {
            let config = self.config.read().await;
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
            return Err(anyhow::anyhow!("aria2 error: {err}"));
        }

        Ok(res["result"].clone())
    }

    pub async fn get_uris(&self, gid: &str) -> Result<serde_json::Value> {
        let (rpc_url, rpc_secret) = {
            let config = self.config.read().await;
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
            return Err(anyhow::anyhow!("aria2 error: {err}"));
        }

        Ok(res["result"].clone())
    }

    pub async fn get_peers(&self, gid: &str) -> Result<serde_json::Value> {
        let (rpc_url, rpc_secret) = {
            let config = self.config.read().await;
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
            return Err(anyhow::anyhow!("aria2 error: {err}"));
        }

        Ok(res["result"].clone())
    }

    pub async fn pause_all(&self) -> Result<()> {
        let (rpc_url, rpc_secret) = {
            let config = self.config.read().await;
            (config.rpc_url.clone(), config.rpc_secret.clone())
        };
        let mut params = Vec::new();
        if let Some(secret) = &rpc_secret {
            params.push(serde_json::json!(format!("token:{}", secret)));
        }

        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "aria2-mcp",
            "method": "aria2.pauseAll",
            "params": params,
        });

        let resp = self.client.post(&rpc_url).json(&body).send().await?;
        let res: serde_json::Value = resp.json().await?;

        if let Some(err) = res.get("error") {
            return Err(anyhow::anyhow!("aria2 error: {err}"));
        }

        Ok(())
    }

    pub async fn unpause_all(&self) -> Result<()> {
        let (rpc_url, rpc_secret) = {
            let config = self.config.read().await;
            (config.rpc_url.clone(), config.rpc_secret.clone())
        };
        let mut params = Vec::new();
        if let Some(secret) = &rpc_secret {
            params.push(serde_json::json!(format!("token:{}", secret)));
        }

        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "aria2-mcp",
            "method": "aria2.unpauseAll",
            "params": params,
        });

        let resp = self.client.post(&rpc_url).json(&body).send().await?;
        let res: serde_json::Value = resp.json().await?;

        if let Some(err) = res.get("error") {
            return Err(anyhow::anyhow!("aria2 error: {err}"));
        }

        Ok(())
    }

    pub async fn purge_download_result(&self) -> Result<()> {
        let (rpc_url, rpc_secret) = {
            let config = self.config.read().await;
            (config.rpc_url.clone(), config.rpc_secret.clone())
        };
        let mut params = Vec::new();
        if let Some(secret) = &rpc_secret {
            params.push(serde_json::json!(format!("token:{}", secret)));
        }

        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "aria2-mcp",
            "method": "aria2.purgeDownloadResult",
            "params": params,
        });

        let resp = self.client.post(&rpc_url).json(&body).send().await?;
        let res: serde_json::Value = resp.json().await?;

        if let Some(err) = res.get("error") {
            return Err(anyhow::anyhow!("aria2 error: {err}"));
        }

        Ok(())
    }

    pub async fn remove_download_result(&self, gid: &str) -> Result<()> {
        let (rpc_url, rpc_secret) = {
            let config = self.config.read().await;
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
            "method": "aria2.removeDownloadResult",
            "params": params,
        });

        let resp = self.client.post(&rpc_url).json(&body).send().await?;
        let res: serde_json::Value = resp.json().await?;

        if let Some(err) = res.get("error") {
            return Err(anyhow::anyhow!("aria2 error: {err}"));
        }

        Ok(())
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

    #[tokio::test]
    async fn test_remove_download_result_error() {
        let config = Config::default();
        let client = Aria2Client::new(config);
        let result = client.remove_download_result("dummy").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_aria2_client_methods_success() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;
        let rpc_url = format!("{}/jsonrpc", mock_server.uri());

        let config = Config {
            rpc_url,
            ..Config::default()
        };
        let client = Aria2Client::new(config);

        // Mock getVersion
        Mock::given(method("POST"))
            .and(path("/jsonrpc"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "jsonrpc": "2.0",
                "id": "aria2-mcp",
                "result": { "version": "1.36.0" }
            })))
            .mount(&mock_server)
            .await;

        let version = client.get_version().await.unwrap();
        assert_eq!(version, "1.36.0");

        // Mock pauseAll
        Mock::given(method("POST"))
            .and(path("/jsonrpc"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "jsonrpc": "2.0",
                "id": "aria2-mcp",
                "result": "OK"
            })))
            .mount(&mock_server)
            .await;

        client.pause_all().await.unwrap();
        client.unpause_all().await.unwrap();
        client.purge_download_result().await.unwrap();
    }

    #[tokio::test]
    async fn test_ws_url_error() {
        let config = Config {
            rpc_url: "ftp://127.0.0.1:6800/jsonrpc".to_string(),
            ..Config::default()
        };
        let client = Aria2Client::new(config);
        assert!(client.ws_url().await.is_err());
    }

    #[tokio::test]
    async fn test_connect_notifications_error() {
        let config = Config {
            rpc_url: "http://localhost:1/jsonrpc".to_string(),
            ..Config::default()
        };
        let client = Aria2Client::new(config);
        let result = client.connect_notifications().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_aria2_client_rpc_error() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;
        let rpc_url = format!("{}/jsonrpc", mock_server.uri());

        let config = Config {
            rpc_url,
            ..Config::default()
        };
        let client = Aria2Client::new(config);

        // Mock error response
        Mock::given(method("POST"))
            .and(path("/jsonrpc"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "jsonrpc": "2.0",
                "id": "aria2-mcp",
                "error": { "code": 1, "message": "Some error" }
            })))
            .mount(&mock_server)
            .await;

        let result = client.get_version().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("aria2 error"));
    }

    #[tokio::test]
    async fn test_aria2_client_tell_stopped_success() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;
        let rpc_url = format!("{}/jsonrpc", mock_server.uri());

        let config = Config {
            rpc_url,
            ..Config::default()
        };
        let client = Aria2Client::new(config);

        Mock::given(method("POST"))
            .and(path("/jsonrpc"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "jsonrpc": "2.0",
                "id": "aria2-mcp",
                "result": []
            })))
            .mount(&mock_server)
            .await;

        let result = client.tell_stopped(0, 10, None).await.unwrap();
        assert!(result.is_array());
    }

    #[tokio::test]
    async fn test_aria2_client_tell_active_success() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;
        let rpc_url = format!("{}/jsonrpc", mock_server.uri());

        let config = Config {
            rpc_url,
            ..Config::default()
        };
        let client = Aria2Client::new(config);

        Mock::given(method("POST"))
            .and(path("/jsonrpc"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "jsonrpc": "2.0",
                "id": "aria2-mcp",
                "result": []
            })))
            .mount(&mock_server)
            .await;

        let result = client.tell_active(None).await.unwrap();
        assert!(result.is_array());
    }

    #[tokio::test]
    async fn test_aria2_client_get_option_success() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;
        let rpc_url = format!("{}/jsonrpc", mock_server.uri());

        let config = Config {
            rpc_url,
            ..Config::default()
        };
        let client = Aria2Client::new(config);

        Mock::given(method("POST"))
            .and(path("/jsonrpc"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "jsonrpc": "2.0",
                "id": "aria2-mcp",
                "result": { "dir": "/tmp" }
            })))
            .mount(&mock_server)
            .await;

        let result = client.get_option("dummy").await.unwrap();
        assert_eq!(result["dir"], "/tmp");
    }
}
