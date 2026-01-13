use crate::config_store::ServiceConfig;
use crate::models::{LogEntry, ServiceDefinition, ServiceState};
use crate::runtime;
use crate::runtime::RuntimeManager;
use std::collections::{HashMap, HashSet};
use std::io::{BufRead, BufReader};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub struct ServiceManager {
    runtime: RuntimeManager,
    definitions: Vec<ServiceDefinition>,
    states: HashMap<String, ServiceState>,
    processes: HashMap<String, Child>,
    restart_attempts: HashMap<String, u32>,
    logs: Arc<Mutex<HashMap<String, Vec<LogEntry>>>>,
    log_root: PathBuf,
    log_limit: usize,
    health_retries: u32,
}

impl ServiceManager {
    pub fn new(
        runtime: RuntimeManager,
        definitions: Vec<ServiceDefinition>,
        log_root: PathBuf,
    ) -> Self {
        let _ = std::fs::create_dir_all(&log_root);
        Self {
            runtime,
            definitions,
            states: HashMap::new(),
            processes: HashMap::new(),
            restart_attempts: HashMap::new(),
            logs: Arc::new(Mutex::new(HashMap::new())),
            log_root,
            log_limit: 2000,
            health_retries: 5,
        }
    }

    pub fn list(&mut self) -> Vec<ServiceState> {
        for def in &self.definitions {
            self.states.entry(def.id.clone()).or_insert_with(|| {
                ServiceState {
                    id: def.id.clone(),
                    state: "stopped".to_string(),
                    pid: None,
                    last_error: None,
                    last_updated: now_ts(),
                }
            });
        }
        self.definitions
            .iter()
            .filter_map(|def| self.states.get(&def.id).cloned())
            .collect()
    }

    pub fn start(&mut self, id: &str) -> Result<ServiceState, String> {
        let mut visiting = HashSet::new();
        self.start_with_dependencies(id, &mut visiting)
    }

    pub fn start_with_config(
        &mut self,
        id: &str,
        config: ServiceConfig,
    ) -> Result<ServiceState, String> {
        let mut visiting = HashSet::new();
        self.start_with_dependencies_config(id, &mut visiting, Some(config))
    }

    pub fn restart_with_config(
        &mut self,
        id: &str,
        config: ServiceConfig,
    ) -> Result<ServiceState, String> {
        let _ = self.stop(id);
        self.start_with_config(id, config)
    }

    pub fn apply_config_no_restart(
        &mut self,
        id: &str,
        _config: ServiceConfig,
    ) -> Result<ServiceState, String> {
        let state = self
            .states
            .get(id)
            .cloned()
            .unwrap_or(ServiceState {
                id: id.to_string(),
                state: "stopped".to_string(),
                pid: None,
                last_error: None,
                last_updated: now_ts(),
            });
        self.push_log(id, "info", "applied config without restart");
        Ok(state)
    }

    fn start_with_dependencies(
        &mut self,
        id: &str,
        visiting: &mut HashSet<String>,
    ) -> Result<ServiceState, String> {
        self.start_with_dependencies_config(id, visiting, None)
    }

