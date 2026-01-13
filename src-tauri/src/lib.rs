pub mod config;
mod config_store;
mod diagnostics;
pub mod installer;
pub mod models;
pub mod runtime;
mod metrics;
mod schema;
mod secrets;
pub mod service_manager;
mod updater;
mod projects;
pub mod tooling;
mod proxy;
mod telemetry;
mod database;
mod task_manager;

use crate::service_manager::ServiceManager;
use std::sync::{Arc, Mutex};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    Manager, State,
};
use std::collections::HashMap;
use std::process::Command;

#[derive(Debug)]
struct AppState {
    services: Arc<Mutex<ServiceManager>>,
    config: Arc<Mutex<config_store::ConfigStore>>,
    secrets: Arc<Mutex<secrets::SecretsStore>>,
    installer: Arc<installer::Installer>,
    updater: Arc<updater::Updater>,
    update_progress: Arc<Mutex<updater::UpdateProgress>>,
    projects: Arc<Mutex<projects::ProjectStore>>,
    php_config: Arc<Mutex<config::php::PhpConfigManager>>,
    db_manager: Arc<Mutex<database::DatabaseManager>>,
    task_manager: Arc<Mutex<task_manager::TaskManager>>,
}

#[tauri::command]
fn task_list_scripts(state: State<'_, AppState>, path: String) -> Result<HashMap<String, String>, String> {
    let manager = state.task_manager.lock().expect("task manager lock");
    manager.list_scripts(&path)
}

#[tauri::command]
fn task_run(state: State<'_, AppState>, project_id: String, path: String, script: String) -> Result<(), String> {
    let mut manager = state.task_manager.lock().expect("task manager lock");
    manager.run_script(&project_id, &path, &script)
}

#[tauri::command]
fn task_stop(state: State<'_, AppState>, project_id: String) -> Result<(), String> {
    let mut manager = state.task_manager.lock().expect("task manager lock");
    manager.stop_task(&project_id)
}

#[tauri::command]
fn runtime_list_versions(service: String) -> Vec<String> {
    let runtime = runtime::RuntimeManager::new(".");
    runtime.list_installed_versions(&service)
}

#[tauri::command]
fn db_dump(state: State<'_, AppState>, service: String, db_name: String, path: String) -> Result<String, String> {
    let manager = state.db_manager.lock().expect("db manager lock");
    manager.dump(&service, &db_name, std::path::PathBuf::from(path))
}

#[tauri::command]
fn open_terminal(state: State<'_, AppState>) -> Result<(), String> {
    let services = state.services.lock().expect("service lock");
    // Hacky way to get the runtime manager from service manager or create new one
    // Since ServiceManager owns runtime, we can create a temporary one for path resolution
    // or expose it. For now, creating new RuntimeManager is cheap.
    let runtime = runtime::RuntimeManager::new(".");
    let bin_path = runtime.root.join("runtime/bin"); // base path
    
    // We want to construct a PATH that includes all service bin dirs
    // This logic duplicates some of RuntimeManager::scoped_path but for all services
    let mut paths = Vec::new();
    
    // Hardcoded common service names to add to path
    for service in ["php", "node", "composer", "mariadb", "postgres"] {
        // We need to resolve specific version paths.
        // For now, let's just add the generic bin root and rely on 'current' symlinks if we had them,
        // but since we use specific folders, we need to look up versions.
        let versions = runtime::default_versions();
        if let Some(v) = versions.get(service) {
             let p = runtime::bin_path_for(service, v);
             if let Some(parent) = std::path::Path::new(&p).parent() {
                 if let Ok(abs) = std::fs::canonicalize(runtime.root.join(parent)) {
                     paths.push(abs);
                 }
             }
        }
    }

    let sep = if cfg!(target_os = "windows") { ";" } else { ":" };
    let new_path = std::env::join_paths(paths).map_err(|e| e.to_string())?;
    
    let mut current_path = std::env::var_os("PATH").unwrap_or_default();
    current_path.push(sep);
    current_path.push(new_path);

    #[cfg(target_os = "windows")]
    {
        Command::new("powershell")
            .arg("-NoExit")
            .arg("-Command")
            .arg(format!("$env:PATH = '{}'; Write-Host 'Kojibox Terminal Environment'", current_path.to_string_lossy()))
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "macos")]
    {
        // macOS is tricky to inject env var into a new terminal window easily without a script
        // We will try opening Terminal with a command
        let script = format!("export PATH=\"{}\"; clear; echo 'Kojibox Terminal Environment'; $SHELL", current_path.to_string_lossy());
        Command::new("osascript")
            .arg("-e")
            .arg(format!("tell application \"Terminal\" to do script \"{}\"", script))
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "linux")]
    {
        // Try common terminals
        let shell = std::env::var("SHELL").unwrap_or("/bin/bash".to_string());
        // This is generic and might fail on some distros depending on terminal installed
        if Command::new("gnome-terminal").arg("--").arg(&shell).env("PATH", &current_path).spawn().is_err() {
             if Command::new("xterm").arg("-e").arg(&shell).env("PATH", &current_path).spawn().is_err() {
                 return Err("Could not find gnome-terminal or xterm".to_string());
             }
        }
    }

    Ok(())
}

