# Kojibox Project Plan

Kojibox adalah kompetitor Laragon yang portable dan cross-platform (Windows, macOS, Linux) dengan bundling PHP, Node.js, Mailpit, Postgres, MariaDB, dan layanan pendukung. UI menggunakan Vue dan TailwindCSS.

Status: ✅ completed, 🚧 in-progress, ⏳ pending

## 1) Tujuan Produk dan Scope (✅)
- Definisikan fitur MVP vs Pro.
- Tentukan target OS, arsitektur CPU, dan kanal distribusi.
- Putuskan lisensi dan model update.

## 2) Arsitektur Runtime Portable (✅)
- Standarisasi struktur direktori bundle.
- Skema version pinning dan upgrade untuk setiap layanan.
- Strategi sandboxing agar portable tanpa instalasi global.

## 3) Service Manager dan Orkestrasi (✅)
- Start/stop/restart layanan per OS.
- Health check, auto-recovery, dan dependency order.
- Routing log dan status ke UI.
- Graceful error handling untuk missing binaries (✅ Fix Runtime flow).

## 4) Konfigurasi & Environment (✅)
- Template konfigurasi default.
- Port management dan conflict detection (✅ Privileged port warning).
- Secrets, per-project override, dan isolasi environment.
- Validasi input frontend (✅ Path, URL, Duplicate ID checks).

## 5) Installer & Bootstrapping (✅)
- First-run wizard (✅ Path validation).
- Download/cache binaries (opsional offline pack).
- Update mechanism dan rollback.

## 6) UI/UX Dashboard (Vue + Tailwind) (✅)
- Status layanan real-time.
- Project manager (path, domain, runtime stack).
- Database tools dan Mailpit integration.

## 7) Integrasi Tooling Dev (✅)
- Virtual hosts/domains (✅ Implemented).
- Hosts file helper dan TLS dev certs (✅ Cert generation and hosts manipulation).
- Reverse proxy dan routing (✅ Proxy rules management).
- Automated TLS Trust (✅ Automated trust command for Windows, macOS, and Linux).

## 8) Packaging & Distribusi (Tauri) (✅)
- Config bundling per OS (✅ Configured resources in tauri.conf.json).
- Signing/notarization (✅ CI/CD workflow ready).
- Auto-update dan size optimization (✅ Update feed generator and size auditor).
- Bundling Binaries (✅ prepare-runtime.js implemented).

## 9) Observability & Diagnostics (✅)
- Log terstruktur dan viewer (✅ Implemented).
- Metrics ringan (uptime, ports) (✅ Implemented).
- Diagnostic bundle untuk support (✅ Implemented).
- Telemetry (✅ Stub implementation with opt-in logic).

## 10) Testing & QA (✅)
- Matrix OS dan smoke tests (✅ Smoke tests implemented).
- Integration tests untuk layanan (✅ Integration tests implemented).
- Upgrade/downgrade validation (✅ Updater tests implemented).

## 11) Dokumentasi & Support (✅)
- Getting started dan troubleshooting (✅ README updated).
- FAQ, known issues, dan support channels (✅ Docs available).

## 12) Release Management (✅)
- Versioning dan changelog (✅ Implemented).
- CI/CD pipeline (✅ Implemented).
- Rollback strategy (✅ Implemented).

## 14) Polishing & Finishing (✅)
- Raw Configuration Editor (✅ Implemented).
- Log Management (Clear Logs) (✅ Implemented).
- About Page & Third Party Licenses (✅ Implemented).
- Dark Mode / Theming (✅ Implemented).
- External Binary Source Configuration (✅ Implemented).
- Industrial Minimalism Design (✅ Tailwind v4 integrated with Brutalist Industrial theme).
