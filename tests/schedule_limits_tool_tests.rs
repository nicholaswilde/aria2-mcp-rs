mod common;

use anyhow::Result;
use aria2_mcp_rs::{McpeTool, ScheduleLimitsTool};
use common::Aria2Container;
use serde_json::json;

#[tokio::test]
async fn test_schedule_limits_tool_list_profiles() -> Result<()> {
    if !common::should_run_docker_tests() {
        return Ok(());
    }
    let container = Aria2Container::new().await?;
    let client = container.client();
    let tool = ScheduleLimitsTool;

    let args = json!({
        "action": "list_profiles"
    });

    let result = tool.run(&client, args).await?;
    assert!(result.is_object());
    assert!(result.get("profiles").is_some());

    Ok(())
}

#[tokio::test]
async fn test_schedule_limits_tool_add_set_remove_profile() -> Result<()> {
    if !common::should_run_docker_tests() {
        return Ok(());
    }
    let container = Aria2Container::new().await?;
    let client = container.client();
    let tool = ScheduleLimitsTool;

    // Add profile
    let args_add = json!({
        "action": "add_profile",
        "profile_name": "night",
        "max_download": "10M",
        "max_upload": "1M"
    });
    tool.run(&client, args_add).await?;

    // Verify profile added in list
    let result_list = tool
        .run(&client, json!({"action": "list_profiles"}))
        .await?;
    assert!(result_list["profiles"]["night"].is_object());

    // Set profile
    let args_set = json!({
        "action": "set_profile",
        "profile_name": "night"
    });
    tool.run(&client, args_set).await?;

    // Verify global options changed in aria2
    let global_stat = client.get_global_option().await?;
    assert_eq!(global_stat["max-overall-download-limit"], "10485760");
    assert_eq!(global_stat["max-overall-upload-limit"], "1048576");

    // Remove profile
    let args_remove = json!({
        "action": "remove_profile",
        "profile_name": "night"
    });
    tool.run(&client, args_remove).await?;

    // Verify profile removed
    let result_list2 = tool
        .run(&client, json!({"action": "list_profiles"}))
        .await?;
    assert!(result_list2["profiles"]["night"].is_null());

    Ok(())
}

#[tokio::test]
async fn test_schedule_limits_tool_manage_schedules() -> Result<()> {
    if !common::should_run_docker_tests() {
        return Ok(());
    }
    let container = Aria2Container::new().await?;
    let client = container.client();
    let tool = ScheduleLimitsTool;

    // Add profile first
    tool.run(
        &client,
        json!({
            "action": "add_profile",
            "profile_name": "day",
            "max_download": "1M",
            "max_upload": "100K"
        }),
    )
    .await?;

    // Add schedule
    let args_add_sched = json!({
        "action": "add_schedule",
        "profile_name": "day",
        "schedule": {
            "day": "mon",
            "start_time": "09:00",
            "end_time": "17:00"
        }
    });
    tool.run(&client, args_add_sched).await?;

    // List schedules
    let result_list = tool
        .run(&client, json!({"action": "list_schedules"}))
        .await?;
    assert_eq!(result_list["schedules"].as_array().unwrap().len(), 1);
    assert_eq!(result_list["schedules"][0]["profile_name"], "day");

    // Remove schedule
    let args_rem_sched = json!({
        "action": "remove_schedule",
        "index": 0
    });
    tool.run(&client, args_rem_sched).await?;

    // Verify removed
    let result_list2 = tool
        .run(&client, json!({"action": "list_schedules"}))
        .await?;
    assert_eq!(result_list2["schedules"].as_array().unwrap().len(), 0);

    Ok(())
}
