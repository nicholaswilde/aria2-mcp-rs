use anyhow::Result;
use aria2_mcp_rs::tools::list_download_files::{ListDownloadFilesArgs, ListDownloadFilesTool};
use aria2_mcp_rs::tools::registry::McpeTool;
use std::fs;
use tempfile::TempDir;

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

#[test]
fn test_list_download_files_run_with_dir_success() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let base_dir = temp_dir.path();
    let tool = ListDownloadFilesTool;

    // Create a structure:
    // /file1.txt
    // /dir1/file2.txt
    fs::write(base_dir.join("file1.txt"), "content")?;
    fs::create_dir(base_dir.join("dir1"))?;
    fs::write(base_dir.join("dir1/file2.txt"), "content")?;

    // List root (depth 1 default)
    let args = ListDownloadFilesArgs {
        path: ".".to_string(),
        max_depth: None,
    };
    let result = tool.run_with_dir(base_dir, args)?;
    
    let entries = result["entries"].as_array().unwrap();
    assert!(entries.len() >= 2); // file1.txt and dir1
    
    // Check for file1.txt
    let file1 = entries.iter().find(|e| e["path"].as_str() == Some("file1.txt"));
    assert!(file1.is_some());
    assert!(!file1.unwrap()["isDir"].as_bool().unwrap());

    // Check for dir1
    let dir1 = entries.iter().find(|e| e["path"].as_str() == Some("dir1"));
    assert!(dir1.is_some());
    assert!(dir1.unwrap()["isDir"].as_bool().unwrap());

    // List subdir
    let args_subdir = ListDownloadFilesArgs {
        path: "dir1".to_string(),
        max_depth: None,
    };
    let result_subdir = tool.run_with_dir(base_dir, args_subdir)?;
    let entries_subdir = result_subdir["entries"].as_array().unwrap();
    
    assert!(entries_subdir.iter().any(|e| e["path"].as_str() == Some("dir1/file2.txt")));

    Ok(())
}

#[test]
fn test_list_download_files_run_with_dir_traversal_prevention() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let base_dir = temp_dir.path();
    let tool = ListDownloadFilesTool;

    let args = ListDownloadFilesArgs {
        path: "../".to_string(),
        max_depth: None,
    };
    let result = tool.run_with_dir(base_dir, args);
    assert!(result.is_err()); // Should fail due to PathSandbox

    Ok(())
}
