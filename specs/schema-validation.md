# Schema Validation Guide

This document defines when and how JSON schemas are validated.

## Tooling

- Use Ajv (Node) or jsonschema (Rust) to validate configs.
- Load schemas from `specs/schemas/` at build time and embed in app.

Schema index:
- update-feed.schema.json
- app-config.schema.json
- service-config.schema.json
- project-config.schema.json
- runtime-manifest.schema.json
- service-definition.schema.json
- service-state.schema.json
- log-entry.schema.json
- metrics-snapshot.schema.json
- diagnostics-bundle.schema.json
- domain-mapping.schema.json
- proxy-rule.schema.json
- secret-ref.schema.json
- port-registry.schema.json
- telemetry-event.schema.json
- telemetry-batch.schema.json

## Validation Points

- app/config/app.json on app start.
- app/config/services/*.json on service load.
- app/projects/*/config.json on project load.
- update feed before update apply.

## Failure Handling

- If app config invalid: show blocking error with remediation steps.
- If project config invalid: mark project as invalid and skip start.
- If service config invalid: disable service and notify user.

## Runtime Flow

```txt
App start -> load config -> validate schema -> migrate if needed -> continue
```
