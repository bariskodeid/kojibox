# Inter-Module Data Model & API Contract

This document defines shared data models and API contracts between modules.

## Shared Data Models

RuntimeManifest:
- version: string
- services: ServiceBinary[]
- bundle: BundleMeta

ServiceBinary:
- name: string
- version: string
- os: string
- arch: string
- checksum: string
- size: number
- binPath: string
- defaultPorts: PortDef[]
- env: map<string,string>
- args: string[]

PortDef:
- name: string
- port: number
- protocol: "tcp" | "udp"

BundleMeta:
- createdAt: string
- source: string
- signature: string

ServiceDefinition:
- id: string
- name: string
- binary: string
- args: string[]
- env: map<string,string>
- cwd: string
- ports: PortDef[]
- dependsOn: string[]
- healthCheck: HealthCheck
- restartPolicy: RestartPolicy

HealthCheck:
- type: "pid" | "port" | "http"
- target: string
- timeoutMs: number
- intervalMs: number

RestartPolicy:
- maxRetries: number
- backoffMs: number

ServiceState:
- id: string
- state: "stopped" | "starting" | "running" | "stopping" | "error"
- pid: number | null
- lastError: string | null
- lastUpdated: string

LogEntry:
- ts: string
- level: "debug" | "info" | "warn" | "error"
- service: string
- message: string
- fields: map<string,string>

AppConfig:
- installPath: string
- updateChannel: "stable" | "beta"
- telemetryOptIn: boolean

ServiceConfig:
- id: string
- enabled: boolean
- ports: map<string,number>
- env: map<string,string>
- args: string[]

ProjectConfig:
- id: string
- name: string
- path: string
- domain: string
- stack: string
- overrides: map<string,string>

PortRegistry:
- assigned: map<string,number>
- ranges: map<string,{from:number,to:number}>

SecretRef:
- key: string
- scope: "app" | "project" | "service"

DomainMapping:
- domain: string
- projectId: string
- targetPort: number

ProxyRule:
- host: string
- path: string
- target: string
- tls: boolean

CertMeta:
- name: string
- path: string
- expiresAt: string

DiagnosticsBundle:
- path: string
- createdAt: string
- includes: string[]

MetricsSnapshot:
- ts: string
- uptimeSec: number
- portsInUse: number[]
- cpuPercent: number
- memMB: number

## JSON Payload Examples

RuntimeManifest:
```json
{
  "version": "1",
  "services": [
    {
      "name": "php",
      "version": "8.3.2",
      "os": "windows",
      "arch": "x64",
      "checksum": "sha256:abc123",
      "size": 52428800,
      "binPath": "runtime/bin/php/8.3.2/windows-x64/php.exe",
      "defaultPorts": [{"name": "fpm", "port": 9000, "protocol": "tcp"}],
      "env": {"PHP_INI_SCAN_DIR": "runtime/config/php"},
      "args": ["-v"]
    }
  ],
  "bundle": {
    "createdAt": "2025-01-01T00:00:00Z",
    "source": "official",
    "signature": "sig-xyz"
  }
}
```

ServiceBinary:
```json
{
  "name": "postgres",
  "version": "16.2",
  "os": "linux",
  "arch": "x64",
  "checksum": "sha256:def456",
  "size": 73400320,
  "binPath": "runtime/bin/postgres/16.2/linux-x64/postgres",
  "defaultPorts": [{"name": "db", "port": 5432, "protocol": "tcp"}],
  "env": {"PGDATA": "runtime/data/postgres"},
  "args": ["-D", "runtime/data/postgres"]
}
```

PortDef:
```json
{"name": "http", "port": 8080, "protocol": "tcp"}
```

BundleMeta:
```json
{"createdAt": "2025-01-01T00:00:00Z", "source": "official", "signature": "sig-xyz"}
```

ServiceDefinition:
```json
{
  "id": "postgres",
  "name": "PostgreSQL",
  "binary": "runtime/bin/postgres/16.2/linux-x64/postgres",
  "args": ["-D", "runtime/data/postgres"],
  "env": {"PGDATA": "runtime/data/postgres"},
  "cwd": "runtime",
  "ports": [{"name": "db", "port": 5432, "protocol": "tcp"}],
  "dependsOn": [],
  "healthCheck": {"type": "port", "target": "127.0.0.1:5432", "timeoutMs": 3000, "intervalMs": 2000},
  "restartPolicy": {"maxRetries": 3, "backoffMs": 2000}
}
```

