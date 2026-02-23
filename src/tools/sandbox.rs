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

    pub fn base_dir(&self) -> &Path {
        &self.base_dir
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_resolve_valid_path() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let base_path = temp_dir.path().to_path_buf();
        let sandbox = PathSandbox::new(base_path.clone());

        // Create a file inside the temp dir
        let file_path = base_path.join("test.txt");
        fs::write(&file_path, "content")?;

        let resolved = sandbox.resolve("test.txt")?;
        assert_eq!(resolved, file_path.canonicalize()?);

        Ok(())
    }

    #[test]
    fn test_resolve_nested_path() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let base_path = temp_dir.path().to_path_buf();
        let sandbox = PathSandbox::new(base_path.clone());

        // Create a nested directory and file
        let nested_dir = base_path.join("subdir");
        fs::create_dir(&nested_dir)?;
        let file_path = nested_dir.join("test.txt");
        fs::write(&file_path, "content")?;

        let resolved = sandbox.resolve("subdir/test.txt")?;
        assert_eq!(resolved, file_path.canonicalize()?);

        Ok(())
    }

    #[test]
    fn test_prevent_absolute_path() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let sandbox = PathSandbox::new(temp_dir.path().to_path_buf());

        // Try to resolve an absolute path (e.g., /etc/passwd or similar)
        // Using temp_dir itself as absolute path target to be safe and portable
        let abs_path = temp_dir.path().to_string_lossy();
        let result = sandbox.resolve(&abs_path);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Absolute paths are not allowed"
        );

        Ok(())
    }

    #[test]
    fn test_prevent_traversal_outside() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let sandbox = PathSandbox::new(temp_dir.path().to_path_buf());

        // Try to go up ..
        let result = sandbox.resolve("../outside.txt");

        // The error might be "Failed to resolve path" (if it doesn't exist) or "Path is outside the sandbox"
        // In this test environment, .. likely resolves to /tmp which exists.
        // If it resolves, it should be caught by the starts_with check.
        // If it doesn't resolve (file not found), that's also fine but less specific.

        // Let's rely on the fact that if it resolves, it must be outside.
        if let Ok(path) = result {
            panic!("Should have failed, but resolved to: {:?}", path);
        }

        Ok(())
    }

    #[test]
    #[cfg(unix)]
    fn test_symlink_outside() -> Result<()> {
        use std::os::unix::fs::symlink;

        let temp_dir = TempDir::new()?;
        let base_path = temp_dir.path().to_path_buf();
        let sandbox = PathSandbox::new(base_path.clone());

        // Create a file outside
        let outside_dir = TempDir::new()?;
        let outside_file = outside_dir.path().join("secret.txt");
        fs::write(&outside_file, "secret")?;

        // Create a symlink inside pointing to outside
        let link_path = base_path.join("link_to_secret");
        symlink(&outside_file, &link_path)?;

        let result = sandbox.resolve("link_to_secret");

        // Should fail because canonical path is outside base_dir
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Path is outside the sandbox"
        );

        Ok(())
    }

    #[test]
    fn test_unicode_paths() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let base_path = temp_dir.path().to_path_buf();
        let sandbox = PathSandbox::new(base_path.clone());

        let filename = "ümlaut.txt";
        let file_path = base_path.join(filename);
        fs::write(&file_path, "content")?;

        let resolved = sandbox.resolve(filename)?;
        assert_eq!(resolved, file_path.canonicalize()?);

        Ok(())
    }
}
