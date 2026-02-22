use aria2_mcp_rs::prompts::{Prompt, PromptRegistry};

#[test]
fn test_prompt_registry_list() {
    let registry = PromptRegistry::new();
    let prompts = registry.list_prompts();
    assert!(prompts.is_empty());
}

#[test]
fn test_prompt_registration() {
    let mut registry = PromptRegistry::new();
    let prompt = Prompt {
        name: "test-prompt".to_string(),
        description: Some("Test Description".to_string()),
        arguments: vec![],
    };
    registry.register(prompt);

    let prompts = registry.list_prompts();
    assert_eq!(prompts.len(), 1);
    assert_eq!(prompts[0].name, "test-prompt");
}