HealthCheck:
```json
{"type": "http", "target": "http://127.0.0.1:8025/health", "timeoutMs": 2000, "intervalMs": 2000}
```

RestartPolicy:
```json
{"maxRetries": 5, "backoffMs": 3000}
```

ServiceState:
```json
{
  "id": "postgres",
  "state": "running",
  "pid": 12345,
  "lastError": null,
  "lastUpdated": "2025-01-01T00:00:10Z"
}
```

LogEntry:
```json
{
  "ts": "2025-01-01T00:00:12Z",
  "level": "info",
  "service": "postgres",
  "message": "database system is ready to accept connections",
  "fields": {"pid": "12345"}
}
```

AppConfig:
```json
{
  "installPath": "/opt/kojibox",
  "updateChannel": "stable",
  "telemetryOptIn": false
}
```

ServiceConfig:
```json
{
  "id": "postgres",
  "enabled": true,
  "ports": {"db": 5432},
  "env": {"PGDATA": "runtime/data/postgres"},
  "args": ["-D", "runtime/data/postgres"]
}
```

ProjectConfig:
```json
{
  "id": "proj-1",
  "name": "my-app",
  "path": "/home/user/projects/my-app",
  "domain": "my-app.test",
  "stack": "php",
  "overrides": {"PHP_VERSION": "8.3.2"}
}
```

PortRegistry:
```json
{
  "assigned": {"postgres": 5432, "mariadb": 3306},
  "ranges": {"postgres": {"from": 5400, "to": 5499}}
}
```

SecretRef:
```json
{"key": "db-password", "scope": "service"}
```

DomainMapping:
```json
{"domain": "my-app.test", "projectId": "proj-1", "targetPort": 3000}
```

ProxyRule:
```json
{"host": "my-app.test", "path": "/", "target": "http://127.0.0.1:3000", "tls": true}
```

CertMeta:
```json
{"name": "my-app.test", "path": "app/certs/my-app.test.pem", "expiresAt": "2026-01-01T00:00:00Z"}
```

DiagnosticsBundle:
```json
{"path": "app/cache/diag-2025-01-01.zip", "createdAt": "2025-01-01T00:00:00Z", "includes": ["logs", "config", "manifest"]}
```

MetricsSnapshot:
```json
{
  "ts": "2025-01-01T00:00:00Z",
  "uptimeSec": 3600,
  "portsInUse": [80, 443, 5432],
  "cpuPercent": 12.5,
  "memMB": 256
}
```

## API Contract (Backend)

Runtime:
- runtime.getManifest(): RuntimeManifest
- runtime.ensureService(name, version): ServiceBinary

Service Manager:
- services.list(): ServiceState[]
- services.start(id): ServiceState
- services.stop(id): ServiceState
- services.restart(id): ServiceState
- services.logs(id, tail): LogEntry[]
- services.health(id): HealthCheck

Configuration:
- config.getApp(): AppConfig
- config.setApp(AppConfig): void
- config.getService(id): ServiceConfig
- config.setService(id, ServiceConfig): void
- config.getProject(id): ProjectConfig
- config.setProject(id, ProjectConfig): void
- config.listProjects(): ProjectConfig[]

Secrets:
- secrets.get(SecretRef): string
- secrets.set(SecretRef, value): void

Tooling:
- domains.list(): DomainMapping[]
- domains.upsert(DomainMapping): void
- domains.remove(domain): void
- proxy.rules(): ProxyRule[]
- proxy.apply(rules): void
- certs.generate(domain): CertMeta
- certs.trust(domain): void

Installer/Updater:
- installer.status(): {phase: string, progress: number}
- installer.start(): void
- updater.check(): {available: boolean, version: string}
- updater.apply(version): void

Observability:
- logs.query(filter): LogEntry[]
- logs.export(filter): string
- diagnostics.create(): DiagnosticsBundle
- metrics.snapshot(): MetricsSnapshot

## Event Contract (UI subscription)

Events:
- service:stateChanged -> ServiceState
- service:logLine -> LogEntry
- service:health -> {id, status}
- config:changed -> {scope, id}
- runtime:updated -> RuntimeManifest
- update:available -> {version}
- update:progress -> {phase, progress}

## API Request/Response Examples

runtime.getManifest:
Request:
```json
{"method": "runtime.getManifest", "params": {}}
```
Response:
```json
{"result": {"version": "1", "services": [], "bundle": {"createdAt": "2025-01-01T00:00:00Z", "source": "official", "signature": "sig"}}}
```

