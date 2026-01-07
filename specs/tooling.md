# Tooling Dev & Proxy

Goals:
- Easy local domain mapping and TLS.

Non-goals:
- Modifying system DNS outside opt-in hosts flow.

Virtual hosts:
- domain -> project mapping stored in config/domains.json
- applied to reverse proxy config

Domain mapping schema:
- { "domain": "app.test", "projectId": "...", "targetPort": 3000 }

Hosts helper:
- opt-in flow with explicit consent
- backup and rollback on errors

Hosts storage:
- Keep backup in app/config/hosts.backup

TLS dev certs:
- local CA per user
- trust flow per OS

Certificates:
- Store in app/certs/
- Rotate when expired or revoked

Reverse proxy:
- rules definition file with validation
- hot reload on config change

Proxy rules:
- Match by host/path -> target
- Support http and https listeners

Data model:
- DomainMapping (see `specs/api.md`)
- ProxyRule (see `specs/api.md`)
- CertMeta (see `specs/api.md`)

API contract:
- domains.list/upsert/remove
- proxy.rules/apply
- certs.generate/trust

Sequence flow: Add local domain
```txt
UI -> Backend: domains.upsert(domain, projectId, port)
Backend -> Proxy: update rules and reload
Backend -> Hosts Helper: prompt opt-in, apply if approved
Backend -> UI: config:changed(domain)
```
