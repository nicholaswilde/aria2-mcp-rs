use std::collections::HashMap;
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
}

impl RecoveryManager {
    pub fn new(config: RetryConfig) -> Self {
        Self {
            config,
            analyzer: ErrorAnalyzer::new(),
            retry_counts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn analyze_and_get_retry_backoff(&self, gid: &str, status: &serde_json::Value) -> Option<u64> {
        if !self.analyzer.should_retry(status) {
            return None;
        }

        let mut counts = self.retry_counts.write().await;
        let count = counts.entry(gid.to_string()).or_insert(0);
        
        if *count >= self.config.max_retries {
            return None;
        }

        *count += 1;
        let backoff = self.config.initial_backoff_secs * (2u64.pow(*count - 1));
        Some(backoff)
    }

    pub async fn clear_retry_count(&self, gid: &str) {
        let mut counts = self.retry_counts.write().await;
        counts.remove(gid);
    }
}
