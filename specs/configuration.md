# Configuration & Environment

Goals:
- Predictable defaults and easy overrides.
- No secrets leakage to logs/UI.

Non-goals:
- Managing user dotfiles or global shell profiles.

Templates:
- Templates stored in runtime/config/{service}/default.conf
- Regen on version change with migration steps.

Config locations:
- app/config/app.json (global app settings)
- app/config/services/{service}.json (service config)
- app/projects/{projectId}/config.json (project overrides)

Ports:
- Port registry file (config/ports.json).
- Auto-assign on conflict with reserved ranges per service.

Port allocation:
- Use fixed defaults first.
- If conflict, probe next available in reserved range.
- Persist assigned port to ports.json.

Overrides:
- Project-level overrides in project/config.json.
- Merging strategy: base -> service -> project -> user.

Validation:
- Validate config schema on load.
- Reject invalid ports, paths, and env keys.

Secrets:
- Local encrypted store (OS keychain when available).
- Mask in logs and UI (replace with "***").

Secret access:
- Only resolved at runtime.
- Never written to plain-text config files.

Data model:
- AppConfig (see `specs/api.md`)
- ServiceConfig (see `specs/api.md`)
- ProjectConfig (see `specs/api.md`)
- PortRegistry (see `specs/api.md`)
- SecretRef (see `specs/api.md`)

API contract:
- config.getApp(): AppConfig
- config.setApp(AppConfig): void
- config.getService(id): ServiceConfig
- config.setService(id, ServiceConfig): void
- config.getProject(id): ProjectConfig
- config.setProject(id, ProjectConfig): void
- config.listProjects(): ProjectConfig[]
- secrets.get(SecretRef): string
- secrets.set(SecretRef, value): void

Sequence flow: Load and merge configuration
```txt
Backend -> Config Store: load app/service/project configs
Config Store: validate schema
Config Store -> Secrets: resolve SecretRef values
Config Store: merge base -> service -> project -> user
Backend -> Service Manager: provide resolved env/ports
```
