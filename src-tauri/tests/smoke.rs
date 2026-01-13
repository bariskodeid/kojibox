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
fn smoke_start_stop_dummy_service() {
    let bin = std::env::var("CARGO_BIN_EXE_dummy_service").expect("dummy service bin");
    let port = free_port();
    let def = ServiceDefinition {
        id: "dummy".to_string(),
        name: "Dummy".to_string(),
        binary: bin,
        args: vec![port.to_string()],
        env: HashMap::new(),
        cwd: ".".to_string(),
        ports: vec![PortDef {
            name: "http".to_string(),
            port,
            protocol: "tcp".to_string(),
        }],
        depends_on: Vec::new(),
        health_check: HealthCheck {
            kind: "port".to_string(),
            target: format!("127.0.0.1:{port}"),
            timeout_ms: 2000,
            interval_ms: 500,
        },
        restart_policy: RestartPolicy {
            max_retries: 1,
            backoff_ms: 100,
        },
    };
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let runtime = RuntimeManager::new(".");
    let mut manager = ServiceManager::new(runtime, vec![def], PathBuf::from(temp_dir.path()));

    let state = manager.start("dummy").expect("start dummy");
    assert!(state.state == "running" || state.state == "starting");

    let stopped = manager.stop("dummy").expect("stop dummy");
    assert_eq!(stopped.state, "stopped");
}
