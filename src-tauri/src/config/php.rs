use std::path::PathBuf;
use std::fs;
use std::io::Write;

#[derive(Debug)]
pub struct PhpConfigManager {
    root: PathBuf,
}

impl PhpConfigManager {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    fn config_dir(&self) -> PathBuf {
        self.root.join("runtime/config/php")
    }

    fn extensions_file(&self) -> PathBuf {
        self.config_dir().join("extensions.ini")
    }

    pub fn ensure_config_dir(&self) -> Result<(), String> {
        fs::create_dir_all(self.config_dir()).map_err(|e| e.to_string())
    }

    pub fn list_extensions(&self) -> Result<Vec<(String, bool)>, String> {
        self.ensure_config_dir()?;
        let path = self.extensions_file();
        if !path.exists() {
            // Return default common extensions state
            return Ok(default_extensions());
        }

        let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let mut extensions = default_extensions();
        
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() { continue; }
            
            // extension=name or ;extension=name
            let (is_enabled, name) = if line.starts_with(';') {
                (false, line.trim_start_matches(';').trim())
            } else {
                (true, line)
            };
            
            if let Some(stripped) = name.strip_prefix("extension=") {
                if let Some(entry) = extensions.iter_mut().find(|(n, _)| n == stripped) {
                    entry.1 = is_enabled;
                } else {
                    extensions.push((stripped.to_string(), is_enabled));
                }
            }
        }
        
        Ok(extensions)
    }

    pub fn toggle_extension(&self, name: String, enabled: bool) -> Result<(), String> {
        self.ensure_config_dir()?;
        let current = self.list_extensions()?;
        let mut new_list = current.clone();
        
        if let Some(entry) = new_list.iter_mut().find(|(n, _)| n == &name) {
            entry.1 = enabled;
        } else {
            return Err(format!("Extension {} not found in list", name));
        }

        let mut content = String::new();
        content.push_str("; Managed by Kojibox\n");
        for (ext_name, is_enabled) in new_list {
            let prefix = if is_enabled { "" } else { ";" };
            content.push_str(&format!("{}extension={}\n", prefix, ext_name));
        }

        fs::write(self.extensions_file(), content).map_err(|e| e.to_string())?;
        Ok(())
    }
}

fn default_extensions() -> Vec<(String, bool)> {
    vec![
        ("curl".to_string(), true),
        ("fileinfo".to_string(), true),
        ("gd".to_string(), true),
        ("intl".to_string(), true),
        ("mbstring".to_string(), true),
        ("mysqli".to_string(), true),
        ("openssl".to_string(), true),
        ("pdo_mysql".to_string(), true),
        ("pdo_pgsql".to_string(), true),
        ("pdo_sqlite".to_string(), true),
        ("pgsql".to_string(), true),
        ("sockets".to_string(), true),
        ("sqlite3".to_string(), true),
        ("zip".to_string(), true),
        ("xsl".to_string(), false),
        ("soap".to_string(), false),
        ("ftp".to_string(), false),
        ("bz2".to_string(), false),
    ]
}
