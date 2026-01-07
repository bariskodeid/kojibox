# Kojibox Project Plan

Kojibox adalah kompetitor Laragon yang portable dan cross-platform (Windows, macOS, Linux) dengan bundling PHP, Node.js, Mailpit, Postgres, MariaDB, dan layanan pendukung. UI menggunakan Vue dan TailwindCSS.

Status: ✅ completed, ⏳ pending

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

## 4) Konfigurasi & Environment (✅)
- Template konfigurasi default.
- Port management dan conflict detection.
- Secrets, per-project override, dan isolasi environment.

## 5) Installer & Bootstrapping (⏳)
- First-run wizard.
- Download/cache binaries (opsional offline pack).
- Update mechanism dan rollback.

## 6) UI/UX Dashboard (Vue + Tailwind) (⏳)
- Status layanan real-time.
- Project manager (path, domain, runtime stack).
- Database tools dan Mailpit integration.

## 7) Integrasi Tooling Dev (⏳)
- Virtual hosts/domains.
- Hosts file helper dan TLS dev certs.
- Reverse proxy dan routing.

## 8) Packaging & Distribusi (Tauri) (⏳)
- Config bundling per OS.
- Signing/notarization.
- Auto-update dan size optimization.

## 9) Observability & Diagnostics (⏳)
- Log terstruktur dan viewer.
- Metrics ringan (uptime, ports).
- Diagnostic bundle untuk support.

## 10) Testing & QA (⏳)
- Matrix OS dan smoke tests.
- Integration tests untuk layanan.
- Upgrade/downgrade validation.

## 11) Dokumentasi & Support (⏳)
- Getting started dan troubleshooting.
- FAQ, known issues, dan support channels.

## 12) Release Management (✅)
- Versioning dan changelog.
- CI/CD pipeline.
- Rollback strategy.
