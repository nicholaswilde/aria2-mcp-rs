mod common;

use anyhow::Result;
use aria2_mcp_rs::{ConfigureAria2Tool, McpeTool};
use common::Aria2Container;
use serde_json::json;

#[tokio::test]
async fn test_configure_aria2_tool_integration() -> Result<()> {
    if !common::should_run_docker_tests() {
        return Ok(());
    }
    let container = Aria2Container::new().await?;
    let client = container.client();
    let tool = ConfigureAria2Tool;

    // 1. Test Global Config
    let args_global = json!({
        "action": "change_global",
        "options": {
            "max-overall-download-limit": "500K"
        }
    });
    tool.run(&client, args_global).await?;

    let args_get_global = json!({
        "action": "get_global"
    });
    let result_global = tool.run(&client, args_get_global).await?;
    assert_eq!(result_global["max-overall-download-limit"], "512000");

    // 2. Test Local Config
    let uris = vec!["https://p3terx.com".to_string()];
    let gid = client.add_uri(uris, None).await?;

    let args_local = json!({
        "action": "change_local",
        "gid": gid,
        "options": {
            "max-download-limit": "100K"
        }
    });
    tool.run(&client, args_local).await?;

    let args_get_local = json!({
        "action": "get_local",
        "gid": gid
    });
    let result_local = tool.run(&client, args_get_local).await?;
    assert_eq!(result_local["max-download-limit"], "102400");

    Ok(())
}

#[tokio::test]
async fn test_config_update() -> Result<()> {
    if !common::should_run_docker_tests() {
        return Ok(());
    }
    let container = Aria2Container::new().await?;
    let client = container.client();

    // Get current max-overall-download-limit
    let options = client.get_global_option().await?;
    let original_limit = options["max-overall-download-limit"]
        .as_str()
        .unwrap()
        .to_string();
    println!("original limit: {}", original_limit);

    // Change limit
    let new_limit = "1M";
    let mut new_opts = serde_json::Map::new();
    new_opts.insert(
        "max-overall-download-limit".to_string(),
        serde_json::json!(new_limit),
    );
    client
        .change_global_option(serde_json::Value::Object(new_opts))
        .await?;

    // Verify change
    let updated_options = client.get_global_option().await?;
    assert_eq!(updated_options["max-overall-download-limit"], "1048576");
    println!(
        "updated limit: {}",
        updated_options["max-overall-download-limit"]
    );

    Ok(())
}

#[tokio::test]
async fn test_pause_resume() -> Result<()> {
    if !common::should_run_docker_tests() {
        return Ok(());
    }
    let container = Aria2Container::new().await?;
    let client = container.client();

    let uris = vec!["https://p3terx.com".to_string()];
    let gid = client.add_uri(uris, None).await?;

    // Pause
    client.pause(&gid).await?;
    let status = client.tell_status(&gid).await?;
    assert_eq!(status["status"], "paused");

    // Resume
    client.unpause(&gid).await?;
    let status = client.tell_status(&gid).await?;
    assert!(status["status"] == "active" || status["status"] == "waiting");

    Ok(())
}

#[tokio::test]
async fn test_status_reporting() -> Result<()> {
    if !common::should_run_docker_tests() {
        return Ok(());
    }
    let container = Aria2Container::new().await?;
    let client = container.client();

    let uris = vec!["https://p3terx.com".to_string()];
    let gid = client.add_uri(uris, None).await?;

    // Check progress fields
    let status = client.tell_status(&gid).await?;
    assert!(status.get("completedLength").is_some());
    assert!(status.get("totalLength").is_some());
    assert!(status.get("downloadSpeed").is_some());

    let completed = status["completedLength"]
        .as_str()
        .unwrap()
        .parse::<u64>()
        .unwrap();
    let total = status["totalLength"]
        .as_str()
        .unwrap()
        .parse::<u64>()
        .unwrap();
    let speed = status["downloadSpeed"]
        .as_str()
        .unwrap()
        .parse::<u64>()
        .unwrap();

    println!(
        "progress: {}/{} (speed: {} bytes/s)",
        completed, total, speed
    );

    Ok(())
}

#[tokio::test]
async fn test_add_download() -> Result<()> {
    if !common::should_run_docker_tests() {
        return Ok(());
    }
    let container = Aria2Container::new().await?;
    let client = container.client();

    let uris = vec!["https://p3terx.com".to_string()];
    let gid = client.add_uri(uris, None).await?;

    assert!(!gid.is_empty());
    println!("added download with GID: {}", gid);

    // Verify status
    let status = client.tell_status(&gid).await?;
    assert_eq!(status["gid"], gid);
    let status_str = status["status"].as_str().unwrap();
    println!("download status: {}", status_str);
    assert!(status_str == "active" || status_str == "waiting" || status_str == "complete");

    Ok(())
}

#[tokio::test]
async fn test_inspect_download_files_and_uris() -> Result<()> {
    if !common::should_run_docker_tests() {
        return Ok(());
    }
    let container = Aria2Container::new().await?;
    let client = container.client();

    let uris = vec!["https://p3terx.com".to_string()];
    let gid = client.add_uri(uris, None).await?;

    // Verify files
    let files = client.get_files(&gid).await?;
    assert!(files.is_array());
    let files_array = files.as_array().unwrap();
    assert!(!files_array.is_empty());
    // Check if path or length exists
    assert!(files_array[0].get("path").is_some());
    assert!(files_array[0].get("length").is_some());

    // Verify URIs
    let uris_resp = client.get_uris(&gid).await?;
    assert!(uris_resp.is_array());
    let uris_array = uris_resp.as_array().unwrap();
    assert!(!uris_array.is_empty());
    // Check if uri matches one we added (or at least check structure)
    // Note: aria2 might add default trackers, so we check if our URI is present
    let found = uris_array
        .iter()
        .any(|u| u["uri"].as_str().unwrap() == "https://p3terx.com");
    assert!(found, "Original URI not found in get_uris response");

    Ok(())
}

#[tokio::test]
async fn test_container_starts_and_is_reachable() -> Result<()> {
    if !common::should_run_docker_tests() {
        return Ok(());
    }
    let container = Aria2Container::new().await?;
    let client = container.client();
    let version = client.get_version().await?;
    println!("aria2 version: {}", version);
    assert!(!version.is_empty());
    Ok(())
}
