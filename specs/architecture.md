# Architecture Overview

High-level component diagram:

```txt
                +----------------------+
                |       UI (Vue)       |
                |  Dashboard/Projects  |
                +----------+-----------+
                           |
                           | IPC/HTTP
                           v
                +----------+-----------+
                |    Backend Core     |
                |  (Tauri/Rust/Node)  |
                +---+----+---+----+---+
                    |    |   |    |
                    |    |   |    +------------------+
                    |    |   |                       |
                    v    v   v                       v
            Runtime  Config  Services          Tooling/Proxy
             Manager  Store  Orchestrator      (Domains/TLS)
                    |           |
                    |           v
                    |      Process Manager
                    |           |
                    v           v
                Binaries      Logs/Metrics

                +----------------------+
                |    Installer/Updater |
                +----------------------+
```

Data flow summary:
- UI reads state via API and receives event streams.
- Service Manager controls processes using Runtime binaries.
- Configuration and Secrets resolve env and ports.
- Observability collects logs/metrics and serves diagnostics.

Sequence conventions:
- UI -> Backend: request/command.
- Backend -> UI: events.
