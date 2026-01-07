# Telemetry & Opt-in Flow

This document defines telemetry rules and opt-in UX.

Principles:
- Disabled by default.
- Explicit consent required.
- Collect minimal data.

Opt-in flow:
- First-run wizard asks for telemetry consent.
- User can change in settings at any time.

Data collected (if opted in):
- app version
- OS type/version
- uptime duration
- anonymized error codes (no stack traces)

Not collected:
- project source code
- secrets
- personal identifiers

Storage:
- local toggle stored in app/config/app.json (telemetryOptIn)

Retention:
- Follow `specs/data-retention.md`.

Upload:
- Allowlist in `specs/telemetry-registry.md`.
- Upload policy in `specs/telemetry-upload.md`.
