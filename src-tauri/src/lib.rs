mod config;
mod config_store;
mod diagnostics;
mod installer;
mod models;
mod runtime;
mod schema;
mod secrets;
mod service_manager;
mod updater;
mod projects;

use crate::service_manager::ServiceManager;
use std::sync::{Arc, Mutex};
use tauri::State;
use std::collections::HashMap;

#[derive(Debug)]
struct AppState {
    services: Arc<Mutex<ServiceManager>>,
    config: Arc<Mutex<config_store::ConfigStore>>,
    secrets: Arc<Mutex<secrets::SecretsStore>>,
    installer: Arc<installer::Installer>,
    updater: Arc<updater::Updater>,
    update_progress: Arc<Mutex<updater::UpdateProgress>>,
    projects: Arc<Mutex<projects::ProjectStore>>,
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
    if let Some(port) = service_config.ports.get("main") {
        if *port == 0 {
            let allocated = config.allocate_port(&id)?;
            service_config.ports.insert("main".to_string(), allocated);
            let _ = config.save_service_config(&service_config);
        }
    }
    services.start_with_config(&id, service_config)
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
    if let Some(port) = service_config.ports.get("main") {
        if *port == 0 {
            let allocated = config.allocate_port(&id)?;
            service_config.ports.insert("main".to_string(), allocated);
            let _ = config.save_service_config(&service_config);
        }
    }
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
    if let Some(port) = service_config.ports.get("main") {
        if *port == 0 {
            let allocated = config.allocate_port(&id)?;
            service_config.ports.insert("main".to_string(), allocated);
            let _ = config.save_service_config(&service_config);
        }
    }
    services.restart_with_config(&id, service_config)
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
fn config_set_app(
    state: State<'_, AppState>,
    app: config_store::AppConfig,
) -> Result<(), String> {
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

#[tauri::command]
fn secrets_get(state: State<'_, AppState>, key: String) -> Option<String> {
    let secrets = state.secrets.lock().expect("secrets lock");
    secrets.get(&key)
}

#[tauri::command]
fn secrets_set(state: State<'_, AppState>, key: String, value: String) {
    let mut secrets = state.secrets.lock().expect("secrets lock");
    secrets.set(&key, &value);
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
    let services = state.services.lock().expect("service manager lock");
    let states = services.list().clone();
    let logs = services.snapshot_logs();
    diagnostics::write_bundle(".", app_config, states, logs, service_configs)
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
fn updater_check(state: State<'_, AppState>) -> updater::UpdateStatus {
    state.updater.check()
}

#[tauri::command]
fn updater_apply(state: State<'_, AppState>) {
    let progress = Arc::clone(&state.update_progress);
    state.updater.apply(progress);
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
    let value = serde_json::to_value(project).map_err(|e| e.to_string())?;
    store.save_raw(&value)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let runtime = runtime::RuntimeManager::new(".");
    let definitions = config::default_services();
    let log_root = std::path::PathBuf::from("app/logs/services");
    let service_manager = ServiceManager::new(runtime, definitions, log_root);
    let config_store = config_store::ConfigStore::new(".");
    let secrets_store = secrets::SecretsStore::new();
    let installer = Arc::new(installer::Installer::new());
    let updater = Arc::new(updater::Updater::new());
    let update_progress = Arc::new(Mutex::new(updater::default_progress()));
    let projects = Arc::new(Mutex::new(projects::ProjectStore::new(".")));
    if let Err(err) = config_store.load_app_config() {
        eprintln!("app config invalid: {err}");
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
        })
        .invoke_handler(tauri::generate_handler![
            services_list,
            services_start,
            services_stop,
            services_restart,
            services_apply_config,
            services_logs,
            services_log_path,
            services_health,
            health_summary,
            config_get_app,
            config_set_app,
            config_get_ports,
            config_set_ports,
            config_get_service,
            config_set_service,
            config_reset_service,
            secrets_get,
            secrets_set,
            diagnostics_create,
            installer_status,
            installer_start,
            updater_check,
            updater_apply,
            updater_progress,
            projects_list,
            projects_save
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
