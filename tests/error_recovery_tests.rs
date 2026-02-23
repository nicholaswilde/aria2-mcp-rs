use aria2_mcp_rs::aria2::recovery::{ErrorAnalyzer, RetryConfig};

#[test]
fn test_error_analyzer_retryable() {
    let analyzer = ErrorAnalyzer::new();

    // Retryable
    assert!(analyzer.is_retryable("2")); // Timeout
    assert!(analyzer.is_retryable("6")); // Network problem
    assert!(analyzer.is_retryable("17")); // Name resolution failed
    assert!(analyzer.is_retryable("20")); // Bad HTTP response

    // Not retryable
    assert!(!analyzer.is_retryable("3")); // Not found
    assert!(!analyzer.is_retryable("9")); // Disk space
    assert!(!analyzer.is_retryable("13")); // File exists
    assert!(!analyzer.is_retryable("24")); // Auth failed
}

#[test]
fn test_retry_config_defaults() {
    let config = RetryConfig::default();
    assert_eq!(config.max_retries, 3);
    assert_eq!(config.initial_backoff_secs, 5);
}
