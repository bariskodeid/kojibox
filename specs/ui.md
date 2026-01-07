# UI/UX Dashboard (Vue + Tailwind)

Goals:
- Real-time status visibility.
- Minimal friction for managing projects.

Non-goals:
- Full IDE or code editor.

Views:
- Dashboard: service cards with status + quick actions.
- Projects: list + create/edit.
- Logs: viewer with filter + search.

Routing:
- /dashboard
- /projects
- /logs

Data flow:
- UI polls or subscribes to events from backend.
- UI state normalized in a store (pinia).

Store modules:
- services, projects, logs, settings

Components:
- ServiceCard, ProjectForm, LogViewer, StatusBadge.

Error states:
- Service start failed: show last error and retry action.
- Port conflict: show resolved port or manual override prompt.

Data model (store):
- services: ServiceState[]
- projects: ProjectConfig[]
- logs: LogEntry[]
- settings: AppConfig

API contract (UI usage):
- services.list/start/stop/restart
- config.listProjects/getProject/setProject
- logs.query
- installer.status
- updater.check/apply

Sequence flow: Dashboard load
```txt
UI -> Backend: services.list
UI -> Backend: config.listProjects
Backend -> UI: service states + projects
UI: render status cards and project list
```
