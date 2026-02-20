use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::aria2::Aria2Client;
use crate::tools::registry::Tool;

pub struct MonitorQueueTool;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorQueueArgs {
    /// Action to perform: active, waiting, stopped, stats
    pub action: String,
    /// Offset (for actions: waiting, stopped)
    pub offset: Option<i32>,
    /// Number of tasks to return (for actions: waiting, stopped)
    pub num: Option<i32>,
    /// Keys to return
    pub keys: Option<Vec<String>>,
}

#[async_trait]
impl Tool for MonitorQueueTool {
    fn name(&self) -> &str {
        "monitor_queue"
    }

    fn description(&self) -> &str {
        "Monitor the aria2 download queue: active, waiting, stopped downloads and global stats"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["active", "waiting", "stopped", "stats"],
                    "description": "Action to perform"
                },
                "offset": {
                    "type": "integer",
                    "description": "Offset (for waiting, stopped)"
                },
                "num": {
                    "type": "integer",
                    "description": "Number of tasks (for waiting, stopped)"
                },
                "keys": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Optional keys to return for each task"
                }
            },
            "required": ["action"]
        })
    }

    async fn execute(&self, client: &Aria2Client, args: Value) -> Result<Value> {
        let args: MonitorQueueArgs = serde_json::from_value(args)?;

        match args.action.as_str() {
            "active" => client.tell_active(args.keys).await,
            "waiting" => {
                let offset = args.offset.unwrap_or(0);
                let num = args.num.unwrap_or(100);
                client.tell_waiting(offset, num, args.keys).await
            }
            "stopped" => {
                let offset = args.offset.unwrap_or(0);
                let num = args.num.unwrap_or(100);
                client.tell_stopped(offset, num, args.keys).await
            }
            "stats" => client.get_global_stat().await,
            _ => Err(anyhow::anyhow!("Unknown action: {}", args.action)),
        }
    }
}
