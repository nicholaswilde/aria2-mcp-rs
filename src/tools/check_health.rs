use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::path::Path;
use std::sync::Arc;

use crate::aria2::Aria2Client;
use crate::tools::registry::McpeTool;

pub struct CheckHealthTool;

#[async_trait]
impl McpeTool for CheckHealthTool {
    fn name(&self) -> String {
        "check_health".to_string()
    }

    fn description(&self) -> String {
        "Check the health of the aria2 download queue and local system".to_string()
    }

    fn schema(&self) -> Result<Value> {
        Ok(json!({
            "type": "object",
            "properties": {}
        }))
    }

    async fn run(&self, client: &Aria2Client, _args: Value) -> Result<Value> {
        let stats = client.get_global_stat().await?;
        let active = client.tell_active(None).await?;
        let _waiting = client.tell_waiting(0, 1000, None).await?;
        let stopped = client.tell_stopped(0, 1000, None).await?;

        let global_options = client.get_global_option().await?;
        let download_dir = global_options
            .get("dir")
            .and_then(|v| v.as_str())
            .unwrap_or(".");

        // Check disk space
        let disk_info = get_disk_info(download_dir).ok();

        let report = self.analyze_health(stats, active, stopped, disk_info, download_dir);
        Ok(report)
    }

    async fn run_multi(&self, clients: &[Arc<Aria2Client>], _args: Value) -> Result<Value> {
        let mut results = Vec::new();

        for client in clients {
            let res = self.run(client, json!({})).await;
            results.push(json!({
                "instance": client.name,
                "status": if res.is_ok() { "ok" } else { "error" },
                "report": res.unwrap_or_else(|e| json!({"error": e.to_string()}))
            }));
        }

        Ok(json!({
            "results": results
        }))
    }
}

impl CheckHealthTool {
    fn analyze_health(
        &self,
        stats: Value,
        active: Value,
        stopped: Value,
        disk_info: Option<DiskInfo>,
        download_dir: &str,
    ) -> Value {
        let mut issues = Vec::new();
        let mut recommendations = Vec::new();

        // Check for stalled active downloads
        if let Some(active_arr) = active.as_array() {
            for item in active_arr {
                let gid = item
                    .get("gid")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let connections = item
                    .get("connections")
                    .and_then(|v| v.as_str())
                    .and_then(|v| v.parse::<u64>().ok())
                    .or_else(|| item.get("connections").and_then(|v| v.as_u64()))
                    .unwrap_or(0);
                let download_speed = item
                    .get("downloadSpeed")
                    .and_then(|v| v.as_str())
                    .and_then(|v| v.parse::<u64>().ok())
                    .or_else(|| item.get("downloadSpeed").and_then(|v| v.as_u64()))
                    .unwrap_or(0);

                if connections == 0 && download_speed == 0 {
                    issues.push(json!({
                        "type": "stalled_download",
                        "gid": gid,
                        "message": format!("Download {} has 0 peers and 0 speed.", gid)
                    }));
                    recommendations.push(format!("Consider adding more trackers to download {} or check your network connection.", gid));
                }
            }
        }

        // Check for errors in stopped downloads
        if let Some(stopped_arr) = stopped.as_array() {
            for item in stopped_arr {
                let status = item.get("status").and_then(|v| v.as_str()).unwrap_or("");
                if status == "error" {
                    let gid = item
                        .get("gid")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");
                    let error_code = item
                        .get("errorCode")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");
                    let error_msg = item
                        .get("errorMessage")
                        .and_then(|v| v.as_str())
                        .unwrap_or("No error message provided.");

                    issues.push(json!({
                        "type": "download_error",
                        "gid": gid,
                        "message": format!("Download {} failed with error code {}: {}", gid, error_code, error_msg)
                    }));
                    recommendations.push(format!(
                        "Check the error details for download {} and retry if appropriate.",
                        gid
                    ));
                }
            }
        }

        // Check disk space
        if let Some(info) = &disk_info {
            if info.available < 1024 * 1024 * 1024 {
                // Less than 1GB
                issues.push(json!({
                    "type": "low_disk_space",
                    "message": format!("Available disk space in {} is low: {:.2} GB", download_dir, info.available as f64 / 1e9)
                }));
                recommendations
                    .push("Clean up completed downloads or increase disk space.".to_string());
            }
        }

        // Summary
        let summary = json!({
            "num_active": stats.get("numActive").and_then(|v| v.as_str()).map(|s| s.to_string()).or_else(|| stats.get("numActive").and_then(|v| v.as_u64().map(|u| u.to_string()))),
            "num_waiting": stats.get("numWaiting").and_then(|v| v.as_str()).map(|s| s.to_string()).or_else(|| stats.get("numWaiting").and_then(|v| v.as_u64().map(|u| u.to_string()))),
            "num_stopped": stats.get("numStopped").and_then(|v| v.as_str()).map(|s| s.to_string()).or_else(|| stats.get("numStopped").and_then(|v| v.as_u64().map(|u| u.to_string()))),
            "download_speed": stats.get("downloadSpeed").and_then(|v| v.as_str()).map(|s| s.to_string()).or_else(|| stats.get("downloadSpeed").and_then(|v| v.as_u64().map(|u| u.to_string()))),
            "upload_speed": stats.get("uploadSpeed").and_then(|v| v.as_str()).map(|s| s.to_string()).or_else(|| stats.get("uploadSpeed").and_then(|v| v.as_u64().map(|u| u.to_string()))),
            "disk_available_gb": disk_info.as_ref().map(|i| i.available as f64 / 1e9).unwrap_or(0.0)
        });

        json!({
            "summary": summary,
            "issues": issues,
            "recommendations": recommendations,
            "status": if issues.is_empty() { "healthy" } else { "unhealthy" }
        })
    }
}

