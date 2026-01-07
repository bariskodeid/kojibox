# AGENTS.md - Kojibox

This file defines project-level instructions for coding agents working in this repo.

## Project Summary
- Kojibox is a portable, cross-platform local dev stack (Windows/macOS/Linux).
- Bundles PHP, Node.js, Mailpit, Postgres, MariaDB, and other services.
- UI: Vue + TailwindCSS. App runtime: Tauri-based desktop app.
- Focus: portability, zero global installs, predictable versions, and good UX.

## Key Docs
- Plan: `plan.md`
- Backlog: `issues.md`
- Specs index: `README.md`
- Technical specs: `specs/`
- Schemas: `specs/schemas/`

## Architecture & Contracts
- API/data models: `specs/api.md`
- Interfaces & errors: `specs/interfaces.md`
- Persistence: `specs/persistence.md`
- Deployment: `specs/deployment.md`
- Update feed/signing: `specs/update-feed.md`
- Signature validation: `specs/signature-validation.md`

## Coding Guidelines
- Prefer deterministic behavior and portability.
- Do not modify system-level PATH or install global services.
- Keep config and runtime data under the app/runtime roots.
- Validate configs against JSON schema before use.
- Normalize errors to the shared error shape in `specs/interfaces.md`.
- Mask secrets in logs and UI.

## Filesystem Layout (Concept)
- App root: `app/` (config, projects, logs, certs, cache)
- Runtime root: `runtime/` (bin, data, logs, config, temp, manifest)
- UI source: `src/`
- Tauri backend: `src-tauri/`

## Build & Test (if present)
- Install deps: `pnpm install`
- Dev UI: `pnpm dev`
- Build: `pnpm build`
- Tauri package: `pnpm tauri build`
- Tests: `pnpm test`

## CI/CD
- CI workflow: `.github/workflows/ci.yml`
- Release workflow: `.github/workflows/release.yml`
- Update feed generator: `scripts/generate-update-feed.js`
- Secrets guide: `docs/ci-secrets.md`

## Update Feed & Signing
- Update feed format: `specs/update-feed.md`
- Public keys: `app/config/app.json` -> `updatePublicKeys`
- Feed signature: Ed25519, base64

## Telemetry
- Opt-in only; see `specs/telemetry.md`
- Allowlist: `specs/telemetry-registry.md`
- Upload policy: `specs/telemetry-upload.md`
- Retention: `specs/data-retention.md`

## i18n
- Error copy: `specs/i18n-guidelines.md`
- UI copy: `specs/i18n-ui-guidelines.md`

## When Modifying Specs
- Keep changes consistent across API docs, schemas, and examples.
- Update `README.md` changelog entries when adding new spec docs.

## Safety
- Avoid destructive git commands.
- Preserve user changes in a dirty worktree.
