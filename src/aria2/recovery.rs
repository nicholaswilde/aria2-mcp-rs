use crate::aria2::Aria2Client;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub initial_backoff_secs: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_backoff_secs: 5,
        }
    }
}

pub struct ErrorAnalyzer;

impl ErrorAnalyzer {
    pub fn new() -> Self {
        Self
    }

    pub fn is_retryable(&self, error_code: &str) -> bool {
        matches!(error_code, "2" | "6" | "17" | "20" | "21" | "22")
    }

    pub fn should_retry(&self, status: &serde_json::Value) -> bool {
        if let Some(s) = status.get("status").and_then(|v| v.as_str()) {
            if s == "error" {
                if let Some(code) = status.get("errorCode").and_then(|v| v.as_str()) {
                    return self.is_retryable(code);
                }
            }
        }
        false
    }
}

impl Default for ErrorAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

pub struct RecoveryManager {
    config: RetryConfig,
    analyzer: ErrorAnalyzer,
    retry_counts: Arc<RwLock<HashMap<String, u32>>>,
    pending_retries: Arc<RwLock<HashSet<String>>>,
}

impl RecoveryManager {
    pub fn new(config: RetryConfig) -> Self {
        Self {
            config,
            analyzer: ErrorAnalyzer::new(),
            retry_counts: Arc::new(RwLock::new(HashMap::new())),
            pending_retries: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    pub async fn analyze_and_get_retry_backoff(
        &self,
        gid: &str,
        status: &serde_json::Value,
    ) -> Option<u64> {
        if !self.analyzer.should_retry(status) {
            return None;
        }

        // Check if already pending
        {
            let pending = self.pending_retries.read().await;
            if pending.contains(gid) {
                return None;
            }
        }

        let mut counts = self.retry_counts.write().await;
        let count = counts.entry(gid.to_string()).or_insert(0);

        if *count >= self.config.max_retries {
            return None;
        }

        *count += 1;
        let backoff = self.config.initial_backoff_secs * (2u64.pow(*count - 1));

        // Mark as pending
        {
            let mut pending = self.pending_retries.write().await;
            pending.insert(gid.to_string());
        }

        Some(backoff)
    }

    pub async fn perform_retry(&self, client: &Aria2Client, gid: &str) -> anyhow::Result<String> {
        log::info!("Attempting retry for download {}...", gid);

        // 1. Get URIs and options from old download
        let status = client.tell_status(gid).await?;
        let uris: Vec<String> = status
            .get("files")
            .and_then(|f| f.as_array())
            .and_then(|files| files.first())
            .and_then(|file| file.get("uris"))
            .and_then(|u| u.as_array())
            .map(|uris| {
                uris.iter()
                    .filter_map(|u| u.get("uri").and_then(|v| v.as_str().map(|s| s.to_string())))
                    .collect()
            })
            .unwrap_or_default();

        if uris.is_empty() {
            let res = Err(anyhow::anyhow!("No URIs found for download {}", gid));
            let mut pending = self.pending_retries.write().await;
            pending.remove(gid);
            return res;
        }

        // 2. Add new download
        let new_gid = match client.add_uri(uris, None).await {
            Ok(ng) => ng,
            Err(e) => {
                let mut pending = self.pending_retries.write().await;
                pending.remove(gid);
                return Err(e);
            }
        };

        log::info!("Retry successful. New GID: {}", new_gid);

        // 3. Cleanup old download result (optional but recommended)
        // If we don't remove it, it stays in the stopped list.
        // Let's try to remove it.
        let _ = client.remove(gid).await;

        // 4. Transfer retry count to new GID and cleanup
        {
            let mut counts = self.retry_counts.write().await;
            if let Some(count) = counts.remove(gid) {
                counts.insert(new_gid.clone(), count);
            }

            let mut pending = self.pending_retries.write().await;
            pending.remove(gid);
        }

        Ok(new_gid)
    }

    pub async fn clear_retry_count(&self, gid: &str) {
        let mut counts = self.retry_counts.write().await;
        counts.remove(gid);
    }

    pub async fn inject_trackers(
        &self,
        client: &Aria2Client,
        gid: &str,
        trackers: Vec<String>,
    ) -> anyhow::Result<()> {
        log::info!("Injecting {} trackers into download {}...", trackers.len(), gid);
        let trackers_str = trackers.join(",");
        let options = serde_json::json!({
            "bt-tracker": trackers_str
        });
        client.change_option(gid, options).await?;
        Ok(())
    }
}

pub struct TrackerScraper {
    url: String,
}

impl TrackerScraper {
    pub fn new(url: String) -> Self {
        Self { url }
    }

    pub async fn fetch_trackers(&self) -> anyhow::Result<Vec<String>> {
        let resp = reqwest::get(&self.url).await?.text().await?;
        let trackers: Vec<String> = resp
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect();
        Ok(trackers)
    }
}
