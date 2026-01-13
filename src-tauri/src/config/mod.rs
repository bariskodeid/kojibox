use crate::models::{HealthCheck, PortDef, RestartPolicy, ServiceDefinition};
use crate::runtime;
use std::collections::HashMap;

pub mod php;

fn default_health_check(_service_id: &str, port: u16) -> HealthCheck {
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
    let versions = runtime::default_versions();
    let version = versions.get(service).cloned().unwrap_or_else(|| "0.0.0".to_string());
    runtime::bin_path_for(service, &version)
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
    let mut services = vec![
        service_def("php", "PHP", 9000),
        service_def("node", "Node.js", 3000),
        service_def("postgres", "Postgres", 5432),
        service_def("mariadb", "MariaDB", 3306),
        service_def("mailpit", "Mailpit", 8025),
    ];
    for service in &mut services {
        if service.id == "postgres" {
            service.env.insert("PGDATA".to_string(), "runtime/data/postgres".to_string());
            service.args = vec!["-D".to_string(), "runtime/data/postgres".to_string()];
        }
        if service.id == "mariadb" {
            service.args = vec![
                format!("--defaults-file=runtime/config/mariadb/my.cnf"),
                "--datadir=runtime/data/mariadb".to_string(),
            ];
        }
        if service.id == "php" {
            service.env.insert("PHP_INI_SCAN_DIR".to_string(), "runtime/config/php".to_string());
            // Use built-in server for immediate "It works" experience without Nginx
            service.args = vec![
                "-S".to_string(), "0.0.0.0:8000".to_string(),
                "-t".to_string(), "runtime/www".to_string()
            ];
        }
        if service.id == "php" || service.id == "node" {
            service.depends_on = vec!["postgres".to_string(), "mariadb".to_string()];
        }
    }
    services
}