#[derive(Clone)]
struct DiskInfo {
    available: u64,
    _total: u64,
}

#[cfg(target_os = "linux")]
fn get_disk_info<P: AsRef<Path>>(path: P) -> Result<DiskInfo> {
    use std::mem;
    unsafe {
        let mut stats: libc::statvfs = mem::zeroed();
        let path_str = std::ffi::CString::new(path.as_ref().to_string_lossy().as_bytes())?;
        if libc::statvfs(path_str.as_ptr(), &mut stats) == 0 {
            Ok(DiskInfo {
                available: stats.f_bsize as u64 * stats.f_bavail as u64,
                _total: stats.f_bsize as u64 * stats.f_blocks as u64,
            })
        } else {
            Err(anyhow::anyhow!("Failed to get disk stats"))
        }
    }
}

#[cfg(not(target_os = "linux"))]
fn get_disk_info<P: AsRef<Path>>(_path: P) -> Result<DiskInfo> {
    // Fallback for non-linux systems if needed, or just return dummy info
    Ok(DiskInfo {
        available: 10 * 1024 * 1024 * 1024, // 10GB dummy
        _total: 100 * 1024 * 1024 * 1024,   // 100GB dummy
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aria2::Aria2Client;
    use crate::config::Config;

    #[tokio::test]
    async fn test_check_health_name() {
        let tool = CheckHealthTool;
        assert_eq!(tool.name(), "check_health");
    }

    #[tokio::test]
    async fn test_check_health_run_error() {
        let tool = CheckHealthTool;
        let client = Aria2Client::new(Config::default());
        let result = tool.run(&client, json!({})).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_analyze_health_healthy() {
        let tool = CheckHealthTool;
        let stats = json!({ "numActive": "0", "numWaiting": "0", "numStopped": "0", "downloadSpeed": "0", "uploadSpeed": "0" });
        let active = json!([]);
        let stopped = json!([]);
        let disk_info = Some(DiskInfo {
            available: 10 * 1024 * 1024 * 1024,
            _total: 100 * 1024 * 1024 * 1024,
        });

        let report = tool.analyze_health(stats, active, stopped, disk_info, "/tmp");
        assert_eq!(report["status"], "healthy");
        assert!(report["issues"].as_array().unwrap().is_empty());
    }

    #[test]
    fn test_analyze_health_stalled() {
        let tool = CheckHealthTool;
        let stats = json!({ "numActive": "1", "numWaiting": "0", "numStopped": "0", "downloadSpeed": "0", "uploadSpeed": "0" });
        let active = json!([{ "gid": "1", "connections": "0", "downloadSpeed": "0" }]);
        let stopped = json!([]);

        let report = tool.analyze_health(stats, active, stopped, None, "/tmp");
        assert_eq!(report["status"], "unhealthy");
        assert_eq!(report["issues"][0]["type"], "stalled_download");
    }

    #[test]
    fn test_analyze_health_error() {
        let tool = CheckHealthTool;
        let stats = json!({ "numActive": "0", "numWaiting": "0", "numStopped": "1", "downloadSpeed": "0", "uploadSpeed": "0" });
        let active = json!([]);
        let stopped =
            json!([{ "gid": "2", "status": "error", "errorCode": "1", "errorMessage": "Failed" }]);

        let report = tool.analyze_health(stats, active, stopped, None, "/tmp");
        assert_eq!(report["status"], "unhealthy");
        assert_eq!(report["issues"][0]["type"], "download_error");
    }

    #[test]
    fn test_analyze_health_low_disk() {
        let tool = CheckHealthTool;
        let stats = json!({ "numActive": "0", "numWaiting": "0", "numStopped": "0", "downloadSpeed": "0", "uploadSpeed": "0" });
        let active = json!([]);
        let stopped = json!([]);
        let disk_info = Some(DiskInfo {
            available: 500 * 1024 * 1024,
            _total: 100 * 1024 * 1024 * 1024,
        });

        let report = tool.analyze_health(stats, active, stopped, disk_info, "/tmp");
        assert_eq!(report["status"], "unhealthy");
        assert_eq!(report["issues"][0]["type"], "low_disk_space");
    }
}
