use crate::prompts::{McpPrompt, PromptArgument, PromptContent, PromptMessage};
use serde_json::Value;

pub struct OptimizeSchedulePrompt;

impl McpPrompt for OptimizeSchedulePrompt {
    fn name(&self) -> String {
        "optimize-schedule".to_string()
    }

    fn description(&self) -> Option<String> {
        Some("Helps you review and optimize your bandwidth schedules.".to_string())
    }

    fn arguments(&self) -> Vec<PromptArgument> {
        vec![]
    }

    fn get_messages(&self, _arguments: Value) -> anyhow::Result<Vec<PromptMessage>> {
        Ok(vec![
            PromptMessage {
                role: "user".to_string(),
                content: PromptContent::Text {
                    text: "I want to optimize my bandwidth schedules. Please list the current schedules and profiles, and suggest improvements based on common usage patterns (e.g., lower limits during work hours, higher at night).".to_string(),
                },
            },
        ])
    }
}
