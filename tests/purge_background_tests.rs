mod common;

use anyhow::Result;
use aria2_mcp_rs::server::start_purge_task;
use common::Aria2Container;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn test_background_purge_logic() -> Result<()> {
    if !common::should_run_docker_tests() {
        return Ok(());
    }
    let container = Aria2Container::new().await?;
    let client = Arc::new(container.client());

    // 1. Configure purge: enabled, 1s interval, 0s min age
    {
        let config = client.config();
        let mut config_guard = config.write().await;
        config_guard.purge_config.enabled = true;
        config_guard.purge_config.interval_secs = 1;
        config_guard.purge_config.min_age_secs = 0;
    }

    // 2. Start the purge task in background
    let client_task = Arc::clone(&client);
    let purge_handle = tokio::spawn(async move {
        let _ = start_purge_task(client_task).await;
    });

    // 3. Add a fast download
    let uris = vec![
        "https://raw.githubusercontent.com/nicholaswilde/aria2-mcp-rs/main/LICENSE".to_string(),
    ];
    let gid = client.add_uri(uris, None).await?;

    // 4. Wait for it to be completed
    let mut completed = false;
    for _ in 0..20 {
        let status = client.tell_status(&gid).await?;
        if status["status"] == "complete" {
            completed = true;
            break;
        }
        sleep(Duration::from_millis(500)).await;
    }
    assert!(completed, "Download should have completed");

    // 5. Now wait for the purge task to remove it from the stopped list
    // In aria2, when it's purged, tellStatus(gid) will return an error
    let mut purged = false;
    for _ in 0..10 {
        let stopped = client.tell_stopped(0, 100, None).await?;
        let items = stopped.as_array().unwrap();
        let found = items.iter().any(|item| item["gid"] == gid);
        if !found {
            purged = true;
            break;
        }
        sleep(Duration::from_millis(500)).await;
    }

    assert!(purged, "Download should have been purged from stopped list");

    purge_handle.abort();
    Ok(())
}
