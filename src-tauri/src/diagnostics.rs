use crate::config_store::AppConfig;
use crate::models::{LogEntry, ServiceState};
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use zip::write::FileOptions;
use zip::ZipWriter;

#[derive(Debug, Serialize)]
pub struct DiagnosticsBundle {
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "appConfig")]
    pub app_config: AppConfig,
    pub services: Vec<ServiceState>,
    pub logs: HashMap<String, Vec<LogEntry>>,
    #[serde(rename = "logFiles")]
    pub log_files: Vec<String>,
    #[serde(rename = "serviceConfigs")]
    pub service_configs: Vec<serde_json::Value>,
}

pub fn write_bundle(
    root: impl Into<PathBuf>,
    app_config: AppConfig,
    services: Vec<ServiceState>,
    logs: HashMap<String, Vec<LogEntry>>,
    service_configs: Vec<serde_json::Value>,
) -> Result<String, String> {
    let root = root.into();
    let cache_dir = root.join("app/cache");
    fs::create_dir_all(&cache_dir).map_err(|e| e.to_string())?;
    let log_files = collect_log_files(&root.join("app/logs/services"));
    let filename = format!("diag-{}.zip", now_ts());
    let path = cache_dir.join(filename);
    let bundle = DiagnosticsBundle {
        created_at: now_ts(),
        app_config,
        services,
        logs,
        log_files,
        service_configs,
    };
    let raw = serde_json::to_string_pretty(&bundle).map_err(|e| e.to_string())?;
    let file = fs::File::create(&path).map_err(|e| e.to_string())?;
    let mut zip = ZipWriter::new(file);
    let options = FileOptions::default();
    zip.start_file("diagnostics.json", options)
        .map_err(|e| e.to_string())?;
    use std::io::Write;
    zip.write_all(raw.as_bytes()).map_err(|e| e.to_string())?;
    zip.add_directory("logs/", options)
        .map_err(|e| e.to_string())?;
    write_log_files(&mut zip, &root.join("app/logs/services"), options)?;
    zip.finish().map_err(|e| e.to_string())?;
    Ok(path_to_string(&path))
}

fn now_ts() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    secs.to_string()
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

fn collect_log_files(dir: &PathBuf) -> Vec<String> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                files.push(path.to_string_lossy().to_string());
            }
        }
    }
    files
}

fn write_log_files(
    zip: &mut ZipWriter<fs::File>,
    dir: &PathBuf,
    options: FileOptions,
) -> Result<(), String> {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("log");
            let entry_name = format!("logs/{name}");
            let content = fs::read(&path).map_err(|e| e.to_string())?;
            zip.start_file(entry_name, options)
                .map_err(|e| e.to_string())?;
            use std::io::Write;
            zip.write_all(&content).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
