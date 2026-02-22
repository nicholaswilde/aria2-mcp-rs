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
    // Should have 1 default prompt (diagnose-download)
    assert_eq!(prompts.len(), 1);
    assert_eq!(prompts[0].name, "diagnose-download");
}

#[test]
fn test_prompt_registration() {
    let mut registry = PromptRegistry::new();
    registry.register(Arc::new(MockPrompt));

    let prompts = registry.list_prompts();
    // 1 default (diagnose-download) + 1 mock
    assert_eq!(prompts.len(), 2);
    assert!(prompts.iter().any(|p| p.name == "test-prompt"));
    assert!(prompts.iter().any(|p| p.name == "diagnose-download"));
}

#[test]
fn test_diagnose_download_prompt_messages() {
    let prompt = aria2_mcp_rs::prompts::DiagnoseDownloadPrompt;
    let args = serde_json::json!({ "gid": "123" });
    let messages = prompt.get_messages(args).unwrap();

    assert_eq!(messages.len(), 3);
    match &messages[0].content {
        aria2_mcp_rs::prompts::PromptContent::Text { text } => {
            assert!(text.contains("123"));
        }
        _ => panic!("Expected text content"),
    }
    match &messages[2].content {
        aria2_mcp_rs::prompts::PromptContent::Resource { resource } => {
            assert_eq!(resource.uri, "aria2://logs/recent");
        }
        _ => panic!("Expected resource content"),
    }
}