#[tauri::command]
fn services_list(state: State<'_, AppState>) -> Vec<models::ServiceState> {
    let mut services = state.services.lock().expect("service manager lock");
    services.list()
}

#[tauri::command]
fn services_start(state: State<'_, AppState>, id: String) -> Result<models::ServiceState, String> {
    let mut services = state.services.lock().expect("service manager lock");
    let config = state.config.lock().expect("config store lock");
    let mut service_config = config.load_service_config(&id)?;
    resolve_service_ports(&config, &id, &mut service_config)?;
    let result = services.start_with_config(&id, service_config);
    
    if let Ok(ref state) = result {
        telemetry::track_service_start(&id, state.state == "running");
    } else if let Err(ref err) = result {
        telemetry::track_service_error(&id, err);
    }
    
    result
}

#[tauri::command]
fn services_stop(state: State<'_, AppState>, id: String) -> Result<models::ServiceState, String> {
    let mut services = state.services.lock().expect("service manager lock");
    services.stop(&id)
}

#[tauri::command]
fn services_restart(
    state: State<'_, AppState>,
    id: String,
) -> Result<models::ServiceState, String> {
    let mut services = state.services.lock().expect("service manager lock");
    let config = state.config.lock().expect("config store lock");
    let mut service_config = config.load_service_config(&id)?;
    resolve_service_ports(&config, &id, &mut service_config)?;
    services.restart_with_config(&id, service_config)
}

#[tauri::command]
fn services_apply_config(
    state: State<'_, AppState>,
    id: String,
) -> Result<models::ServiceState, String> {
    let mut services = state.services.lock().expect("service manager lock");
    let config = state.config.lock().expect("config store lock");
    let mut service_config = config.load_service_config(&id)?;
    resolve_service_ports(&config, &id, &mut service_config)?;
    services.restart_with_config(&id, service_config)
}

#[tauri::command]
fn services_apply_config_no_restart(
    state: State<'_, AppState>,
    id: String,
) -> Result<models::ServiceState, String> {
    let mut services = state.services.lock().expect("service manager lock");
    let config = state.config.lock().expect("config store lock");
    let service_config = config.load_service_config(&id)?;
    services.apply_config_no_restart(&id, service_config)
}

#[tauri::command]
fn services_logs(
    state: State<'_, AppState>,
    id: String,
    tail: usize,
) -> Vec<models::LogEntry> {
    let services = state.services.lock().expect("service manager lock");
    services.logs(&id, tail)
}

#[tauri::command]
fn services_log_path(state: State<'_, AppState>, id: String) -> String {
    let services = state.services.lock().expect("service manager lock");
    services.log_path(&id)
}

