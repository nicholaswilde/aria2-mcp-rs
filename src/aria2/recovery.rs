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
}

impl Default for ErrorAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