    fn start_with_dependencies_config(
        &mut self,
        id: &str,
        visiting: &mut HashSet<String>,
        config_override: Option<ServiceConfig>,
    ) -> Result<ServiceState, String> {
        if visiting.contains(id) {
            return Err("dependency cycle detected".to_string());
        }
        visiting.insert(id.to_string());

        let mut def = self
            .definitions
            .iter()
            .find(|d| d.id == id)
            .ok_or_else(|| format!("service not found: {id}"))?
            .clone();

        for dep in &def.depends_on {
            let _ = self.start_with_dependencies_config(dep, visiting, None);
        }

        if let Some(state) = self.states.get(id) {
            if state.state == "running" {
                visiting.remove(id);
                return Ok(state.clone());
            }
        }

        if let Some(config) = config_override {
            if !config.enabled {
                visiting.remove(id);
                return Err("service disabled".to_string());
            }
            for port in def.ports.iter_mut() {
                if let Some(value) = config.ports.get(&port.name) {
                    port.port = *value;
                }
            }
            for (key, value) in config.env {
                def.env.insert(key, value);
            }
            for arg in config.args {
                def.args.push(arg);
            }
            // Override binary if version is specified
            if let Some(ver) = &config.version {
                if !ver.is_empty() {
                    let new_bin_path = runtime::bin_path_for(&def.id, ver);
                    // Check if it exists, otherwise fallback or fail?
                    // We assume it exists if user selected it.
                    // But we need to resolve it relative to root.
                    // Since runtime.resolve_binary does lookup, we just need to pass the relative path string
                    // But wait, resolve_binary expects configured binary path from definition.
                    // We should update def.binary.
                    def.binary = new_bin_path;
                }
            }
            self.push_log(&def.id, "info", "applied service config");
        }

        let binary = match self.runtime.resolve_binary(&def.binary) {
            Ok(path) => path,
            Err(err) => {
                if let Some(version) = runtime::default_versions().get(&def.id).cloned() {
                    let _ = self.runtime.ensure_service(&def.id, &version);
                }
                match self.runtime.resolve_binary(&def.binary) {
                    Ok(path) => path,
                    Err(_) => {
                        visiting.remove(id);
                        return Err(err);
                    }
                }
            }
        };
        self.ensure_service_data(&def, &binary);
        let starting = ServiceState {
            id: def.id.clone(),
            state: "starting".to_string(),
            pid: None,
            last_error: None,
            last_updated: now_ts(),
        };
        self.states.insert(def.id.clone(), starting);
        let mut cmd = Command::new(&binary);
        cmd.args(&def.args)
            .current_dir(&def.cwd)
            .envs(&def.env)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let path_value = self.runtime.scoped_path(&binary);
        cmd.env("PATH", path_value);

        let mut child = match cmd.spawn() {
            Ok(child) => child,
            Err(err) => {
                visiting.remove(id);
                return Err(err.to_string());
            }
        };
        let pid = Some(child.id());
        self.capture_logs(&def.id, &mut child);
        self.processes.insert(def.id.clone(), child);

        let health = self.check_health_with_retries(&def);
        let (state_value, last_error) = match health {
            Ok(()) => ("running".to_string(), None),
            Err(err) => ("error".to_string(), Some(err)),
        };
        let state = ServiceState {
            id: def.id.clone(),
            state: state_value,
            pid,
            last_error,
            last_updated: now_ts(),
        };
        self.states.insert(def.id.clone(), state.clone());
        if state.state == "running" {
            self.restart_attempts.insert(def.id.clone(), 0);
        }
        self.push_log(&def.id, "info", "service started");
        visiting.remove(id);
        Ok(state)
    }

    pub fn stop(&mut self, id: &str) -> Result<ServiceState, String> {
        let child = self.processes.get_mut(id);
        if let Some(child) = child {
            child.kill().map_err(|e| e.to_string())?;
            let _ = child.wait();
        }
        self.processes.remove(id);

        let state = ServiceState {
            id: id.to_string(),
            state: "stopped".to_string(),
            pid: None,
            last_error: None,
            last_updated: now_ts(),
        };
        self.states.insert(id.to_string(), state.clone());
        self.push_log(id, "info", "service stopped");
        Ok(state)
    }

    pub fn restart(&mut self, id: &str) -> Result<ServiceState, String> {
        self.states.insert(
            id.to_string(),
            ServiceState {
                id: id.to_string(),
                state: "restarting".to_string(),
                pid: None,
                last_error: None,
                last_updated: now_ts(),
            },
        );
        let _ = self.stop(id);
        self.start(id)
    }

    pub fn tick(&mut self) {
        self.poll_process_exits();
        self.refresh_health();
    }

    pub fn logs(&self, id: &str, tail: usize) -> Vec<LogEntry> {
        let logs = self
            .logs
            .lock()
            .expect("logs lock")
            .get(id)
            .cloned()
            .unwrap_or_default();
        if logs.len() <= tail {
            logs
        } else {
            logs[logs.len() - tail..].to_vec()
        }
    }

