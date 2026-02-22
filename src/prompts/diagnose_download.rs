use crate::prompts::{McpPrompt, PromptArgument, PromptContent, PromptMessage, PromptResource};
use serde_json::Value;

pub struct DiagnoseDownloadPrompt;

impl McpPrompt for DiagnoseDownloadPrompt {
    fn name(&self) -> String {
        "diagnose-download".to_string()
    }

    fn description(&self) -> Option<String> {
        Some(
            "Guides you through diagnosing issues with a specific download or the entire queue."
                .to_string(),
        )
    }

    fn arguments(&self) -> Vec<PromptArgument> {
        vec![PromptArgument {
            name: "gid".to_string(),
            description: Some("Optional GID of the download to diagnose.".to_string()),
            required: false,
        }]
    }

    fn get_messages(&self, arguments: Value) -> anyhow::Result<Vec<PromptMessage>> {
        let gid = arguments.get("gid").and_then(|v| v.as_str());

        let mut text = "I need help diagnosing a download issue.".to_string();
        if let Some(gid) = gid {
            text.push_str(&format!(" The GID is {}.", gid));
        }

        Ok(vec![
            PromptMessage {
                role: "user".to_string(),
                content: PromptContent::Text { text },
            },
            PromptMessage {
                role: "user".to_string(),
                content: PromptContent::Text {
                    text: "Please check the health of the downloads and review the recent logs."
                        .to_string(),
                },
            },
            PromptMessage {
                role: "user".to_string(),
                content: PromptContent::Resource {
                    resource: PromptResource {
                        uri: "aria2://logs/recent".to_string(),
                    },
                },
            },
        ])
    }
}
