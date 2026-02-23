use aria2_mcp_rs::aria2::recovery::{ErrorAnalyzer, RecoveryManager, RetryConfig};
use aria2_mcp_rs::aria2::Aria2Client;
use aria2_mcp_rs::config::Config;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

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
fn test_error_analyzer_analyze_status() {
    let analyzer = ErrorAnalyzer::new();

    // Failed download with retryable error
    let status = serde_json::json!({
        "status": "error",
        "errorCode": "2", // Timeout
        "gid": "123"
    });
    assert!(analyzer.should_retry(&status));

    // Failed download with non-retryable error
    let status_fatal = serde_json::json!({
        "status": "error",
        "errorCode": "3", // Not found
        "gid": "456"
    });
    assert!(!analyzer.should_retry(&status_fatal));

    // Active download (not error)
    let status_active = serde_json::json!({
        "status": "active",
        "gid": "789"
    });
    assert!(!analyzer.should_retry(&status_active));
}

#[test]
fn test_retry_config_defaults() {
    let config = RetryConfig::default();
    assert_eq!(config.max_retries, 3);
    assert_eq!(config.initial_backoff_secs, 5);
}

#[tokio::test]
async fn test_monitor_queue_integration() -> anyhow::Result<()> {
    use aria2_mcp_rs::tools::McpeTool;
    use aria2_mcp_rs::tools::MonitorQueueTool;

    let mock_server = MockServer::start().await;

    // Mock tellStopped response
    let response = serde_json::json!({
        "id": "aria2-mcp",
        "jsonrpc": "2.0",
        "result": [
            {
                "gid": "retryable-1",
                "status": "error",
                "errorCode": "2"
            },
            {
                "gid": "fatal-1",
                "status": "error",
                "errorCode": "3"
            }
        ]
    });

    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response))
        .mount(&mock_server)
        .await;

    let config = Config::new(mock_server.uri(), None);
    let client = Aria2Client::new(config);
    let tool = MonitorQueueTool;

    let args = serde_json::json!({
        "action": "stopped"
    });

    let result = tool.run(&client, args).await?;
    let items = result.as_array().unwrap();

    // Check retryable-1
    let item1 = items.iter().find(|i| i["gid"] == "retryable-1").unwrap();
    assert_eq!(item1["retryable"], true);

    // Check fatal-1
    let item2 = items.iter().find(|i| i["gid"] == "fatal-1").unwrap();
    assert!(item2.get("retryable").is_none());

    Ok(())
}

#[tokio::test]
async fn test_recovery_manager_perform_retry() -> anyhow::Result<()> {
    let mock_server = MockServer::start().await;

    // Mock responses
    let status_response = serde_json::json!({
        "id": "aria2-mcp",
        "jsonrpc": "2.0",
        "result": {
            "gid": "old-gid",
            "files": [
                {
                    "uris": [
                        { "uri": "http://example.com/file" }
                    ]
                }
            ]
        }
    });

    let add_uri_response = serde_json::json!({
        "id": "aria2-mcp",
        "jsonrpc": "2.0",
        "result": "new-gid"
    });

    let remove_response = serde_json::json!({
        "id": "aria2-mcp",
        "jsonrpc": "2.0",
        "result": "OK"
    });

    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(move |req: &wiremock::Request| {
            let body: serde_json::Value = serde_json::from_slice(&req.body).unwrap();
            let method = body["method"].as_str().unwrap();
            match method {
                "aria2.tellStatus" => ResponseTemplate::new(200).set_body_json(&status_response),
                "aria2.addUri" => ResponseTemplate::new(200).set_body_json(&add_uri_response),
                "aria2.remove" => ResponseTemplate::new(200).set_body_json(&remove_response),
                _ => ResponseTemplate::new(404),
            }
        })
        .mount(&mock_server)
        .await;

    let config = Config::new(mock_server.uri(), None);
    let client = Aria2Client::new(config);
    let manager = RecoveryManager::new(RetryConfig::default());

    let result = manager.perform_retry(&client, "old-gid").await?;
    assert_eq!(result, "new-gid");

    Ok(())
}
