# Testing & QA

Goals:
- Cross-platform stability.

Non-goals:
- Full performance benchmarking in MVP.

Testing matrix:
- Windows, macOS, Linux.

CI stages:
- build
- smoke
- integration

Smoke tests:
- start/stop all services.
- verify ports and basic health.

Smoke steps:
- start services
- check pid and ports
- stop services

Integration tests:
- DB connectivity.
- Mailpit delivery.

Test harness:
- scripts/qa/ for OS-specific runners

Upgrade tests:
- upgrade/downgrade with data migration.

Artifacts:
- store logs for failed runs

Data model:
- TestCase: {id, name, steps, expected}
- TestRun: {id, startedAt, status, results}

API contract:
- qa.runSmoke(): TestRun
- qa.runIntegration(): TestRun
- qa.report(runId): TestRun

Sequence flow: CI smoke tests
```txt
CI -> QA Runner: qa.runSmoke()
QA Runner -> Service Manager: start services
QA Runner: validate health checks
QA Runner -> Service Manager: stop services
QA Runner -> CI: results + logs
```
