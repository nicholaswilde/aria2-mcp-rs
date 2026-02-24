mod common;

use anyhow::Result;
use aria2_mcp_rs::tools::manage_downloads::ManageDownloadsTool;
use aria2_mcp_rs::tools::manage_torrent::ManageTorrentTool;
use aria2_mcp_rs::tools::registry::McpeTool;
use common::Aria2Container;
use serde_json::json;

#[tokio::test]
async fn test_manage_downloads_sequential_flag() -> Result<()> {
    if !common::should_run_docker_tests() {
        return Ok(());
    }
    let container = Aria2Container::new().await?;
    let client = container.client();
    let tool = ManageDownloadsTool;

    // Add a download with sequential=true and max-download-limit
    // We use max-download-limit because aria2 always returns it in getOption,
    // whereas bt-sequential might be hidden for non-BT downloads.
    let args = json!({
        "action": "add",
        "uris": ["https://p3terx.com"],
        "sequential": true,
        "options": {
            "max-download-limit": "1M"
        }
    });

    let result = tool.run(&client, args).await?;
    let gid = result["gid"].as_str().unwrap();

    // Verify the options were set in aria2
    let options = client.get_option(gid).await?;

    // max-download-limit should be 1048576 (1M)
    assert_eq!(options["max-download-limit"], "1048576");

    // We can't easily verify bt-sequential here because it's a non-BT download,
    // but we've verified the option passing mechanism works.

    Ok(())
}

#[tokio::test]
async fn test_manage_torrent_toggle_sequential() -> Result<()> {
    if !common::should_run_docker_tests() {
        return Ok(());
    }
    let container = Aria2Container::new().await?;
    let client = container.client();
    let tool = ManageTorrentTool;

    // Add a download
    let uris = vec!["https://p3terx.com".to_string()];
    let gid = client.add_uri(uris, None).await?;

    // Toggle sequential ON
    // Even if it's not a BT download, the tool should successfully call changeOption
    let args = json!({
        "action": "toggleSequential",
        "gid": gid,
        "sequential": true
    });
    let result = tool.run(&client, args).await?;
    assert_eq!(result["status"], "success");
    assert_eq!(result["sequential"], true);

    Ok(())
}