#[tauri::command]
fn logs_export(
    state: State<'_, AppState>,
    service: Option<String>,
    level: Option<String>,
    limit: Option<usize>,
) -> Result<String, String> {
    let services = state.services.lock().expect("service manager lock");
    services.export_logs(
        service.as_deref(),
        level.as_deref(),
        limit.unwrap_or(200),
    )
}

#[tauri::command]
fn logs_clear(
    state: State<'_, AppState>,
    service: Option<String>,
) -> Result<(), String> {
    let services = state.services.lock().expect("service manager lock");
    services.clear_logs(service.as_deref())
}

#[tauri::command]
fn system_open_file(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .arg("/C")
            .arg("start")
            .arg("")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn services_health(state: State<'_, AppState>, id: String) -> Result<String, String> {
    let services = state.services.lock().expect("service manager lock");
    services.health(&id)
}

#[tauri::command]
fn health_summary(state: State<'_, AppState>) -> HashMap<String, String> {
    let services = state.services.lock().expect("service manager lock");
    let mut summary = HashMap::new();
    for def in config::default_services() {
        let status = services.health(&def.id).unwrap_or_else(|_| "error".to_string());
        summary.insert(def.id, status);
    }
    summary
}
#[tauri::command]
fn config_get_app(state: State<'_, AppState>) -> Result<config_store::AppConfig, String> {
    let config = state.config.lock().expect("config store lock");
    config.load_app_config()
}

#[tauri::command]
fn config_app_exists(state: State<'_, AppState>) -> bool {
    let config = state.config.lock().expect("config store lock");
    config.app_config_exists()
}

#[tauri::command]
fn config_set_app(
    state: State<'_, AppState>,
    app: config_store::AppConfig,
) -> Result<(), String> {
    telemetry::set_enabled(app.telemetry_opt_in);
    let config = state.config.lock().expect("config store lock");
    config.save_app_config(&app)
}

#[tauri::command]
fn config_get_ports(state: State<'_, AppState>) -> Result<config_store::PortRegistry, String> {
    let config = state.config.lock().expect("config store lock");
    config.load_port_registry()
}

#[tauri::command]
fn config_set_ports(
    state: State<'_, AppState>,
    registry: config_store::PortRegistry,
) -> Result<(), String> {
    let config = state.config.lock().expect("config store lock");
    config.save_port_registry(&registry)
}

#[tauri::command]
fn config_get_service(
    state: State<'_, AppState>,
    id: String,
) -> Result<config_store::ServiceConfig, String> {
    let config = state.config.lock().expect("config store lock");
    config.load_service_config(&id)
}

#[tauri::command]
fn config_set_service(
    state: State<'_, AppState>,
    service: config_store::ServiceConfig,
) -> Result<(), String> {
    let config = state.config.lock().expect("config store lock");
    config.save_service_config(&service)
}

#[tauri::command]
fn config_reset_service(
    state: State<'_, AppState>,
    id: String,
) -> Result<config_store::ServiceConfig, String> {
    let config = state.config.lock().expect("config store lock");
    config.reset_service_config(&id)
}

fn resolve_service_ports(
    config: &config_store::ConfigStore,
    id: &str,
    service_config: &mut config_store::ServiceConfig,
) -> Result<(), String> {
    if let Some(port) = service_config.ports.get("main") {
        let resolved = config.resolve_port(id, *port)?;
        if resolved != *port {
            service_config.ports.insert("main".to_string(), resolved);
        }
    } else {
        let resolved = config.resolve_port(id, 0)?;
        service_config.ports.insert("main".to_string(), resolved);
    }
    let _ = config.save_service_config(service_config);
    Ok(())
}

#[tauri::command]
fn secrets_get(state: State<'_, AppState>, key: String) -> Option<String> {
    let secrets = state.secrets.lock().expect("secrets lock");
    secrets.get(&key)
}

#[tauri::command]
fn secrets_set(state: State<'_, AppState>, key: String, value: String) -> Result<(), String> {
    let mut secrets = state.secrets.lock().expect("secrets lock");
    secrets.set(&key, &value)
}

#[tauri::command]
fn diagnostics_create(state: State<'_, AppState>) -> Result<String, String> {
    let config = state.config.lock().expect("config store lock");
    let app_config = config.load_app_config_or_default();
    let service_configs = {
        let mut values = Vec::new();
        for def in &config::default_services() {
            if let Ok(cfg) = config.load_service_config(&def.id) {
                if let Ok(value) = serde_json::to_value(cfg) {
                    values.push(value);
                }
            }
        }
        values
    };
    let mut services = state.services.lock().expect("service manager lock");
    let states = services.list().clone();
    let logs = services.snapshot_logs();
    diagnostics::write_bundle(".", app_config, states, logs, service_configs)
}

#[tauri::command]
fn metrics_snapshot(state: State<'_, AppState>) -> Result<metrics::MetricsSnapshot, String> {
    let config = state.config.lock().expect("config store lock");
    metrics::snapshot(&config)
}

#[tauri::command]
fn runtime_get_manifest(_state: State<'_, AppState>) -> Result<runtime::RuntimeManifest, String> {
    let runtime = runtime::RuntimeManager::new(".");
    runtime.get_manifest()
}

#[tauri::command]
fn runtime_ensure_service(
    _state: State<'_, AppState>,
    name: String,
    version: String,
) -> Result<runtime::ServiceBinary, String> {
    let runtime = runtime::RuntimeManager::new(".");
    runtime.ensure_service(&name, &version)
}

#[tauri::command]
fn runtime_refresh_manifest() -> Result<runtime::RuntimeManifest, String> {
    let runtime = runtime::RuntimeManager::new(".");
    runtime.refresh_manifest()
}

#[tauri::command]
fn runtime_download_status() -> runtime::RuntimeDownloadStatus {
    runtime::download_status()
}
#[tauri::command]
fn installer_status(state: State<'_, AppState>) -> installer::InstallerStatus {
    state.installer.status()
}

#[tauri::command]
fn installer_start(state: State<'_, AppState>) {
    state.installer.start();
}

#[tauri::command]
fn installer_import_offline(
    _state: State<'_, AppState>,
    path: String,
    checksum: Option<String>,
) -> Result<(), String> {
    let root = std::path::PathBuf::from(".");
    let archive = std::path::PathBuf::from(path);
    let checksum = checksum.unwrap_or_default();
    installer::apply_archive_with_rollback(&root, &archive, &checksum)
}

#[tauri::command]
fn updater_check(state: State<'_, AppState>) -> Result<updater::UpdateStatus, String> {
    let config = state.config.lock().expect("config store lock");
    let app_config = config.load_app_config_or_default();
    state.updater.check(&app_config)
}

#[tauri::command]
fn updater_apply(state: State<'_, AppState>) {
    let progress = Arc::clone(&state.update_progress);
    if let Ok(archive) = std::env::var("KOJIBOX_UPDATE_ARCHIVE") {
        let checksum = std::env::var("KOJIBOX_UPDATE_CHECKSUM").unwrap_or_default();
        state.updater.apply_with_archive(
            progress,
            std::path::PathBuf::from("."),
            std::path::PathBuf::from(archive),
            checksum,
        );
    } else {
        let config = state.config.lock().expect("config store lock");
        let app_config = config.load_app_config_or_default();
        state
            .updater
            .apply_latest(progress, app_config, std::path::PathBuf::from("."));
    }
}

#[tauri::command]
fn updater_apply_archive(
    state: State<'_, AppState>,
    path: String,
    checksum: Option<String>,
) {
    let progress = Arc::clone(&state.update_progress);
    state.updater.apply_with_archive(
        progress,
        std::path::PathBuf::from("."),
        std::path::PathBuf::from(path),
        checksum.unwrap_or_default(),
    );
}

#[tauri::command]
fn updater_progress(state: State<'_, AppState>) -> updater::UpdateProgress {
    state.update_progress.lock().expect("update progress lock").clone()
}

#[tauri::command]
fn projects_list(state: State<'_, AppState>) -> Result<Vec<projects::ProjectConfig>, String> {
    let store = state.projects.lock().expect("project store lock");
    store.list()
}

#[tauri::command]
fn projects_save(
    state: State<'_, AppState>,
    project: projects::ProjectConfig,
) -> Result<(), String> {
    let store = state.projects.lock().expect("project store lock");
    let value = serde_json::to_value(&project).map_err(|e| e.to_string())?;
    store.save_raw(&value)?;
    
    // Auto-sync domain
    if !project.domain.is_empty() {
        let root = std::path::PathBuf::from(".");
        let mapping = tooling::DomainMapping {
            domain: project.domain.clone(),
            project_id: project.id.clone(),
            target_port: if project.stack == "node" { 3000 } else { 8000 }, // Default ports
        };
        tooling::domains_upsert(&root, mapping)?;
        
        // Auto-apply hosts (best effort, might fail if no admin/sudo, user warned in UI usually)
        let _ = tooling::hosts_apply(&root, tooling::domains_list(&root)?);
    }
    
    Ok(())
}

#[tauri::command]
fn projects_delete(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let store = state.projects.lock().expect("project store lock");
    store.delete(&id)
}

#[tauri::command]
fn domains_list() -> Result<Vec<tooling::DomainMapping>, String> {
    tooling::domains_list(&std::path::PathBuf::from("."))
}

#[tauri::command]
fn domains_upsert(mapping: tooling::DomainMapping) -> Result<(), String> {
    tooling::domains_upsert(&std::path::PathBuf::from("."), mapping)
}

#[tauri::command]
fn domains_remove(domain: String) -> Result<(), String> {
    tooling::domains_remove(&std::path::PathBuf::from("."), &domain)
}

#[tauri::command]
fn proxy_rules() -> Result<Vec<tooling::ProxyRule>, String> {
    tooling::proxy_rules(&std::path::PathBuf::from("."))
}

#[tauri::command]
fn proxy_apply(rules: Vec<tooling::ProxyRule>) -> Result<(), String> {
    tooling::proxy_apply(&std::path::PathBuf::from("."), rules.clone())?;
    proxy::apply_rules(rules)
}

#[tauri::command]
fn hosts_apply(mappings: Vec<tooling::DomainMapping>) -> Result<(), String> {
    tooling::hosts_apply(&std::path::PathBuf::from("."), mappings)
}

#[tauri::command]
fn hosts_rollback() -> Result<(), String> {
    tooling::hosts_rollback(&std::path::PathBuf::from("."))
}

#[tauri::command]
fn certs_generate(domains: Vec<String>) -> Result<tooling::CertMeta, String> {
    tooling::certs_generate(&std::path::PathBuf::from("."), domains)
}

#[tauri::command]
fn certs_list() -> Result<Vec<tooling::CertMeta>, String> {
    tooling::certs_list(&std::path::PathBuf::from("."))
}

#[tauri::command]
fn certs_trust(cert_path: String) -> Result<String, String> {
    tooling::certs_trust(&std::path::PathBuf::from("."), &cert_path)
}

#[tauri::command]
fn certs_trust_os(cert_path: String, apply: bool) -> tooling::TrustResult {
    tooling::certs_trust_os(&cert_path, apply)
}

#[tauri::command]
fn php_extensions_list(state: State<'_, AppState>) -> Result<Vec<(String, bool)>, String> {
    let manager = state.php_config.lock().expect("php config lock");
    manager.list_extensions()
}

#[tauri::command]
fn php_extensions_toggle(
    state: State<'_, AppState>,
    name: String,
    enabled: bool,
) -> Result<(), String> {
    let manager = state.php_config.lock().expect("php config lock");
    manager.toggle_extension(name, enabled)
}

#[tauri::command]
fn check_port_availability(port: u16) -> bool {
    std::net::TcpListener::bind(("127.0.0.1", port)).is_ok()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let runtime = runtime::RuntimeManager::new(".");
    let _ = runtime.ensure_manifest();
    metrics::init_start();
    let definitions = config::default_services();
    let log_root = std::path::PathBuf::from("app/logs/services");
    let service_manager = ServiceManager::new(runtime, definitions, log_root);
    let config_store = config_store::ConfigStore::new(".");
    let secrets_store = secrets::SecretsStore::new(".").unwrap_or_else(|err| {
        eprintln!("secrets init failed: {err}");
        secrets::SecretsStore::new_in_memory()
    });
    let installer = Arc::new(installer::Installer::new());
    let updater = Arc::new(updater::Updater::new());
    let update_progress = Arc::new(Mutex::new(updater::default_progress()));
    let projects = Arc::new(Mutex::new(projects::ProjectStore::new(".")));
    let php_config = Arc::new(Mutex::new(config::php::PhpConfigManager::new(".")));
    let db_manager = Arc::new(Mutex::new(database::DatabaseManager::new(runtime::RuntimeManager::new("."))));
    let task_manager = Arc::new(Mutex::new(task_manager::TaskManager::new(runtime::RuntimeManager::new("."))));

    if let Ok(app_config) = config_store.load_app_config() {
        telemetry::set_enabled(app_config.telemetry_opt_in);
        telemetry::track_app_start(&app_config.update_channel, Vec::new());
    } else {
        eprintln!("app config invalid");
    }
    
    if let Err(err) = config_store.load_port_registry() {
        eprintln!("port registry invalid: {err}");
    }
    for def in &config::default_services() {
        if let Err(err) = config_store.ensure_service_config(&def.id) {
            eprintln!("service config init failed for {}: {}", def.id, err);
        }
    }
    let services = Arc::new(Mutex::new(service_manager));
    let services_bg = Arc::clone(&services);
    std::thread::spawn(move || loop {
        {
            let mut manager = services_bg.lock().expect("service manager lock");
            manager.tick();
        }
        std::thread::sleep(std::time::Duration::from_secs(5));
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            services,
            config: Arc::new(Mutex::new(config_store)),
            secrets: Arc::new(Mutex::new(secrets_store)),
            installer,
            updater,
            update_progress,
            projects,
            php_config,
            db_manager,
            task_manager,
        })
        .invoke_handler(tauri::generate_handler![
            services_list,
            services_start,
            services_stop,
            services_restart,
            services_apply_config,
            services_apply_config_no_restart,
            services_logs,
            services_log_path,
            services_health,
            health_summary,
            logs_export,
            logs_clear,
            system_open_file,
            config_get_app,
            config_app_exists,
            config_set_app,
            config_get_ports,
            config_set_ports,
            config_get_service,
            config_set_service,
            config_reset_service,
            secrets_get,
            secrets_set,
            diagnostics_create,
            metrics_snapshot,
            runtime_get_manifest,
            runtime_ensure_service,
            runtime_refresh_manifest,
            runtime_download_status,
            runtime_list_versions,
            db_dump,
            task_list_scripts,
            task_run,
            task_stop,
            installer_status,
            installer_start,
            installer_import_offline,
            updater_check,
            updater_apply,
            updater_apply_archive,
            updater_progress,
            projects_list,
            projects_save,
            projects_delete,
            domains_list,
            domains_upsert,
            domains_remove,
            proxy_rules,
            proxy_apply,
            hosts_apply,
            hosts_rollback,
            certs_generate,
            certs_list,
            certs_trust,
            certs_trust_os,
            php_extensions_list,
            php_extensions_toggle,
            check_port_availability,
            open_terminal
        ])
        .setup(|app| {
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let show_i = MenuItem::with_id(app, "show", "Show Dashboard", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

            let _tray = TrayIconBuilder::with_id("tray")
                .menu(&menu)
                .on_menu_event(|app: &tauri::AppHandle, event| match event.id.as_ref() {
                    "quit" => {
                        app.exit(0);
                    }
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray: &tauri::tray::TrayIcon, event| match event {
                    TrayIconEvent::Click {
                        button: MouseButton::Left,
                        ..
                    } => {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    _ => {}
                })
                .build(app)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
