use serde::Serialize;
use std::sync::{Arc, Mutex};
use std::fs;
use std::path::PathBuf;

use crate::runtime;
use reqwest::blocking::Client;
use reqwest::header::RANGE;
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use zip::ZipArchive;

#[derive(Debug, Clone, Serialize)]
pub struct InstallerStatus {
    pub phase: String,
    pub progress: f32,
}

#[derive(Debug)]
pub struct Installer {
    status: Arc<Mutex<InstallerStatus>>,
}

impl Installer {
    pub fn new() -> Self {
        Self {
            status: Arc::new(Mutex::new(InstallerStatus {
                phase: "idle".to_string(),
                progress: 0.0,
            })),
        }
    }

    pub fn status(&self) -> InstallerStatus {
        self.status.lock().expect("installer lock").clone()
    }

    pub fn start(&self) {
        let status = Arc::clone(&self.status);
        std::thread::spawn(move || {
            {
                let mut state = status.lock().expect("installer lock");
                state.phase = "downloading".to_string();
                state.progress = 0.0;
            }
            for i in 1..=10 {
                std::thread::sleep(std::time::Duration::from_millis(300));
                let mut state = status.lock().expect("installer lock");
                state.progress = i as f32 / 10.0;
                if i == 10 {
                    state.phase = "verifying".to_string();
                }
            }
            let _ = init_runtime_layout(".");
            let _ = download_runtime_if_configured(".");
            let mut state = status.lock().expect("installer lock");
            state.phase = "complete".to_string();
            state.progress = 1.0;
        });
    }
}

