use anyhow::Result;
use async_trait::async_trait;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::Path;

use crate::aria2::Aria2Client;
use crate::tools::registry::McpeTool;

pub struct OrganizeCompletedTool;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Rule {
    pub name: String,
    pub pattern: Option<String>,
    pub extensions: Option<Vec<String>>,
    pub target_dir: String,
}

impl Rule {
    pub fn matches(&self, filename: &str) -> bool {
        if let Some(extensions) = &self.extensions {
            let path = Path::new(filename);
            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                if extensions.iter().any(|e| e.eq_ignore_ascii_case(ext)) {
                    return true;
                }
            }
        }

        if let Some(pattern) = &self.pattern {
            if let Ok(re) = Regex::new(pattern) {
                if re.is_match(filename) {
                    return true;
                }
            }
        }

        false
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrganizeCompletedArgs {
    /// GID of the download to organize (optional, if omitted, all completed downloads are organized)
    pub gid: Option<String>,
    /// Rules for organizing (optional, uses config if omitted)
    pub rules: Option<Vec<Rule>>,
}

#[async_trait]
impl McpeTool for OrganizeCompletedTool {
    fn name(&self) -> String {
        "organize_completed".to_string()
    }

    fn description(&self) -> String {
        "Organize completed downloads by moving them to directories based on rules".to_string()
    }

    fn schema(&self) -> Result<Value> {
        Ok(json!({
            "type": "object",
            "properties": {
                "gid": {
                    "type": "string",
                    "description": "GID of the download to organize"
                },
                "rules": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "name": { "type": "string" },
                            "pattern": { "type": "string" },
                            "extensions": {
                                "type": "array",
                                "items": { "type": "string" }
                            },
                            "targetDir": { "type": "string" }
                        },
                        "required": ["name", "targetDir"]
                    },
                    "description": "Rules for organizing"
                }
            }
        }))
    }

    async fn run(&self, client: &Aria2Client, args: Value) -> Result<Value> {
        let args: OrganizeCompletedArgs = serde_json::from_value(args)?;
        let rules = args.rules.unwrap_or_default();

        if rules.is_empty() {
            return Ok(json!({ "status": "no_rules", "message": "No rules provided for organization" }));
        }

        let mut organized_count = 0;

        if let Some(gid) = args.gid {
            let status = client.tell_status(&gid).await?;
            if status["status"] == "complete" {
                if self.organize_download(&status, &rules).await? {
                    organized_count += 1;
                }
            } else {
                return Err(anyhow::anyhow!(
                    "Download {} is not complete (status: {})",
                    gid,
                    status["status"]
                ));
            }
        } else {
            let stopped = client.tell_stopped(0, 1000, None).await?;
            if let Some(stopped_list) = stopped.as_array() {
                for status in stopped_list {
                    if status["status"] == "complete" {
                        if self.organize_download(status, &rules).await? {
                            organized_count += 1;
                        }
                    }
                }
            }
        }

        Ok(json!({ "status": "success", "organizedCount": organized_count }))
    }
}

impl OrganizeCompletedTool {
    async fn organize_download(&self, status: &Value, rules: &[Rule]) -> Result<bool> {
        let files = status["files"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("No files found in download status"))?;

        let mut organized = false;

        for file in files {
            let path_str = file["path"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("File path is missing"))?;
            
            if path_str.is_empty() {
                continue;
            }

            let path = Path::new(path_str);
            let filename = path
                .file_name()
                .and_then(|s| s.to_str())
                .ok_or_else(|| anyhow::anyhow!("Failed to get filename from path: {}", path_str))?;

            for rule in rules {
                if rule.matches(filename) {
                    let target_dir = Path::new(&rule.target_dir);
                    if !target_dir.exists() {
                        tokio::fs::create_dir_all(target_dir).await?;
                    }

                    let target_path = target_dir.join(filename);
                    println!("Moving {} to {}", path_str, target_path.display());
                    
                    // Standard fs::rename might fail across filesystems, but for now we use tokio::fs::rename
                    tokio::fs::rename(path, target_path).await?;
                    organized = true;
                    break;
                }
            }
        }

        Ok(organized)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_matches_extension() {
        let rule = Rule {
            name: "Movies".to_string(),
            pattern: None,
            extensions: Some(vec!["mp4".to_string(), "mkv".to_string()]),
            target_dir: "/movies".to_string(),
        };

        assert!(rule.matches("movie.mp4"));
        assert!(rule.matches("movie.MKV"));
        assert!(!rule.matches("document.pdf"));
    }

    #[test]
    fn test_rule_matches_pattern() {
        let rule = Rule {
            name: "Linux ISOs".to_string(),
            pattern: Some("ubuntu-.*\\.iso".to_string()),
            extensions: None,
            target_dir: "/isos".to_string(),
        };

        assert!(rule.matches("ubuntu-22.04-desktop-amd64.iso"));
        assert!(!rule.matches("fedora-36-x86_64.iso"));
    }
}
