use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct RuntimeManager {
    root: PathBuf,
}

impl RuntimeManager {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn resolve_binary(&self, binary: &str) -> Result<PathBuf, String> {
        let path = Path::new(binary);
        let resolved = if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.root.join(path)
        };
        if resolved.exists() {
            Ok(resolved)
        } else {
            Err(format!("binary not found: {}", resolved.display()))
        }
    }
}
