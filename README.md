# Kojibox

Kojibox adalah kompetitor Laragon yang portable dan cross-platform (Windows, macOS, Linux) dengan bundling PHP, Node.js, Mailpit, Postgres, MariaDB, dan layanan pendukung lainnya. UI dibangun dengan Vue dan TailwindCSS.

Rencana proyek lengkap tersimpan di `plan.md`, dan spesifikasi teknis per modul ada di folder `specs/`.

## Spesifikasi Teknis
- `specs/architecture.md`
- `specs/api.md`
- `specs/interfaces.md`
- `specs/persistence.md`
- `specs/deployment.md`
- `specs/update-feed.md`
- `specs/runtime.md`
- `specs/service-manager.md`
- `specs/configuration.md`
- `specs/installer.md`
- `specs/ui.md`
- `specs/tooling.md`
- `specs/packaging.md`
- `specs/observability.md`
- `specs/testing.md`
- `specs/documentation.md`

## Breakdown Teknis per Modul

### 1) Runtime & Bundle
- Struktur direktori portable untuk semua binaries.
- Version pinning per layanan dan strategi upgrade.
- Isolasi runtime agar tidak mengubah sistem global.

### 2) Service Manager & Orkestrasi
- Start/stop/restart dan dependency order antar layanan.
- Health check, auto-recovery, dan status stream ke UI.
- Log routing dan penyimpanan log terstruktur.

### 3) Konfigurasi & Environment
- Template konfigurasi default untuk tiap layanan.
- Manajemen port + conflict detection.
- Per-project overrides dan secrets handling.

### 4) Installer & Bootstrapping
- First-run wizard.
- Download/cache binaries, dukungan offline pack.
- Update mechanism dan rollback aman.

### 5) UI/UX Dashboard (Vue + Tailwind)
- Status layanan real-time.
- Project manager (path, domain, stack).
- Database tools dan Mailpit integration.

### 6) Tooling Dev & Proxy
- Virtual hosts/domains.
- Hosts file helper dan TLS dev certs.
- Reverse proxy dan routing fleksibel.

### 7) Packaging & Distribusi (Tauri)
- Bundling per OS, signing/notarization.
- Auto-update dan optimasi ukuran.

### 8) Observability & Diagnostics
- Log viewer dan export diagnostic bundle.
- Metrics ringan (uptime, ports, resource).

### 9) Testing & QA
- Matrix OS dan smoke tests.
- Integration tests antar layanan.
- Upgrade/downgrade validation.

### 10) Dokumentasi & Support
- Getting started, troubleshooting, dan FAQ.
- Guide kontribusi dan release notes.

## Backlog Tasks

### Runtime & Bundle
- Definisikan struktur direktori portable dan naming convention.
- Tentukan versi pinned untuk PHP/Node/Postgres/MariaDB/Mailpit.
- Buat manifest runtime untuk dependency mapping per OS/arch.
- Implementasikan isolasi PATH agar tidak mengubah sistem global.

### Service Manager & Orkestrasi
- Rancang state machine service (start/stop/restart/error).
- Implement dependency order (db sebelum app, mailpit opsional).
- Health check per layanan (pid/port/process).
- Log streaming ke UI (ring buffer + persistent log).

### Konfigurasi & Environment
- Template default config per layanan.
- Port auto-assign dan conflict detection.
- Per-project override config + env merging.
- Secrets handling (local store + masking di UI/log).

### Installer & Bootstrapping
- First-run wizard (path, ports, defaults).
- Download/caching binaries + checksum verification.
- Offline pack format dan extractor.
- Update mechanism + rollback artifacts.

### UI/UX Dashboard
- Service status cards real-time.
- Project manager (CRUD project, path, domain).
- Quick actions (start/stop/restart, open logs).
- Integrasi Mailpit viewer.

### Tooling Dev & Proxy
- Virtual host manager (domain -> project mapping).
- Hosts file helper per OS (opt-in).
- Dev TLS cert generation + trust flow.
- Reverse proxy router + rules editor.

### Packaging & Distribusi
- Tauri bundling per OS + size optimization.
- Signing/notarization pipeline.
- Auto-update config dan feed.

### Observability & Diagnostics
- Log viewer + filter + export.
- Diagnostic bundle generator.
- Metrics ringan (uptime, ports, resource usage).

### Testing & QA
- Smoke tests per OS (start/stop all).
- Integration tests antar layanan.
- Upgrade/downgrade validation.

### Dokumentasi & Support
- Getting started + troubleshooting.
- FAQ + known issues.
- Guide kontribusi.

## Changelog

### Unreleased
- Tambah rencana proyek di `plan.md`.
- Tambah breakdown teknis dan backlog tasks di `README.md`.
- Tambah backlog issue tracker di `issues.md` dengan prioritas dan milestone.
- Pecah spesifikasi teknis per modul ke folder `specs/`.
- Tambah index spesifikasi di `README.md` dan detail teknis tambahan di `specs/`.
- Tambah kontrak API, data model, dan diagram arsitektur di `specs/`.
- Tambah mapping interface antar modul, error taxonomy, dan desain data persistence di `specs/`.
- Tambah desain deployment/release pipeline di `specs/deployment.md`.
- Tambah CI/CD workflow, update feed config, dan signing key management.
- Tambah signing feed, generator feed, dan dokumen secrets CI di `docs/ci-secrets.md`.
- Tambah panduan key Ed25519 dan validasi signature di `specs/signature-validation.md`.
- Tambah contoh implementasi verifikasi signature dan schema JSON di `specs/schemas/`.
- Tambah schema untuk service definition/state, log entry, dan metrics snapshot.
- Tambah schema diagnostics, domain/proxy, dan panduan validasi schema di `specs/schema-validation.md`.
- Tambah schema secret/port registry dan mapping error validasi di `specs/validation-errors.md`.
- Tambah index schema validation dan panduan UI error remediation di `specs/ui-error-remediation.md`.
- Tambah remediation mapping di `specs/interfaces.md` dan checklist error UI di `specs/ui-error-checklist.md`.
- Tambah panduan i18n dan flowchart penanganan error di `specs/i18n-guidelines.md` dan `specs/error-flowcharts.md`.
- Tambah panduan i18n UI umum dan dokumen telemetry opt-in di `specs/i18n-ui-guidelines.md` dan `specs/telemetry.md`.
- Tambah kebijakan data retention untuk telemetry/log di `specs/data-retention.md`.
- Tambah schema dan contoh payload telemetry di `specs/telemetry-events.md`.
- Tambah allowlist event telemetry, kebijakan upload, dan schema batch telemetry.
- Tambah detail upload response, mapping telemetry interface, dan note allowlist event.
- Sinkronkan status plan ke `plan.md`.