runtime.ensureService:
Request:
```json
{"method": "runtime.ensureService", "params": {"name": "php", "version": "8.3.2"}}
```
Response:
```json
{"result": {"name": "php", "version": "8.3.2", "os": "windows", "arch": "x64", "checksum": "sha256:abc", "size": 52428800, "binPath": "runtime/bin/php/8.3.2/windows-x64/php.exe", "defaultPorts": [{"name": "fpm", "port": 9000, "protocol": "tcp"}], "env": {}, "args": []}}
```

services.list:
Request:
```json
{"method": "services.list", "params": {}}
```
Response:
```json
{"result": [{"id": "postgres", "state": "running", "pid": 12345, "lastError": null, "lastUpdated": "2025-01-01T00:00:10Z"}]}
```

services.start:
Request:
```json
{"method": "services.start", "params": {"id": "postgres"}}
```
Response:
```json
{"result": {"id": "postgres", "state": "starting", "pid": 12345, "lastError": null, "lastUpdated": "2025-01-01T00:00:11Z"}}
```

services.stop:
Request:
```json
{"method": "services.stop", "params": {"id": "postgres"}}
```
Response:
```json
{"result": {"id": "postgres", "state": "stopping", "pid": 12345, "lastError": null, "lastUpdated": "2025-01-01T00:00:12Z"}}
```

services.restart:
Request:
```json
{"method": "services.restart", "params": {"id": "postgres"}}
```
Response:
```json
{"result": {"id": "postgres", "state": "starting", "pid": 12345, "lastError": null, "lastUpdated": "2025-01-01T00:00:13Z"}}
```

services.logs:
Request:
```json
{"method": "services.logs", "params": {"id": "postgres", "tail": 100}}
```
Response:
```json
{"result": [{"ts": "2025-01-01T00:00:12Z", "level": "info", "service": "postgres", "message": "ready", "fields": {}}]}
```

services.health:
Request:
```json
{"method": "services.health", "params": {"id": "postgres"}}
```
Response:
```json
{"result": {"type": "port", "target": "127.0.0.1:5432", "timeoutMs": 3000, "intervalMs": 2000}}
```

config.getApp:
Request:
```json
{"method": "config.getApp", "params": {}}
```
Response:
```json
{"result": {"installPath": "/opt/kojibox", "updateChannel": "stable", "telemetryOptIn": false}}
```

config.setApp:
Request:
```json
{"method": "config.setApp", "params": {"installPath": "/opt/kojibox", "updateChannel": "stable", "telemetryOptIn": false}}
```
Response:
```json
{"result": "ok"}
```

config.getService:
Request:
```json
{"method": "config.getService", "params": {"id": "postgres"}}
```
Response:
```json
{"result": {"id": "postgres", "enabled": true, "ports": {"db": 5432}, "env": {}, "args": []}}
```

config.setService:
Request:
```json
{"method": "config.setService", "params": {"id": "postgres", "enabled": true, "ports": {"db": 5432}, "env": {}, "args": []}}
```
Response:
```json
{"result": "ok"}
```

config.getProject:
Request:
```json
{"method": "config.getProject", "params": {"id": "proj-1"}}
```
Response:
```json
{"result": {"id": "proj-1", "name": "my-app", "path": "/home/user/projects/my-app", "domain": "my-app.test", "stack": "php", "overrides": {}}}
```

config.setProject:
Request:
```json
{"method": "config.setProject", "params": {"id": "proj-1", "name": "my-app", "path": "/home/user/projects/my-app", "domain": "my-app.test", "stack": "php", "overrides": {}}}
```
Response:
```json
{"result": "ok"}
```

config.listProjects:
Request:
```json
{"method": "config.listProjects", "params": {}}
```
Response:
```json
{"result": [{"id": "proj-1", "name": "my-app", "path": "/home/user/projects/my-app", "domain": "my-app.test", "stack": "php", "overrides": {}}]}
```

secrets.get:
Request:
```json
{"method": "secrets.get", "params": {"key": "db-password", "scope": "service"}}
```
Response:
```json
{"result": "secret-value"}
```

secrets.set:
Request:
```json
{"method": "secrets.set", "params": {"key": "db-password", "scope": "service", "value": "secret-value"}}
```
Response:
```json
{"result": "ok"}
```

domains.list:
Request:
```json
{"method": "domains.list", "params": {}}
```
Response:
```json
{"result": [{"domain": "my-app.test", "projectId": "proj-1", "targetPort": 3000}]}
```

domains.upsert:
Request:
```json
{"method": "domains.upsert", "params": {"domain": "my-app.test", "projectId": "proj-1", "targetPort": 3000}}
```
Response:
```json
{"result": "ok"}
```

