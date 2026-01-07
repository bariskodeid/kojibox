# Service Manager & Orchestration

Goals:
- Reliable start/stop/restart with dependency order.
- Visible state transitions for UI.

Non-goals:
- Acting as a system daemon outside the app lifecycle.

State model:
- states: stopped, starting, running, stopping, error
- transitions: stopped->starting->running, running->stopping->stopped, any->error

Service definition (concept):
- id, name, binary, args, env, cwd
- ports: list of {name, port, protocol}
- dependsOn: list of service ids
- healthCheck: {type, target, timeoutMs, intervalMs}
- restartPolicy: {maxRetries, backoffMs}

Dependency order:
- Base order: database -> runtime app -> mailpit
- Customizable per project.

Health checks:
- pid check (process exists)
- port check (listening)
- protocol check (optional per service)
- timeout per check

Process management:
- Spawn child processes with stdout/stderr captured.
- Graceful stop with signal/command, then hard kill after timeout.
- Backoff on repeated failures.

Log routing:
- Ring buffer for UI (last N lines, default 2000).
- Persistent file log with rotation (size-based).

Events to UI:
- service:stateChanged
- service:logLine
- service:health

Data model:
- ServiceDefinition (see `specs/api.md`)
- ServiceState (see `specs/api.md`)
- HealthCheck (see `specs/api.md`)
- LogEntry (see `specs/api.md`)

API contract:
- services.list(): ServiceState[]
- services.start(id): ServiceState
- services.stop(id): ServiceState
- services.restart(id): ServiceState
- services.logs(id, tail): LogEntry[]
- services.health(id): HealthCheck

Sequence flow: Start service with dependency
```txt
UI -> Backend: services.start("postgres")
Backend -> Service Manager: resolve dependencies
Service Manager -> Process Manager: spawn binary
Process Manager -> Service Manager: pid + stdout
Service Manager -> Health Check: probe port
Service Manager -> UI: service:stateChanged(running)
```
