mod common;

use anyhow::Result;
use aria2_mcp_rs::{McpeTool, OrganizeCompletedTool};
use common::Aria2Container;
use serde_json::json;

#[tokio::test]
async fn test_organize_completed_name() -> Result<()> {
    let tool = OrganizeCompletedTool;
    assert_eq!(tool.name(), "organize_completed");
    Ok(())
}

#[tokio::test]
async fn test_organize_completed_schema() -> Result<()> {
    let tool = OrganizeCompletedTool;
    let schema = tool.schema()?;
    assert_eq!(schema["type"], "object");
    Ok(())
}

#[tokio::test]
async fn test_organize_completed_run_todo() -> Result<()> {
    if !common::should_run_docker_tests() {
        return Ok(());
    }
    let container = Aria2Container::new().await?;
    let client = container.client();
    let tool = OrganizeCompletedTool;

    let args = json!({});
    let result = tool.run(&client, args).await;

    // This should fail because it calls todo!()
    // assert!(result.is_err());
    // assert!(result.unwrap_err().to_string().contains("not yet implemented"));

    // Now it should return success but with 0 organized count because there are no completed downloads
    assert!(result.is_ok());
    let res = result.unwrap();
    assert_eq!(res["status"], "no_rules");

    Ok(())
}

#[tokio::test]
async fn test_organize_completed_not_complete_error() -> Result<()> {
    if !common::should_run_docker_tests() {
        return Ok(());
    }
    let container = Aria2Container::new().await?;
    let client = container.client();
    let tool = OrganizeCompletedTool;

    // Add a download that will be active (using a large file or just not completing yet)
    let gid = client
        .add_uri(
            vec!["https://proof.ovh.net/files/100Mb.dat".to_string()],
            None,
        )
        .await?;

    let args = json!({
        "gid": gid,
        "rules": [
            {
                "name": "ISOs",
                "extensions": ["dat"],
                "targetDir": "/tmp/isos"
            }
        ]
    });

    let result = tool.run(&client, args).await;

    // Should fail because it's not complete
    assert!(result.is_err(), "Expected error for non-complete download");
    let err_msg = result.unwrap_err().to_string();
    println!("Error message: {}", err_msg);
    assert!(
        err_msg.contains("is not complete"),
        "Error message '{}' does not contain 'is not complete'",
        err_msg
    );

    Ok(())
}
