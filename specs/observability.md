# Observability & Diagnostics

Goals:
- Actionable logs and easy diagnostics export.

Non-goals:
- Always-on remote telemetry by default.

Log viewer:
- filter by service, level, time.
- export to file.

Log format:
- timestamp, level, service, message, fields

Diagnostics:
- bundle logs + config + system info.
- redact secrets.

Bundle content:
- app version, OS info, runtime manifest, recent logs, config snapshots

Metrics:
- uptime, ports in use, resource usage.

Retention:
- logs rotated by size and max files per service
- See `specs/data-retention.md` for retention rules.

Data model:
- LogEntry (see `specs/api.md`)
- DiagnosticsBundle (see `specs/api.md`)
- MetricsSnapshot (see `specs/api.md`)

API contract:
- logs.query(filter): LogEntry[]
- logs.export(filter): string
- diagnostics.create(): DiagnosticsBundle
- metrics.snapshot(): MetricsSnapshot

Sequence flow: Export diagnostics
```txt
UI -> Backend: diagnostics.create()
Backend -> Log Store: collect recent logs
Backend -> Config Store: snapshot config
Backend: redact secrets and package bundle
Backend -> UI: DiagnosticsBundle path
```
