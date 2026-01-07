# Data Retention Policy

This document defines retention rules for telemetry and logs.

## Logs

Runtime logs:
- Rotate by size (default 10MB).
- Keep max 5 files per service.
- Optional compression for rotated logs.

App logs:
- Rotate by size (default 5MB).
- Keep max 3 files for UI and backend logs.

## Telemetry (Opt-in)

Local buffer:
- Store last 7 days of anonymized events.
- Purge on opt-out.

Upload policy:
- Batch upload every 24 hours.
- Drop events older than 30 days.

## Diagnostics Bundles

Retention:
- Keep last 3 bundles locally.
- Auto-delete older bundles.

## Cache

Runtime cache:
- LRU cleanup when exceeding size threshold (default 2GB).
- Preserve latest version per service.
