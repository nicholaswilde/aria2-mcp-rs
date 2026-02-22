use anyhow::Result;
use aria2_mcp_rs::tools::list_download_files::ListDownloadFilesTool;
use aria2_mcp_rs::tools::registry::McpeTool;

#[test]
fn test_list_download_files_name() {
    let tool = ListDownloadFilesTool;
    assert_eq!(tool.name(), "list_download_files");
}

#[test]
fn test_list_download_files_schema() -> Result<()> {
    let tool = ListDownloadFilesTool;
    let schema = tool.schema()?;

    assert_eq!(schema["type"], "object");
    assert!(schema["properties"].get("path").is_some());
    assert!(schema["required"]
        .as_array()
        .unwrap()
        .contains(&serde_json::json!("path")));

    Ok(())
}
