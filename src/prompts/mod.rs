use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    pub name: String,
    pub description: Option<String>,
    pub arguments: Vec<PromptArgument>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptArgument {
    pub name: String,
    pub description: Option<String>,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptMessage {
    pub role: String, // "user" or "assistant"
    pub content: PromptContent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PromptContent {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "resource")]
    Resource { resource: PromptResource },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptResource {
    pub uri: String,
}

pub mod diagnose_download;

pub use diagnose_download::DiagnoseDownloadPrompt;

pub trait McpPrompt: Send + Sync {
    fn name(&self) -> String;
    fn description(&self) -> Option<String>;
    fn arguments(&self) -> Vec<PromptArgument>;
    fn get_messages(&self, arguments: serde_json::Value) -> anyhow::Result<Vec<PromptMessage>>;
}

pub struct PromptRegistry {
    prompts: Vec<Arc<dyn McpPrompt>>,
}

impl Default for PromptRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl PromptRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            prompts: Vec::new(),
        };

        registry.register(Arc::new(DiagnoseDownloadPrompt));

        registry
    }

    pub fn register(&mut self, prompt: Arc<dyn McpPrompt>) {
        self.prompts.push(prompt);
    }

    pub fn list_prompts(&self) -> Vec<Prompt> {
        self.prompts
            .iter()
            .map(|p| Prompt {
                name: p.name(),
                description: p.description(),
                arguments: p.arguments(),
            })
            .collect()
    }

    pub fn get_prompt(&self, name: &str) -> Option<Arc<dyn McpPrompt>> {
        self.prompts.iter().find(|p| p.name() == name).cloned()
    }
}

use std::sync::Arc;