domains.remove:
Request:
```json
{"method": "domains.remove", "params": {"domain": "my-app.test"}}
```
Response:
```json
{"result": "ok"}
```

proxy.rules:
Request:
```json
{"method": "proxy.rules", "params": {}}
```
Response:
```json
{"result": [{"host": "my-app.test", "path": "/", "target": "http://127.0.0.1:3000", "tls": true}]}
```

proxy.apply:
Request:
```json
{"method": "proxy.apply", "params": {"rules": [{"host": "my-app.test", "path": "/", "target": "http://127.0.0.1:3000", "tls": true}]}}
```
Response:
```json
{"result": "ok"}
```

certs.generate:
Request:
```json
{"method": "certs.generate", "params": {"domain": "my-app.test"}}
```
Response:
```json
{"result": {"name": "my-app.test", "path": "app/certs/my-app.test.pem", "expiresAt": "2026-01-01T00:00:00Z"}}
```

certs.trust:
Request:
```json
{"method": "certs.trust", "params": {"domain": "my-app.test"}}
```
Response:
```json
{"result": "ok"}
```

installer.status:
Request:
```json
{"method": "installer.status", "params": {}}
```
Response:
```json
{"result": {"phase": "idle", "progress": 0}}
```

installer.start:
Request:
```json
{"method": "installer.start", "params": {}}
```
Response:
```json
{"result": "ok"}
```

updater.check:
Request:
```json
{"method": "updater.check", "params": {}}
```
Response:
```json
{"result": {"available": true, "version": "1.1.0"}}
```

updater.apply:
Request:
```json
{"method": "updater.apply", "params": {"version": "1.1.0"}}
```
Response:
```json
{"result": "ok"}
```

logs.query:
Request:
```json
{"method": "logs.query", "params": {"service": "postgres", "level": "info", "limit": 100}}
```
Response:
```json
{"result": [{"ts": "2025-01-01T00:00:12Z", "level": "info", "service": "postgres", "message": "ready", "fields": {}}]}
```

logs.export:
Request:
```json
{"method": "logs.export", "params": {"service": "postgres", "level": "info", "limit": 100}}
```
Response:
```json
{"result": "app/logs/exports/postgres-info-2025-01-01.log"}
```

diagnostics.create:
Request:
```json
{"method": "diagnostics.create", "params": {}}
```
Response:
```json
{"result": {"path": "app/cache/diag-2025-01-01.zip", "createdAt": "2025-01-01T00:00:00Z", "includes": ["logs", "config", "manifest"]}}
```

metrics.snapshot:
Request:
```json
{"method": "metrics.snapshot", "params": {}}
```
Response:
```json
{"result": {"ts": "2025-01-01T00:00:00Z", "uptimeSec": 3600, "portsInUse": [80, 443, 5432], "cpuPercent": 12.5, "memMB": 256}}
```

## Error Response Examples

Port conflict:
```json
{
  "error": {
    "code": "PORT_CONFLICT",
    "message": "Requested port is already in use",
    "detail": "postgres db port 5432 is occupied by process 4321",
    "retryable": true,
    "source": "config",
    "ts": "2025-01-01T00:00:00Z"
  }
}
```

Missing runtime binary:
```json
{
  "error": {
    "code": "RUNTIME_MISSING",
    "message": "Required runtime binary not found",
    "detail": "php 8.3.2 not present in runtime/bin",
    "retryable": true,
    "source": "runtime",
    "ts": "2025-01-01T00:00:00Z"
  }
}
```

Validation error:
```json
{
  "error": {
    "code": "VALIDATION_FAILED",
    "message": "Invalid project path",
    "detail": "Path does not exist or is not readable",
    "retryable": false,
    "source": "config",
    "ts": "2025-01-01T00:00:00Z"
  }
}
```

Permission error (hosts file):
```json
{
  "error": {
    "code": "PERMISSION_DENIED",
    "message": "Insufficient permission to modify hosts file",
    "detail": "User declined permission or requires elevated privileges",
    "retryable": false,
    "source": "tooling",
    "ts": "2025-01-01T00:00:00Z"
  }
}
```

Update signature mismatch:
```json
{
  "error": {
    "code": "UPDATE_SIGNATURE_INVALID",
    "message": "Update package signature mismatch",
    "detail": "Signature verification failed for version 1.1.0",
    "retryable": false,
    "source": "updater",
    "ts": "2025-01-01T00:00:00Z"
  }
}
```