    pub fn log_path(&self, id: &str) -> String {
        self.log_root.join(format!("{id}.log")).to_string_lossy().to_string()
    }

    pub fn snapshot_logs(&self) -> HashMap<String, Vec<LogEntry>> {
        self.logs.lock().expect("logs lock").clone()
    }

    pub fn export_logs(
        &self,
        service: Option<&str>,
        level: Option<&str>,
        limit: usize,
    ) -> Result<String, String> {
        let logs = self.snapshot_logs();
        let mut entries = Vec::new();
        for (id, items) in logs {
            if let Some(filter) = service {
                if filter != id {
                    continue;
                }
            }
            for entry in items {
                if let Some(level_filter) = level {
                    if entry.level != level_filter {
                        continue;
                    }
                }
                entries.push(entry);
            }
        }
        entries.sort_by(|a, b| a.ts.cmp(&b.ts));
        let cap = if limit == 0 { 200 } else { limit };
        let slice = if entries.len() > cap {
            entries[entries.len() - cap..].to_vec()
        } else {
            entries
        };

        let export_dir = self
            .log_root
            .parent()
            .unwrap_or(&self.log_root)
            .join("exports");
        std::fs::create_dir_all(&export_dir).map_err(|e| e.to_string())?;
        let service_tag = service.unwrap_or("all");
        let level_tag = level.unwrap_or("all");
        let filename = format!("{service_tag}-{level_tag}-{}.log", now_ts());
        let path = export_dir.join(filename);
        let mut content = String::new();
        for entry in slice {
            content.push_str(&format!(
                "{} [{}] {} {}\n",
                entry.ts, entry.level, entry.service, entry.message
            ));
        }
        std::fs::write(&path, content).map_err(|e| e.to_string())?;
        Ok(path.to_string_lossy().to_string())
    }

