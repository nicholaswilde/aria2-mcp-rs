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

pub struct PromptRegistry {
    prompts: Vec<Prompt>,
}

impl Default for PromptRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl PromptRegistry {
    pub fn new() -> Self {
        Self {
            prompts: Vec::new(),
        }
    }

    pub fn register(&mut self, prompt: Prompt) {
        self.prompts.push(prompt);
    }

    pub fn list_prompts(&self) -> Vec<Prompt> {
        self.prompts.clone()
    }

    pub fn get_prompt(&self, name: &str) -> Option<Prompt> {
        self.prompts.iter().find(|p| p.name == name).cloned()
    }
}
