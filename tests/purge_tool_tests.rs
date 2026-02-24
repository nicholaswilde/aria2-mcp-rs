mod common;

use anyhow::Result;
use common::Aria2Container;
use serde_json::json;
use aria2_mcp_rs::tools::purge_policy::PurgePolicyTool;
use aria2_mcp_rs::tools::registry::McpeTool;

#[tokio::test]
async fn test_purge_policy_tool_flow() -> Result<()> {
    if !common::should_run_docker_tests() {
        return Ok(());
    }
    let container = Aria2Container::new().await?;
    let client = container.client();
    let tool = PurgePolicyTool;

    // 1. Get initial policy
    let args = json!({ "action": "get_policy" });
    let result = tool.run(&client, args).await?;
    assert!(!result["policy"]["enabled"].as_bool().unwrap());

    // 2. Enable purging and set interval
    let args = json!({
        "action": "update_policy",
        "enabled": true,
        "intervalSecs": 10
    });
    let result = tool.run(&client, args).await?;
    assert!(result["policy"]["enabled"].as_bool().unwrap());
    assert_eq!(result["policy"]["interval_secs"], 10);

    // 3. Exclude a GID
    let args = json!({
        "action": "exclude_gid",
        "gid": "test-gid-123"
    });
    let result = tool.run(&client, args).await?;
    assert_eq!(result["status"], "success");

    // 4. Verify exclusion in policy
    let args = json!({ "action": "get_policy" });
    let result = tool.run(&client, args).await?;
    let exclusions = result["policy"]["excluded_gids"].as_array().unwrap();
    assert!(exclusions.contains(&json!("test-gid-123")));

    // 5. Remove exclusion
    let args = json!({
        "action": "remove_exclusion",
        "gid": "test-gid-123"
    });
    let result = tool.run(&client, args).await?;
    assert_eq!(result["status"], "success");

    // 6. Verify exclusion removed
    let args = json!({ "action": "get_policy" });
    let result = tool.run(&client, args).await?;
    let exclusions = result["policy"]["excluded_gids"].as_array().unwrap();
    assert!(!exclusions.contains(&json!("test-gid-123")));

    Ok(())
}
