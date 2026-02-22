use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};

pub struct PathSandbox {
    base_dir: PathBuf,
}

impl PathSandbox {
    pub fn new(base_dir: PathBuf) -> Self {
        let base_dir = base_dir.canonicalize().unwrap_or(base_dir);
        Self { base_dir }
    }

    pub fn resolve(&self, relative_path: &str) -> Result<PathBuf> {
        let rel_path = Path::new(relative_path);

        // Prevent absolute paths
        if rel_path.is_absolute() {
            return Err(anyhow!("Absolute paths are not allowed"));
        }

        // Join and canonicalize
        let joined = self.base_dir.join(rel_path);
        let canonical = joined
            .canonicalize()
            .map_err(|e| anyhow!("Failed to resolve path: {}", e))?;

        // Check if it starts with base_dir
        if !canonical.starts_with(&self.base_dir) {
            return Err(anyhow!("Path is outside the sandbox"));
        }

        Ok(canonical)
    }
}
