use aria2_mcp_rs::prompts::{McpPrompt, PromptArgument, PromptMessage, PromptRegistry};
use std::sync::Arc;

struct MockPrompt;
impl McpPrompt for MockPrompt {
    fn name(&self) -> String {
        "test-prompt".to_string()
    }
    fn description(&self) -> Option<String> {
        Some("Test Description".to_string())
    }
    fn arguments(&self) -> Vec<PromptArgument> {
        vec![]
    }
    fn get_messages(&self, _arguments: serde_json::Value) -> anyhow::Result<Vec<PromptMessage>> {
        Ok(vec![])
    }
}

#[test]
fn test_prompt_registry_list() {
    let registry = PromptRegistry::new();
    let prompts = registry.list_prompts();
    assert!(prompts.is_empty());
}

#[test]
fn test_prompt_registration() {
    let mut registry = PromptRegistry::new();
    registry.register(Arc::new(MockPrompt));

    let prompts = registry.list_prompts();
    assert_eq!(prompts.len(), 1);
    assert_eq!(prompts[0].name, "test-prompt");
}
