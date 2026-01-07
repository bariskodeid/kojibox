# Telemetry Events

This document defines telemetry event schema and payload examples.

## Schema
- See `specs/schemas/telemetry-event.schema.json`

## Events

Note:
- Events must be listed in `specs/telemetry-registry.md` to be uploaded.

app_start:
```json
{
  "event": "app_start",
  "ts": "2025-01-01T00:00:00Z",
  "appVersion": "1.0.0",
  "os": "windows",
  "payload": {
    "channel": "stable",
    "servicesEnabled": ["php", "postgres"]
  }
}
```

service_start:
```json
{
  "event": "service_start",
  "ts": "2025-01-01T00:01:00Z",
  "appVersion": "1.0.0",
  "os": "windows",
  "payload": {
    "service": "postgres",
    "durationMs": 1200,
    "success": true
  }
}
```

service_error:
```json
{
  "event": "service_error",
  "ts": "2025-01-01T00:02:00Z",
  "appVersion": "1.0.0",
  "os": "windows",
  "payload": {
    "service": "mariadb",
    "errorCode": "PORT_CONFLICT",
    "retryable": true
  }
}
```

update_check:
```json
{
  "event": "update_check",
  "ts": "2025-01-01T01:00:00Z",
  "appVersion": "1.0.0",
  "os": "windows",
  "payload": {
    "channel": "stable",
    "available": true,
    "latest": "1.1.0"
  }
}
```

diagnostics_export:
```json
{
  "event": "diagnostics_export",
  "ts": "2025-01-01T02:00:00Z",
  "appVersion": "1.0.0",
  "os": "windows",
  "payload": {
    "bundleSizeMB": 12.4,
    "includes": ["logs", "config", "manifest"]
  }
}
```
