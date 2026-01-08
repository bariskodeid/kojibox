use crate::models::{HealthCheck, PortDef, RestartPolicy, ServiceDefinition};
use std::collections::HashMap;

fn default_health_check(service_id: &str, port: u16) -> HealthCheck {
    HealthCheck {
        kind: "port".to_string(),
        target: format!("127.0.0.1:{}", port),
        timeout_ms: 3000,
        interval_ms: 2000,
    }
}

fn default_restart_policy() -> RestartPolicy {
    RestartPolicy {
        max_retries: 3,
        backoff_ms: 2000,
    }
}

fn bin_path(service: &str) -> String {
    let ext = if cfg!(target_os = "windows") { ".exe" } else { "" };
    format!("runtime/bin/{service}/current/{service}{ext}")
}

fn service_def(id: &str, name: &str, port: u16) -> ServiceDefinition {
    ServiceDefinition {
        id: id.to_string(),
        name: name.to_string(),
        binary: bin_path(id),
        args: Vec::new(),
        env: HashMap::new(),
        cwd: "runtime".to_string(),
        ports: vec![PortDef {
            name: "main".to_string(),
            port,
            protocol: "tcp".to_string(),
        }],
        depends_on: Vec::new(),
        health_check: default_health_check(id, port),
        restart_policy: default_restart_policy(),
    }
}

pub fn default_services() -> Vec<ServiceDefinition> {
    vec![
        service_def("php", "PHP", 9000),
        service_def("node", "Node.js", 3000),
        service_def("postgres", "Postgres", 5432),
        service_def("mariadb", "MariaDB", 3306),
        service_def("mailpit", "Mailpit", 8025),
    ]
}
