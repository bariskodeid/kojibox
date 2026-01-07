# Data Persistence Design

This document defines data storage locations, file formats, and migration rules.

## Storage Roots

App root:
- app/
  - config/
  - projects/
  - logs/
  - certs/
  - cache/

Runtime root:
- runtime/
  - bin/
  - data/
  - logs/
  - config/
  - temp/

## File Formats

Primary config format:
- JSON for app/project/service config.
- One file per service in app/config/services/.

Recommended files:
- app/config/app.json
- app/config/ports.json
- app/config/services/{service}.json
- app/projects/{projectId}/config.json
- app/projects/{projectId}/env.json
- app/projects/{projectId}/domains.json

Runtime files:
- runtime/manifest.json
- runtime/cache/index.json
- runtime/data/{service}/
- runtime/logs/{service}/

Logs:
- app/logs/ui.log
- app/logs/backend.log
- runtime/logs/{service}/service.log

## Schema Versioning

Config schema:
- app/config/app.json includes schemaVersion
- app/config/services/{service}.json includes schemaVersion
- app/projects/{projectId}/config.json includes schemaVersion

Migrations:
- app/config/migrations/ tracks applied versions
- Each migration is idempotent
- Downgrade path stored for last 2 versions

## Atomic Writes

Write strategy:
- write to temp file
- fsync
- rename to target

Backup:
- keep last N backups per config file
- restore on parse failure

## Data Retention

Logs:
- rotate by size (default 10MB) and max files (default 5)
- compress rotated logs (optional)

Cache:
- prune by total size and LRU policy

## Security

Secrets:
- stored in OS keychain if available
- never written to plain JSON

Access control:
- app data is user-owned
- avoid system-level writes unless opt-in
