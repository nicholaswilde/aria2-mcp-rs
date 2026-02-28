use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;

use crate::config::{BandwidthProfile, BandwidthSchedule};
use crate::error::{Error, Result};
use crate::tools::organize_completed::Rule;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StateData {
    pub rules: HashMap<String, String>, // Kept for generic usage if needed
    #[serde(default)]
    pub bandwidth_profiles: HashMap<String, BandwidthProfile>,
    #[serde(default)]
    pub bandwidth_schedules: Vec<BandwidthSchedule>,
    #[serde(default)]
    pub organize_rules: Vec<Rule>,
}

#[derive(Debug, Clone)]
pub struct StateManager {
    path: PathBuf,
}

impl StateManager {
    #[must_use]
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub async fn load(&self) -> Result<StateData> {
        if !self.path.exists() {
            return Ok(StateData::default());
        }

        let content = fs::read_to_string(&self.path)
            .await
            .map_err(|e| Error::Internal(format!("Failed to read state file: {e}")))?;

        if content.trim().is_empty() {
            return Ok(StateData::default());
        }

        let data = serde_json::from_str(&content)
            .map_err(|e| Error::Internal(format!("Failed to parse state file: {e}")))?;

        Ok(data)
    }

    pub async fn save(&self, data: &StateData) -> Result<()> {
        let content = serde_json::to_string_pretty(data)
            .map_err(|e| Error::Internal(format!("Failed to serialize state: {e}")))?;

        if let Some(parent) = self.path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).await.map_err(|e| {
                    Error::Internal(format!(
                        "Failed to create parent directories for state: {e}"
                    ))
                })?;
            }
        }

        fs::write(&self.path, content)
            .await
            .map_err(|e| Error::Internal(format!("Failed to write state file: {e}")))?;

        Ok(())
    }
}
