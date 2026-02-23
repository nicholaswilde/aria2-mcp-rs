use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::aria2::Aria2Client;
use crate::tools::registry::McpeTool;

pub struct ListDownloadFilesTool;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListDownloadFilesArgs {
    /// Relative path within the download directory
    pub path: String,
    /// Maximum depth for listing (optional)
    pub max_depth: Option<u32>,
}

#[async_trait]
impl McpeTool for ListDownloadFilesTool {
    fn name(&self) -> String {
        "list_download_files".to_string()
    }

    fn description(&self) -> String {
        "List files and directories within a specified path relative to the download directory."
            .to_string()
    }

    fn schema(&self) -> Result<Value> {
        Ok(json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Relative path within the download directory"
                },
                "maxDepth": {
                    "type": "integer",
                    "description": "Maximum depth for recursive listing (optional, default is 1)",
                    "minimum": 1
                }
            },
            "required": ["path"]
        }))
    }

    async fn run(&self, client: &Aria2Client, args: Value) -> Result<Value> {
        let args: ListDownloadFilesArgs = serde_json::from_value(args)?;

        // Fetch the download directory from aria2
        let global_options = client.get_global_option().await?;
        let dir_str = global_options
            .get("dir")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Failed to get 'dir' option from aria2"))?;

        self.run_with_dir(std::path::Path::new(dir_str), args)
    }
}

impl ListDownloadFilesTool {
    pub fn run_with_dir(
        &self,
        base_dir: &std::path::Path,
        args: ListDownloadFilesArgs,
    ) -> Result<Value> {
        let max_depth = args.max_depth.unwrap_or(1);
        let sandbox = crate::tools::sandbox::PathSandbox::new(base_dir.to_path_buf());
        let resolved_path = sandbox.resolve(&args.path)?;

        let mut entries = Vec::new();
        self.list_recursive(&resolved_path, &sandbox, max_depth, 0, &mut entries)?;

        Ok(json!({
            "baseDir": base_dir.to_string_lossy(),
            "path": args.path,
            "entries": entries
        }))
    }

    fn list_recursive(
        &self,
        current_path: &std::path::Path,
        sandbox: &crate::tools::sandbox::PathSandbox,
        max_depth: u32,
        current_depth: u32,
        entries: &mut Vec<Value>,
    ) -> Result<()> {
        if current_depth >= max_depth {
            return Ok(());
        }

        let read_dir = std::fs::read_dir(current_path)
            .map_err(|e| anyhow::anyhow!("Failed to read directory {:?}: {}", current_path, e))?;

        for entry in read_dir {
            let entry = entry?;
            let path = entry.path();
            let metadata = entry.metadata()?;
            let is_dir = metadata.is_dir();

            // Get relative path for output
            let rel_path = path
                .strip_prefix(sandbox.base_dir())
                .unwrap_or(&path)
                .to_string_lossy()
                .to_string();

            entries.push(json!({
                "path": rel_path,
                "isDir": is_dir,
                "size": if is_dir { None } else { Some(metadata.len()) }
            }));

            if is_dir {
                self.list_recursive(&path, sandbox, max_depth, current_depth + 1, entries)?;
            }
        }

        Ok(())
    }
}
