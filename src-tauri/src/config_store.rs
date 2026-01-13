use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::net::TcpListener;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(rename = "schemaVersion")]
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    #[serde(rename = "installPath")]
    pub install_path: String,
    #[serde(rename = "updateChannel")]
    pub update_channel: String,
    #[serde(rename = "telemetryOptIn")]
    pub telemetry_opt_in: bool,
    #[serde(rename = "updateFeedUrl")]
    pub update_feed_url: String,
    #[serde(rename = "updatePublicKeys")]
    pub update_public_keys: Vec<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            schema_version: 1,
            install_path: "app".to_string(),
            update_channel: "stable".to_string(),
            telemetry_opt_in: false,
            update_feed_url: "https://updates.kojibox.dev/feed.json".to_string(),
            update_public_keys: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct ConfigStore {
    root: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortRange {
    pub from: u16,
    pub to: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortRegistry {
    #[serde(rename = "schemaVersion")]
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    pub assigned: HashMap<String, u16>,
    pub ranges: HashMap<String, PortRange>,
}

impl Default for PortRegistry {
    fn default() -> Self {
        let mut ranges = HashMap::new();
        ranges.insert("php".to_string(), PortRange { from: 9000, to: 9099 });
        ranges.insert("node".to_string(), PortRange { from: 3000, to: 3099 });
        ranges.insert("postgres".to_string(), PortRange { from: 5400, to: 5499 });
        ranges.insert("mariadb".to_string(), PortRange { from: 3306, to: 3399 });
        ranges.insert("mailpit".to_string(), PortRange { from: 8025, to: 8099 });
        Self {
            schema_version: 1,
            assigned: HashMap::new(),
            ranges,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    #[serde(rename = "schemaVersion")]
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    pub id: String,
    pub enabled: bool,
    pub version: Option<String>,
    pub ports: HashMap<String, u16>,
    pub env: HashMap<String, String>,
    pub args: Vec<String>,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            schema_version: 1,
            id: "unknown".to_string(),
            enabled: true,
            version: None,
            ports: HashMap::new(),
            env: HashMap::new(),
            args: Vec::new(),
        }
    }
}

fn default_schema_version() -> u32 {
    0
}
impl ConfigStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn load_app_config(&self) -> Result<AppConfig, String> {
        let path = self.app_config_path();
        if !path.exists() {
            return Ok(AppConfig::default());
        }
        let raw = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let value: Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
        crate::schema::validate_app_config(&value)?;
        let mut config: AppConfig = serde_json::from_value(value).map_err(|e| e.to_string())?;
        config = self.migrate_app_config(config)?;
        self.validate_app_config(&config)?;
        Ok(config)
    }

    pub fn save_app_config(&self, config: &AppConfig) -> Result<(), String> {
        self.validate_app_config(config)?;
        let path = self.app_config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let raw = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
        fs::write(path, raw).map_err(|e| e.to_string())
    }

    pub fn app_config_exists(&self) -> bool {
        self.app_config_path().exists()
    }

    pub fn load_port_registry(&self) -> Result<PortRegistry, String> {
        let path = self.port_registry_path();
        if !path.exists() {
            return Ok(PortRegistry::default());
        }
        let raw = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let value: Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
        crate::schema::validate_port_registry(&value)?;
        let mut registry: PortRegistry = serde_json::from_value(value).map_err(|e| e.to_string())?;
        registry = self.migrate_port_registry(registry)?;
        self.validate_port_registry(&registry)?;
        Ok(registry)
    }

    pub fn save_port_registry(&self, registry: &PortRegistry) -> Result<(), String> {
        self.validate_port_registry(registry)?;
        let path = self.port_registry_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let raw = serde_json::to_string_pretty(registry).map_err(|e| e.to_string())?;
        fs::write(path, raw).map_err(|e| e.to_string())
    }

    pub fn allocate_port(&self, service_id: &str) -> Result<u16, String> {
        let mut registry = self.load_port_registry()?;
        if let Some(port) = registry.assigned.get(service_id) {
            if Self::is_port_available(*port) {
                return Ok(*port);
            }
        }
        let range = registry
            .ranges
            .get(service_id)
            .ok_or_else(|| format!("no port range for {service_id}"))?;
        for port in range.from..=range.to {
            if !registry.assigned.values().any(|value| *value == port)
                && Self::is_port_available(port)
            {
                registry.assigned.insert(service_id.to_string(), port);
                self.save_port_registry(&registry)?;
                return Ok(port);
            }
        }
        Err("no available ports".to_string())
    }

    pub fn resolve_port(&self, service_id: &str, desired: u16) -> Result<u16, String> {
        if desired == 0 {
            return self.allocate_port(service_id);
        }
        if Self::is_port_available(desired) {
            return Ok(desired);
        }
        self.allocate_port(service_id)
    }

    pub fn load_service_config(&self, id: &str) -> Result<ServiceConfig, String> {
        let path = self.service_config_path(id);
        if !path.exists() {
            let mut config = ServiceConfig::default();
            config.id = id.to_string();
            return Ok(config);
        }
        let raw = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let value: Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
        crate::schema::validate_service_config(&value)?;
        let mut config: ServiceConfig = serde_json::from_value(value).map_err(|e| e.to_string())?;
        config = self.migrate_service_config(config)?;
        if config.id != id {
            return Err("service config id mismatch".to_string());
        }
        Ok(config)
    }

    pub fn ensure_service_config(&self, id: &str) -> Result<(), String> {
        let path = self.service_config_path(id);
        if path.exists() {
            return Ok(());
        }
        let mut config = ServiceConfig::default();
        config.id = id.to_string();
        self.save_service_config(&config)
    }

    pub fn save_service_config(&self, config: &ServiceConfig) -> Result<(), String> {
        let path = self.service_config_path(&config.id);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let raw = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
        fs::write(path, raw).map_err(|e| e.to_string())
    }

    pub fn reset_service_config(&self, id: &str) -> Result<ServiceConfig, String> {
        let path = self.service_config_path(id);
        if path.exists() {
            fs::remove_file(&path).map_err(|e| e.to_string())?;
        }
        let mut config = ServiceConfig::default();
        config.id = id.to_string();
        self.save_service_config(&config)?;
        Ok(config)
    }

    fn app_config_path(&self) -> PathBuf {
        self.root.join("app/config/app.json")
    }

    fn port_registry_path(&self) -> PathBuf {
        self.root.join("app/config/ports.json")
    }

    fn service_config_path(&self, id: &str) -> PathBuf {
        self.root.join(format!("app/config/services/{id}.json"))
    }

    fn validate_app_config(&self, config: &AppConfig) -> Result<(), String> {
        if config.install_path.trim().is_empty() {
            return Err("installPath is required".to_string());
        }
        if config.update_channel != "stable" && config.update_channel != "beta" {
            return Err("updateChannel must be stable or beta".to_string());
        }
        if !is_valid_url(&config.update_feed_url) {
            return Err("updateFeedUrl must be a valid URL".to_string());
        }
        Ok(())
    }

    fn validate_port_registry(&self, registry: &PortRegistry) -> Result<(), String> {
        for (key, range) in &registry.ranges {
            if range.from == 0 || range.to == 0 || range.from > range.to {
                return Err(format!("invalid port range for {key}"));
            }
        }
        Ok(())
    }

    fn is_port_available(port: u16) -> bool {
        TcpListener::bind(("127.0.0.1", port)).is_ok()
    }

    pub fn load_app_config_or_default(&self) -> AppConfig {
        self.load_app_config().unwrap_or_else(|_| AppConfig::default())
    }

    fn migrate_app_config(&self, mut config: AppConfig) -> Result<AppConfig, String> {
        if config.schema_version == 0 {
            config.schema_version = 1;
            self.save_app_config(&config)?;
        }
        Ok(config)
    }

    fn migrate_port_registry(&self, mut registry: PortRegistry) -> Result<PortRegistry, String> {
        if registry.schema_version == 0 {
            registry.schema_version = 1;
            self.save_port_registry(&registry)?;
        }
        Ok(registry)
    }

    fn migrate_service_config(&self, mut config: ServiceConfig) -> Result<ServiceConfig, String> {
        if config.schema_version == 0 {
            config.schema_version = 1;
            self.save_service_config(&config)?;
        }
        Ok(config)
    }
}

fn is_valid_url(url: &str) -> bool {
    url.starts_with("http://") || url.starts_with("https://")
}