    pub fn clear_logs(&self, service_id: Option<&str>) -> Result<(), String> {
        let mut logs = self.logs.lock().expect("logs lock");
        if let Some(id) = service_id {
            if let Some(buffer) = logs.get_mut(id) {
                buffer.clear();
            }
            let path = self.log_root.join(format!("{id}.log"));
            if path.exists() {
                std::fs::write(&path, "").map_err(|e| e.to_string())?;
            }
        } else {
            logs.clear();
            if let Ok(entries) = std::fs::read_dir(&self.log_root) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map_or(false, |e| e == "log") {
                        let _ = std::fs::write(&path, "");
                    }
                }
            }
        }
        Ok(())
    }

    fn push_log(&self, id: &str, level: &str, message: &str) {
        let entry = LogEntry {
            ts: now_ts(),
            level: level.to_string(),
            service: id.to_string(),
            message: message.to_string(),
            fields: HashMap::new(),
        };
        let mut logs = self.logs.lock().expect("logs lock");
        let buffer = logs.entry(id.to_string()).or_default();
        buffer.push(entry);
        if buffer.len() > self.log_limit {
            let extra = buffer.len() - self.log_limit;
            buffer.drain(0..extra);
        }
        let _ = write_log_line(&self.log_root, id, level, message);
    }

    fn capture_logs(&self, id: &str, child: &mut Child) {
        if let Some(stdout) = child.stdout.take() {
            let logs = self.logs.clone();
            let log_root = self.log_root.clone();
            let id = id.to_string();
            thread::spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines().flatten() {
                    push_log_shared(&logs, &log_root, &id, "info", &line, 2000);
                }
            });
        }

        if let Some(stderr) = child.stderr.take() {
            let logs = self.logs.clone();
            let log_root = self.log_root.clone();
            let id = id.to_string();
            thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines().flatten() {
                    push_log_shared(&logs, &log_root, &id, "error", &line, 2000);
                }
            });
        }
    }

    fn ensure_service_data(&self, def: &ServiceDefinition, binary: &PathBuf) {
        if def.id == "postgres" {
            let data_dir = def
                .env
                .get("PGDATA")
                .cloned()
                .unwrap_or_else(|| "runtime/data/postgres".to_string());
            let data_path = PathBuf::from(&data_dir);
            let _ = std::fs::create_dir_all(&data_path);
            let marker = data_path.join("PG_VERSION");
            if marker.exists() {
                return;
            }
            let initdb = binary
                .parent()
                .map(|parent| parent.join(if cfg!(target_os = "windows") { "initdb.exe" } else { "initdb" }))
                .unwrap_or_else(|| PathBuf::from("initdb"));
            if initdb.exists() {
                let _ = Command::new(initdb)
                    .arg("-D")
                    .arg(&data_dir)
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status();
            } else {
                self.push_log(&def.id, "error", "initdb not found for postgres");
            }
        }
        if def.id == "mariadb" {
            let data_dir = "runtime/data/mariadb".to_string();
            let data_path = PathBuf::from(&data_dir);
            let _ = std::fs::create_dir_all(&data_path);
            let marker = data_path.join("mysql");
            if marker.exists() {
                return;
            }
            let installer = binary
                .parent()
                .map(|parent| {
                    parent.join(if cfg!(target_os = "windows") {
                        "mariadb-install-db.exe"
                    } else {
                        "mariadb-install-db"
                    })
                })
                .unwrap_or_else(|| PathBuf::from("mariadb-install-db"));
            if installer.exists() {
                let _ = Command::new(installer)
                    .arg(format!("--datadir={data_dir}"))
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status();
            } else {
                self.push_log(&def.id, "error", "mariadb-install-db not found");
            }
        }
    }

    fn check_health(&self, def: &ServiceDefinition) -> Result<(), String> {
        let kind = def.health_check.kind.as_str();
        match kind {
            "pid" => Ok(()),
            "port" | "http" => {
                let addr = resolve_addr(&def.health_check.target)?;
                let timeout = Duration::from_millis(def.health_check.timeout_ms);
                TcpStream::connect_timeout(&addr, timeout)
                    .map(|_| ())
                    .map_err(|e| e.to_string())
            }
            _ => Err("unsupported health check".to_string()),
        }
    }

    fn check_health_with_retries(&self, def: &ServiceDefinition) -> Result<(), String> {
        let mut last_error = None;
        for _ in 0..self.health_retries {
            match self.check_health(def) {
                Ok(()) => return Ok(()),
                Err(err) => last_error = Some(err),
            }
            thread::sleep(Duration::from_millis(def.health_check.interval_ms));
        }
        Err(last_error.unwrap_or_else(|| "health check failed".to_string()))
    }

    pub fn health(&self, id: &str) -> Result<String, String> {
        let def = self
            .definitions
            .iter()
            .find(|d| d.id == id)
            .ok_or_else(|| format!("service not found: {id}"))?;
        self.check_health(def).map(|_| "ok".to_string())
    }

    fn refresh_health(&mut self) {
        let ids: Vec<String> = self.states.keys().cloned().collect();
        for id in ids {
            let state = match self.states.get(&id) {
                Some(state) => state.clone(),
                None => continue,
            };
            if state.state != "running" && state.state != "starting" {
                continue;
            }
            let def = match self.definitions.iter().find(|d| d.id == id) {
                Some(def) => def,
                None => continue,
            };
            match self.check_health(def) {
                Ok(()) => {
                    if state.state != "running" {
                        self.states.insert(
                            id.clone(),
                            ServiceState {
                                id: id.clone(),
                                state: "running".to_string(),
                                pid: state.pid,
                                last_error: None,
                                last_updated: now_ts(),
                            },
                        );
                    }
                }
                Err(err) => {
                    self.states.insert(
                        id.clone(),
                        ServiceState {
                            id: id.clone(),
                            state: "error".to_string(),
                            pid: state.pid,
                            last_error: Some(err),
                            last_updated: now_ts(),
                        },
                    );
                }
            }
        }
    }

    fn poll_process_exits(&mut self) {
        let ids: Vec<String> = self.processes.keys().cloned().collect();
        for id in ids {
            let exit = if let Some(child) = self.processes.get_mut(&id) {
                match child.try_wait() {
                    Ok(status) => status,
                    Err(_) => None,
                }
            } else {
                None
            };

            if let Some(status) = exit {
                self.processes.remove(&id);
                let success = status.success();
                let (state, last_error) = if success {
                    ("stopped".to_string(), None)
                } else {
                    ("error".to_string(), Some("process exited".to_string()))
                };
                self.states.insert(
                    id.clone(),
                    ServiceState {
                        id: id.clone(),
                        state,
                        pid: None,
                        last_error,
                        last_updated: now_ts(),
                    },
                );
                let level = if success { "info" } else { "error" };
                self.push_log(&id, level, "process exited");
                if !success {
                    if let Some(def) = self.definitions.iter().find(|d| d.id == id) {
                        let attempt = self.restart_attempts.entry(id.clone()).or_insert(0);
                        if *attempt < def.restart_policy.max_retries {
                            *attempt += 1;
                            self.push_log(&id, "info", "restarting after crash");
                            std::thread::sleep(Duration::from_millis(def.restart_policy.backoff_ms));
                            let _ = self.start(&id);
                        }
                    }
                } else {
                    self.restart_attempts.insert(id.clone(), 0);
                }
            }
        }
    }
}

