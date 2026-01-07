# Module Interface Mapping & Error Taxonomy

This document defines who calls whom, and how errors are classified.

## Interface Mapping

UI -> Backend:
- services.list/start/stop/restart
- config.getApp/setApp
- config.listProjects/getProject/setProject
- logs.query/export
- installer.status/start
- updater.check/apply
- diagnostics.create
- metrics.snapshot
- domains.list/upsert/remove
- proxy.rules/apply
- certs.generate/trust

Backend -> Runtime Manager:
- runtime.getManifest
- runtime.ensureService

Backend -> Service Manager:
- services.list/start/stop/restart
- services.logs/health

Backend -> Config Store:
- loadAppConfig/saveAppConfig
- loadServiceConfig/saveServiceConfig
- loadProjectConfig/saveProjectConfig
- loadPortRegistry/savePortRegistry

Backend -> Secrets Store:
- secrets.get/set

Backend -> Installer/Updater:
- installer.start/status
- updater.check/apply

Backend -> Observability:
- logs.query/export
- diagnostics.create
- metrics.snapshot

Backend -> Telemetry:
- telemetry.enqueue(batch)
- telemetry.flush()

Backend -> Tooling/Proxy:
- domains.list/upsert/remove
- proxy.rules/apply
- certs.generate/trust

Service Manager -> Process Manager:
- spawn/stop/kill

Process Manager -> Observability:
- logLine
- processExit

## Error Taxonomy

Error class:
- RuntimeError: runtime binaries missing or invalid.
- ConfigError: invalid or missing config schema.
- ServiceError: process start/stop failures, health check failure.
- PortError: port conflicts or allocation failure.
- NetworkError: download failures or unreachable update feed.
- PermissionError: insufficient permissions (hosts file, cert trust).
- ValidationError: user input invalid.
- UpdateError: update apply failure or signature mismatch.
- StorageError: read/write failures (config/logs/cache).
- SecurityError: secret access denied or redaction failure.

Error fields (normalized):
- code: string (e.g., "PORT_CONFLICT")
- message: string
- detail: string (optional)
- retryable: boolean
- source: module name
- ts: ISO-8601 string

Error handling rules:
- Always return normalized error shape to UI.
- Mask secrets in error detail.
- Include remediation hints for P0 failures (port conflict, missing binary).

Remediation mapping:
- PORT_CONFLICT -> Auto-assign/Change port
- CONFIG_INVALID -> Open config/Reset to defaults
- RUNTIME_MISSING -> Download runtime/Retry
- PROJECT_PATH_INVALID -> Select folder/Retry
- UPDATE_SIGNATURE_INVALID -> Retry/Contact support
- SECRET_ACCESS_DENIED -> Re-enter secret/Unlock keychain
