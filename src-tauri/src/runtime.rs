use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::models::PortDef;
use crate::installer;
use once_cell::sync::Lazy;
use std::sync::Mutex;
use zip::ZipArchive;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeManifest {
    pub version: String,
    pub services: Vec<ServiceBinary>,
    pub bundle: BundleInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleInfo {
    #[serde(rename = "createdAt")]
    pub created_at: String,
    pub source: String,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceBinary {
    pub name: String,
    pub version: String,
    pub os: String,
    pub arch: String,
    pub checksum: String,
    pub size: u64,
    #[serde(rename = "binPath")]
    pub bin_path: String,
    #[serde(rename = "defaultPorts")]
    pub default_ports: Vec<PortDef>,
    pub env: HashMap<String, String>,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeDownloadStatus {
    pub phase: String,
    pub progress: f32,
    pub service: Option<String>,
    pub error: Option<String>,
}

static DOWNLOAD_STATUS: Lazy<Mutex<RuntimeDownloadStatus>> = Lazy::new(|| {
    Mutex::new(RuntimeDownloadStatus {
        phase: "idle".to_string(),
        progress: 0.0,
        service: None,
        error: None,
    })
});

#[derive(Debug, Clone)]
pub struct RuntimeManager {
    pub root: PathBuf,
}

impl RuntimeManager {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn ensure_manifest(&self) -> Result<RuntimeManifest, String> {
        let path = self.manifest_path();
        if path.exists() {
            return self.load_manifest();
        }
        let manifest = default_manifest();
        self.write_manifest(&manifest)?;
        Ok(manifest)
    }

    pub fn load_manifest(&self) -> Result<RuntimeManifest, String> {
        let path = self.manifest_path();
        let raw = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
        serde_json::from_str(&raw).map_err(|e| e.to_string())
    }

    pub fn write_manifest(&self, manifest: &RuntimeManifest) -> Result<(), String> {
        let path = self.manifest_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let raw = serde_json::to_string_pretty(manifest).map_err(|e| e.to_string())?;
        std::fs::write(path, raw).map_err(|e| e.to_string())
    }

    pub fn manifest_path(&self) -> PathBuf {
        self.root.join("runtime/manifest.json")
    }

    pub fn scoped_path(&self, binary_path: &Path) -> String {
        let mut entries = Vec::new();
        if let Some(parent) = binary_path.parent() {
            entries.push(parent.to_path_buf());
        }
        entries.push(self.root.join("runtime/bin"));
        let sep = if cfg!(target_os = "windows") { ";" } else { ":" };
        let mut path_value = entries
            .into_iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join(sep);
        if let Ok(existing) = std::env::var("PATH") {
            if !existing.is_empty() {
                path_value.push_str(sep);
                path_value.push_str(&existing);
            }
        }
        path_value
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

    pub fn get_manifest(&self) -> Result<RuntimeManifest, String> {
        self.ensure_manifest()
    }

    pub fn ensure_service(&self, name: &str, version: &str) -> Result<ServiceBinary, String> {
        let mut manifest = self.ensure_manifest()?;
        let (os, arch) = os_arch_tag();
        let mut service = manifest
            .services
            .iter()
            .find(|entry| {
                entry.name == name && entry.version == version && entry.os == os && entry.arch == arch
            })
            .cloned();
        if service.is_none() {
            if let Ok(updated) = self.refresh_manifest() {
                manifest = updated;
                service = manifest
                    .services
                    .iter()
                    .find(|entry| {
                        entry.name == name && entry.version == version && entry.os == os && entry.arch == arch
                    })
                    .cloned();
            }
        }
        let service = service.ok_or_else(|| "service not found in manifest".to_string())?;

        let bin_path = self.root.join(&service.bin_path);
        if bin_path.exists() {
            return Ok(service);
        }

        set_download_status("downloading", 0.0, Some(name.to_string()), None);
        let cache_archive = self
            .root
            .join(format!("runtime/cache/{name}-{version}-{os}-{arch}.zip"));
        if cache_archive.exists() {
            set_download_status("extracting", 0.5, Some(name.to_string()), None);
            extract_zip(&cache_archive, &self.root)?;
        } else if let Ok(template) = std::env::var("KOJIBOX_RUNTIME_SOURCE_URL_TEMPLATE") {
            if !template.trim().is_empty() {
                let url = template
                    .replace("{name}", name)
                    .replace("{version}", version)
                    .replace("{os}", &os)
                    .replace("{arch}", &arch);
                installer::download_with_resume(&url, &cache_archive, &service.checksum)?;
                set_download_status("extracting", 0.6, Some(name.to_string()), None);
                extract_zip(&cache_archive, &self.root)?;
            }
        }

        if bin_path.exists() {
            ensure_executable(&bin_path)?;
            set_download_status("complete", 1.0, Some(name.to_string()), None);
            return Ok(service);
        }
        set_download_status(
            "error",
            0.0,
            Some(name.to_string()),
            Some("runtime binary not available".to_string()),
        );
        Err("runtime binary not available".to_string())
    }

    pub fn refresh_manifest(&self) -> Result<RuntimeManifest, String> {
        let url = match std::env::var("KOJIBOX_RUNTIME_MANIFEST_URL") {
            Ok(value) if !value.trim().is_empty() => value,
            // Check sources.json
            _ => {
                let sources_path = self.root.join("runtime/sources.json");
                if sources_path.exists() {
                    if let Ok(raw) = std::fs::read_to_string(&sources_path) {
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&raw) {
                            if let Some(url) = json.get("manifestUrl").and_then(|s| s.as_str()) {
                                url.to_string()
                            } else {
                                return self.ensure_manifest();
                            }
                        } else {
                            return self.ensure_manifest();
                        }
                    } else {
                        return self.ensure_manifest();
                    }
                } else {
                    return self.ensure_manifest();
                }
            }
        };
        let checksum = std::env::var("KOJIBOX_RUNTIME_MANIFEST_CHECKSUM").unwrap_or_default();
        let cache_path = self.root.join("runtime/cache/manifest.json");
        installer::download_with_resume(&url, &cache_path, &checksum)?;
        let raw = std::fs::read_to_string(&cache_path).map_err(|e| e.to_string())?;
        let manifest: RuntimeManifest = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
        self.write_manifest(&manifest)?;
        Ok(manifest)
    }

    pub fn list_installed_versions(&self, service: &str) -> Vec<String> {
        let service_dir = self.root.join("runtime/bin").join(service);
        let mut versions = Vec::new();
        if let Ok(entries) = std::fs::read_dir(service_dir) {
            for entry in entries.flatten() {
                if let Ok(ft) = entry.file_type() {
                    if ft.is_dir() {
                        if let Ok(name) = entry.file_name().into_string() {
                            // Check if it looks like a version (simple check)
                            if name.chars().next().map_or(false, |c| c.is_ascii_digit()) {
                                versions.push(name);
                            }
                        }
                    }
                }
            }
        }
        versions.sort_by(|a, b| {
            // Reverse sort to put newest first roughly
            b.cmp(a)
        });
        versions
    }
}

pub fn default_manifest() -> RuntimeManifest {
    let now = now_ts();
    RuntimeManifest {
        version: "1".to_string(),
        services: default_service_binaries(),
        bundle: BundleInfo {
            created_at: now,
            source: "installer".to_string(),
            signature: "".to_string(),
        },
    }
}

pub fn default_versions() -> HashMap<String, String> {
    let mut versions = HashMap::new();
    versions.insert("php".to_string(), "8.3.2".to_string());
    versions.insert("node".to_string(), "20.11.1".to_string());
    versions.insert("postgres".to_string(), "16.2".to_string());
    versions.insert("mariadb".to_string(), "10.11.6".to_string());
    versions.insert("mailpit".to_string(), "1.15.0".to_string());
    versions
}

pub fn os_arch_tag() -> (String, String) {
    let os = match std::env::consts::OS {
        "macos" => "macos",
        "windows" => "windows",
        "linux" => "linux",
        other => other,
    };
    let arch = match std::env::consts::ARCH {
        "x86_64" => "x64",
        "aarch64" => "arm64",
        other => other,
    };
    (os.to_string(), arch.to_string())
}

pub fn bin_path_for(service: &str, version: &str) -> String {
    let ext = if cfg!(target_os = "windows") { ".exe" } else { "" };
    let (os, arch) = os_arch_tag();
    format!("runtime/bin/{service}/{version}/{os}-{arch}/{service}{ext}")
}

pub fn default_service_binaries() -> Vec<ServiceBinary> {
    let (os, arch) = os_arch_tag();
    let versions = default_versions();
    let mut services = Vec::new();
    let php_version = versions.get("php").cloned().unwrap_or_default();
    services.push(ServiceBinary {
        name: "php".to_string(),
        version: php_version.clone(),
        os: os.clone(),
        arch: arch.clone(),
        checksum: "".to_string(),
        size: 0,
        bin_path: bin_path_for("php", &php_version),
        default_ports: vec![PortDef {
            name: "fpm".to_string(),
            port: 9000,
            protocol: "tcp".to_string(),
        }],
        env: {
            let mut env = HashMap::new();
            env.insert("PHP_INI_SCAN_DIR".to_string(), "runtime/config/php".to_string());
            env
        },
        args: Vec::new(),
    });
    let node_version = versions.get("node").cloned().unwrap_or_default();
    services.push(ServiceBinary {
        name: "node".to_string(),
        version: node_version.clone(),
        os: os.clone(),
        arch: arch.clone(),
        checksum: "".to_string(),
        size: 0,
        bin_path: bin_path_for("node", &node_version),
        default_ports: vec![PortDef {
            name: "http".to_string(),
            port: 3000,
            protocol: "tcp".to_string(),
        }],
        env: HashMap::new(),
        args: Vec::new(),
    });
    let postgres_version = versions.get("postgres").cloned().unwrap_or_default();
    services.push(ServiceBinary {
        name: "postgres".to_string(),
        version: postgres_version.clone(),
        os: os.clone(),
        arch: arch.clone(),
        checksum: "".to_string(),
        size: 0,
        bin_path: bin_path_for("postgres", &postgres_version),
        default_ports: vec![PortDef {
            name: "db".to_string(),
            port: 5432,
            protocol: "tcp".to_string(),
        }],
        env: {
            let mut env = HashMap::new();
            env.insert("PGDATA".to_string(), "runtime/data/postgres".to_string());
            env
        },
        args: vec!["-D".to_string(), "runtime/data/postgres".to_string()],
    });
    let mariadb_version = versions.get("mariadb").cloned().unwrap_or_default();
    services.push(ServiceBinary {
        name: "mariadb".to_string(),
        version: mariadb_version.clone(),
        os: os.clone(),
        arch: arch.clone(),
        checksum: "".to_string(),
        size: 0,
        bin_path: bin_path_for("mariadb", &mariadb_version),
        default_ports: vec![PortDef {
            name: "db".to_string(),
            port: 3306,
            protocol: "tcp".to_string(),
        }],
        env: HashMap::new(),
        args: Vec::new(),
    });
    let mailpit_version = versions.get("mailpit").cloned().unwrap_or_default();
    services.push(ServiceBinary {
        name: "mailpit".to_string(),
        version: mailpit_version.clone(),
        os,
        arch,
        checksum: "".to_string(),
        size: 0,
        bin_path: bin_path_for("mailpit", &mailpit_version),
        default_ports: vec![PortDef {
            name: "http".to_string(),
            port: 8025,
            protocol: "tcp".to_string(),
        }],
        env: HashMap::new(),
        args: Vec::new(),
    });
    services
}

fn now_ts() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    secs.to_string()
}

pub fn download_status() -> RuntimeDownloadStatus {
    DOWNLOAD_STATUS
        .lock()
        .expect("download status lock")
        .clone()
}

fn set_download_status(
    phase: &str,
    progress: f32,
    service: Option<String>,
    error: Option<String>,
) {
    let mut status = DOWNLOAD_STATUS.lock().expect("download status lock");
    status.phase = phase.to_string();
    status.progress = progress;
    status.service = service;
    status.error = error;
}

fn extract_zip(archive: &PathBuf, root: &PathBuf) -> Result<(), String> {
    let file = std::fs::File::open(archive).map_err(|e| e.to_string())?;
    let mut zip = ZipArchive::new(file).map_err(|e| e.to_string())?;
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i).map_err(|e| e.to_string())?;
        let name = entry.name().to_string();
        if name.contains("..") {
            return Err("invalid archive entry".to_string());
        }
        let out_path = root.join(&name);
        if entry.is_dir() {
            std::fs::create_dir_all(&out_path).map_err(|e| e.to_string())?;
        } else {
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            let mut outfile = std::fs::File::create(&out_path).map_err(|e| e.to_string())?;
            std::io::copy(&mut entry, &mut outfile).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn ensure_executable(path: &PathBuf) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perm = std::fs::metadata(path)
            .map_err(|e| e.to_string())?
            .permissions();
        perm.set_mode(0o755);
        std::fs::set_permissions(path, perm).map_err(|e| e.to_string())?;
    }
    Ok(())
}