fn now_ts() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    secs.to_string()
}

fn resolve_addr(target: &str) -> Result<SocketAddr, String> {
    if target.contains("://") {
        let trimmed = target
            .split("://")
            .nth(1)
            .ok_or_else(|| "invalid target".to_string())?;
        let host_port = trimmed.split('/').next().unwrap_or(trimmed);
        return resolve_host_port(host_port);
    }
    resolve_host_port(target)
}

fn resolve_host_port(target: &str) -> Result<SocketAddr, String> {
    target
        .to_socket_addrs()
        .map_err(|e| e.to_string())?
        .next()
        .ok_or_else(|| "unable to resolve target".to_string())
}

fn push_log_shared(
    logs: &Arc<Mutex<HashMap<String, Vec<LogEntry>>>>,
    log_root: &PathBuf,
    id: &str,
    level: &str,
    message: &str,
    log_limit: usize,
) {
    let entry = LogEntry {
        ts: now_ts(),
        level: level.to_string(),
        service: id.to_string(),
        message: message.to_string(),
        fields: HashMap::new(),
    };
    let mut logs = logs.lock().expect("logs lock");
    let buffer = logs.entry(id.to_string()).or_default();
    buffer.push(entry);
    if buffer.len() > log_limit {
        let extra = buffer.len() - log_limit;
        buffer.drain(0..extra);
    }
    let _ = write_log_line(log_root, id, level, message);
}

fn write_log_line(root: &PathBuf, id: &str, level: &str, message: &str) -> Result<(), String> {
    let dir = root;
    let _ = std::fs::create_dir_all(dir);
    let path = dir.join(format!("{id}.log"));
    rotate_log_if_needed(&path, 10 * 1024 * 1024, 3)?;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| e.to_string())?;
    use std::io::Write;
    writeln!(file, "{} [{}] {}", now_ts(), level, message).map_err(|e| e.to_string())
}

fn rotate_log_if_needed(path: &PathBuf, max_size: u64, max_files: u32) -> Result<(), String> {
    if let Ok(metadata) = std::fs::metadata(path) {
        if metadata.len() < max_size {
            return Ok(());
        }
        for index in (1..=max_files).rev() {
            let src = if index == 1 {
                path.clone()
            } else {
                PathBuf::from(format!("{}.{}", path.display(), index - 1))
            };
            let dst = PathBuf::from(format!("{}.{}", path.display(), index));
            if src.exists() {
                let _ = std::fs::rename(&src, &dst);
            }
        }
    }
    Ok(())
}
