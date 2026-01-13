use kojibox_lib::models::{HealthCheck, PortDef, RestartPolicy, ServiceDefinition};
use kojibox_lib::runtime::RuntimeManager;
use kojibox_lib::service_manager::ServiceManager;
use std::collections::HashMap;
use std::net::TcpListener;
use std::path::PathBuf;

fn free_port() -> u16 {
    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind failed");
    listener.local_addr().unwrap().port()
}

#[test]
fn integration_dependency_start() {
    let bin = std::env::var("CARGO_BIN_EXE_dummy_service").expect("dummy service bin");
    let db_port = free_port();
    let app_port = free_port();
    assert_ne!(db_port, app_port);

    let db = ServiceDefinition {
        id: "postgres".to_string(),
        name: "Postgres".to_string(),
        binary: bin.clone(),
        args: vec![db_port.to_string()],
        env: HashMap::new(),
        cwd: ".".to_string(),
        ports: vec![PortDef {
            name: "db".to_string(),
            port: db_port,
            protocol: "tcp".to_string(),
        }],
        depends_on: Vec::new(),
        health_check: HealthCheck {
            kind: "port".to_string(),
            target: format!("127.0.0.1:{db_port}"),
            timeout_ms: 2000,
            interval_ms: 200,
        },
        restart_policy: RestartPolicy {
            max_retries: 1,
            backoff_ms: 100,
        },
    };

    let app = ServiceDefinition {
        id: "app".to_string(),
        name: "App".to_string(),
        binary: bin,
        args: vec![app_port.to_string()],
        env: HashMap::new(),
        cwd: ".".to_string(),
        ports: vec![PortDef {
            name: "http".to_string(),
            port: app_port,
            protocol: "tcp".to_string(),
        }],
        depends_on: vec!["postgres".to_string()],
        health_check: HealthCheck {
            kind: "port".to_string(),
            target: format!("127.0.0.1:{app_port}"),
            timeout_ms: 2000,
            interval_ms: 200,
        },
        restart_policy: RestartPolicy {
            max_retries: 1,
            backoff_ms: 100,
        },
    };

    let temp_dir = tempfile::tempdir().expect("tempdir");
    let runtime = RuntimeManager::new(".");
    let mut manager = ServiceManager::new(runtime, vec![db, app], PathBuf::from(temp_dir.path()));

    let state = manager.start("app").expect("start app");
    assert!(state.state == "running" || state.state == "starting");
    assert!(manager.health("postgres").is_ok());
    assert!(manager.health("app").is_ok());

    let _ = manager.stop("app");
    let _ = manager.stop("postgres");
}
