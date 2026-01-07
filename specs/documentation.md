# Documentation & Support

Goals:
- Clear onboarding and troubleshooting.

Non-goals:
- Marketing site content.

Docs:
- Getting started.
- FAQ + known issues.
- Contribution guide.

Structure:
- docs/getting-started.md
- docs/troubleshooting.md
- docs/faq.md
- docs/contributing.md

Support:
- Diagnostic bundle flow and support channels.

Support flow:
- user exports diagnostics -> support ticket -> reproduce -> fix

Data model:
- DocSet: {name, path, version}
- SupportRequest: {id, user, createdAt, bundlePath}

API contract:
- docs.list(): DocSet[]
- docs.open(name): string
- support.create(bundlePath): SupportRequest

Sequence flow: Support request
```txt
User -> UI: export diagnostics
UI -> Backend: diagnostics.create
UI -> Backend: support.create(bundlePath)
Backend -> UI: SupportRequest id
```
