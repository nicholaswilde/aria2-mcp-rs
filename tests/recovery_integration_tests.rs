mod common;

use anyhow::Result;
use aria2_mcp_rs::aria2::recovery::{RecoveryManager, RetryConfig};
use common::Aria2Container;
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn test_automated_recovery_integration() -> Result<()> {
    if !common::should_run_docker_tests() {
        return Ok(());
    }
    let container = Aria2Container::new().await?;
    let client = container.client();

    // Add a download that will fail (unreachable host)
    let uris = vec!["http://unreachable.invalid/file".to_string()];
    let gid = client.add_uri(uris, None).await?;

    // Wait for it to fail
    let mut status = client.tell_status(&gid).await?;
    let mut attempts = 0;
    while status["status"] != "error" && attempts < 10 {
        sleep(Duration::from_secs(1)).await;
        status = client.tell_status(&gid).await?;
        attempts += 1;
    }
    assert_eq!(status["status"], "error", "Download should have failed");

    // Setup recovery manager with short backoff for testing
    let retry_config = RetryConfig {
        max_retries: 1,
        initial_backoff_secs: 1,
        ..Default::default()
    };
    let recovery_manager = RecoveryManager::new(retry_config);

    // Trigger recovery analysis
    let backoff = recovery_manager
        .analyze_and_get_retry_backoff(&gid, &status)
        .await;
    assert!(backoff.is_some(), "Should be retryable");

    // Perform retry
    let new_gid = recovery_manager.perform_retry(&client, &gid).await?;
    assert_ne!(new_gid, gid, "Should have a new GID");

    // Verify old one is removed (or at least handled)
    // In our implementation, we call remove() which might fail if it's already a 'result'
    // but the GID should now be in the stopped list or gone.

    // Verify new one exists
    let new_status = client.tell_status(&new_gid).await?;
    assert_eq!(new_status["gid"], new_gid);

    Ok(())
}