fn init_runtime_layout(root: &str) -> Result<(), String> {
    let base = PathBuf::from(root);
    let runtime = base.join("runtime");
    let manifest = runtime.join("manifest.json");
    fs::create_dir_all(runtime.join("bin")).map_err(|e| e.to_string())?;
    fs::create_dir_all(runtime.join("data")).map_err(|e| e.to_string())?;
    fs::create_dir_all(runtime.join("logs")).map_err(|e| e.to_string())?;
    fs::create_dir_all(runtime.join("config")).map_err(|e| e.to_string())?;
    fs::create_dir_all(runtime.join("temp")).map_err(|e| e.to_string())?;
    fs::create_dir_all(runtime.join("cache")).map_err(|e| e.to_string())?;

    for service in ["php", "node", "postgres", "mariadb", "mailpit"] {
        fs::create_dir_all(runtime.join("data").join(service)).map_err(|e| e.to_string())?;
        fs::create_dir_all(runtime.join("logs").join(service)).map_err(|e| e.to_string())?;
        fs::create_dir_all(runtime.join("config").join(service)).map_err(|e| e.to_string())?;
    }

    let versions = runtime::default_versions();
    let (os, arch) = runtime::os_arch_tag();
    for service in ["php", "node", "postgres", "mariadb", "mailpit"] {
        if let Some(version) = versions.get(service) {
            let bin_dir = runtime
                .join("bin")
                .join(service)
                .join(version)
                .join(format!("{os}-{arch}"));
            fs::create_dir_all(bin_dir).map_err(|e| e.to_string())?;
        }
    }

    write_templates(&runtime)?;

    if !manifest.exists() {
        let manifest_value = runtime::default_manifest();
        let raw = serde_json::to_string_pretty(&manifest_value).map_err(|e| e.to_string())?;
        fs::write(manifest, raw).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn write_templates(runtime: &PathBuf) -> Result<(), String> {
    let templates = [
        ("php", "default.conf", "# php default config\n"),
        ("php", "php.ini", "display_errors=On\nerror_reporting=E_ALL\n"),
        (
            "php",
            "php-fpm.conf",
            "[global]\nerror_log=runtime/logs/php/php-fpm.log\ninclude=runtime/config/php/pool.d/*.conf\n",
        ),
        (
            "php",
            "pool.d/www.conf",
            "[www]\nlisten=127.0.0.1:9000\npm=dynamic\npm.max_children=5\npm.start_servers=2\npm.min_spare_servers=1\npm.max_spare_servers=3\n",
        ),
        ("node", "default.conf", "# node default config\n"),
        (
            "postgres",
            "postgresql.conf",
            "port=5432\nmax_connections=50\nlogging_collector=on\nlog_directory='runtime/logs/postgres'\n",
        ),
        (
            "postgres",
            "pg_hba.conf",
            "local all all trust\nhost all all 127.0.0.1/32 trust\nhost all all ::1/128 trust\n",
        ),
        (
            "mariadb",
            "my.cnf",
            "[mysqld]\nport=3306\ndatadir=runtime/data/mariadb\nlog_error=runtime/logs/mariadb/mariadb.log\n",
        ),
        ("mailpit", "default.conf", "# mailpit default config\n"),
    ];
    for (service, filename, content) in templates {
        let path = runtime.join("config").join(service).join(filename);
        if path.exists() {
            continue;
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        fs::write(path, content).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn download_runtime_if_configured(root: &str) -> Result<(), String> {
    let url = match std::env::var("KOJIBOX_RUNTIME_URL") {
        Ok(value) if !value.trim().is_empty() => value,
        _ => return Ok(()),
    };
    let checksum = std::env::var("KOJIBOX_RUNTIME_CHECKSUM").unwrap_or_default();
    let base = PathBuf::from(root);
    let cache_path = base.join("runtime/cache/runtime.zip");
    download_with_resume(&url, &cache_path, &checksum)
}

pub fn download_with_resume(url: &str, dest: &PathBuf, checksum: &str) -> Result<(), String> {
    let client = Client::new();
    let mut request = client.get(url);
    let mut file = if dest.exists() {
        let size = dest.metadata().map_err(|e| e.to_string())?.len();
        request = request.header(RANGE, format!("bytes={size}-"));
        fs::OpenOptions::new()
            .append(true)
            .open(dest)
            .map_err(|e| e.to_string())?
    } else {
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        fs::File::create(dest).map_err(|e| e.to_string())?
    };
    let mut response = request.send().map_err(|e| e.to_string())?;
    if !response.status().is_success() {
        return Err(format!("download failed: {}", response.status()));
    }
    let mut buffer = [0u8; 8192];
    loop {
        let read = response.read(&mut buffer).map_err(|e| e.to_string())?;
        if read == 0 {
            break;
        }
        file.write_all(&buffer[..read]).map_err(|e| e.to_string())?;
    }
    verify_checksum(dest, checksum)?;
    Ok(())
}

pub fn verify_checksum(path: &PathBuf, checksum: &str) -> Result<(), String> {
    let expected = checksum.strip_prefix("sha256:").unwrap_or(checksum).to_string();
    if expected.trim().is_empty() {
        return Ok(());
    }
    let mut file = fs::File::open(path).map_err(|e| e.to_string())?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    loop {
        let read = file.read(&mut buffer).map_err(|e| e.to_string())?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    let digest = hasher.finalize();
    let actual = hex::encode(digest);
    if actual != expected {
        return Err("checksum mismatch".to_string());
    }
    Ok(())
}

pub fn apply_archive_with_rollback(
    root: &PathBuf,
    archive: &PathBuf,
    checksum: &str,
) -> Result<(), String> {
    verify_checksum(archive, checksum)?;
    let staging_root = root.join("runtime_staging");
    if staging_root.exists() {
        fs::remove_dir_all(&staging_root).map_err(|e| e.to_string())?;
    }
    fs::create_dir_all(&staging_root).map_err(|e| e.to_string())?;
    extract_zip(archive, &staging_root)?;

    let staging_runtime = staging_root.join("runtime");
    if !staging_runtime.exists() {
        return Err("offline pack missing runtime/ directory".to_string());
    }

    let runtime_root = root.join("runtime");
    let backup_root = root.join("runtime_backups");
    fs::create_dir_all(&backup_root).map_err(|e| e.to_string())?;
    let backup_path = backup_root.join(format!("runtime-{}", now_ts()));
    if runtime_root.exists() {
        fs::rename(&runtime_root, &backup_path).map_err(|e| e.to_string())?;
    }
    let move_result = fs::rename(&staging_runtime, &runtime_root);
    if let Err(err) = move_result {
        if backup_path.exists() {
            let _ = fs::rename(&backup_path, &runtime_root);
        }
        return Err(err.to_string());
    }
    let manifest_from_pack = staging_root.join("manifest.json");
    if manifest_from_pack.exists() {
        let _ = fs::copy(&manifest_from_pack, runtime_root.join("manifest.json"));
    }
    let _ = fs::remove_dir_all(&staging_root);
    Ok(())
}

fn extract_zip(archive_path: &PathBuf, dest: &PathBuf) -> Result<(), String> {
    let file = fs::File::open(archive_path).map_err(|e| e.to_string())?;
    let mut archive = ZipArchive::new(file).map_err(|e| e.to_string())?;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
        let name = entry.name().to_string();
        if name.contains("..") {
            return Err("invalid archive entry".to_string());
        }
        let out_path = dest.join(&name);
        if entry.is_dir() {
            fs::create_dir_all(&out_path).map_err(|e| e.to_string())?;
        } else {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            let mut outfile = fs::File::create(&out_path).map_err(|e| e.to_string())?;
            std::io::copy(&mut entry, &mut outfile).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn now_ts() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    secs.to_string()
}
