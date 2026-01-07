# Kojibox Backlog (GitHub Issues Format)

Format:
- Title
- Body
- Labels
- Milestone

Legend:
- Priority: P0 (must-have), P1 (should-have), P2 (nice-to-have)
- Milestone: MVP, V1, V2

## Epic: Runtime & Bundle
Title: [Runtime] Definisikan struktur direktori portable
Body: Tentukan layout folder runtime untuk semua layanan (bin, data, logs, config) dan naming convention per OS. Priority: P0
Labels: epic, runtime
Milestone: MVP

Title: [Runtime][Task] Draft layout folder runtime
Body: Buat proposal layout folder (bin, data, logs, config, temp) dan contoh struktur. Priority: P0
Labels: task, runtime
Milestone: MVP

Title: [Runtime][Task] Tentukan naming convention per OS
Body: Skema penamaan binary, folder data, dan log per OS/arch. Priority: P0
Labels: task, runtime
Milestone: MVP

Title: [Runtime] Version pinning dan manifest runtime
Body: Buat manifest versi untuk PHP/Node/Postgres/MariaDB/Mailpit per OS/arch dan strategi upgrade. Priority: P0
Labels: epic, runtime
Milestone: MVP

Title: [Runtime][Task] Daftar versi default per layanan
Body: Tentukan versi default untuk setiap layanan dan alasan pemilihan. Priority: P0
Labels: task, runtime
Milestone: MVP

Title: [Runtime][Task] Format manifest runtime
Body: Tentukan format manifest (JSON/YAML) dan fields yang dibutuhkan. Priority: P0
Labels: task, runtime
Milestone: MVP

Title: [Runtime] Isolasi PATH dan environment
Body: Implementasi isolasi PATH agar tidak mengubah sistem global dan memastikan binary lokal yang dipakai. Priority: P0
Labels: epic, runtime
Milestone: MVP

Title: [Runtime][Task] Wrapper runner untuk PATH lokal
Body: Buat wrapper yang menyuntik PATH lokal sebelum menjalankan layanan. Priority: P0
Labels: task, runtime
Milestone: MVP

Title: [Runtime][Task] Validasi isolasi environment
Body: Tambahkan cek agar tidak terjadi override PATH global. Priority: P0
Labels: task, runtime
Milestone: MVP

## Epic: Service Manager & Orkestrasi
Title: [Service] State machine layanan
Body: Rancang dan implementasikan state machine (start/stop/restart/error) untuk semua layanan. Priority: P0
Labels: epic, service
Milestone: MVP

Title: [Service][Task] Definisikan state dan transitions
Body: Draft state machine diagram dan transisi untuk start/stop/restart/error. Priority: P0
Labels: task, service
Milestone: MVP

Title: [Service][Task] Implementasi handler state
Body: Implementasi handler state dan error handling konsisten. Priority: P0
Labels: task, service
Milestone: MVP

Title: [Service] Dependency order dan health checks
Body: Implement dependency order (DB sebelum app) dan health check per layanan (pid/port/process). Priority: P0
Labels: epic, service
Milestone: MVP

Title: [Service][Task] Dependency graph layanan
Body: Definisikan dependency graph layanan dan urutan start/stop. Priority: P0
Labels: task, service
Milestone: MVP

Title: [Service][Task] Health check per layanan
Body: Implementasi health check per layanan (pid/port/process) dengan timeout. Priority: P0
Labels: task, service
Milestone: MVP

Title: [Service] Log routing dan streaming ke UI
Body: Sediakan log router (ring buffer + persistent) dan streaming status ke UI. Priority: P1
Labels: epic, service
Milestone: V1

Title: [Service][Task] Ring buffer log
Body: Implement ring buffer log untuk UI realtime. Priority: P1
Labels: task, service
Milestone: V1

Title: [Service][Task] Persistent log storage
Body: Simpan log ke disk dengan rotasi ukuran. Priority: P1
Labels: task, service
Milestone: V1

## Epic: Konfigurasi & Environment
Title: [Config] Template konfigurasi default
Body: Sediakan template konfigurasi default per layanan dan mekanisme regen. Priority: P0
Labels: epic, config
Milestone: MVP

Title: [Config][Task] Draft template per layanan
Body: Buat template awal untuk PHP, Node, Postgres, MariaDB, Mailpit. Priority: P0
Labels: task, config
Milestone: MVP

Title: [Config][Task] Mekanisme regen config
Body: Implement mekanisme regen config saat update versi. Priority: P1
Labels: task, config
Milestone: V1

Title: [Config] Port auto-assign dan conflict detection
Body: Implement port scanning, auto-assign, dan conflict detection yang aman. Priority: P0
Labels: epic, config
Milestone: MVP

Title: [Config][Task] Port scanner
Body: Implement port scanner yang aman dan cepat. Priority: P0
Labels: task, config
Milestone: MVP

Title: [Config][Task] Port allocator
Body: Implement auto-assign port dan simpan ke config. Priority: P0
Labels: task, config
Milestone: MVP

Title: [Config] Per-project override + secrets handling
Body: Implement overrides per project dan penyimpanan secrets lokal dengan masking di UI/log. Priority: P1
Labels: epic, config
Milestone: V1

Title: [Config][Task] Skema override config
Body: Definisikan skema override config dan merging rules. Priority: P1
Labels: task, config
Milestone: V1

Title: [Config][Task] Secret storage lokal
Body: Simpan secrets secara lokal dan masking di UI/log. Priority: P1
Labels: task, config
Milestone: V1

## Epic: Installer & Bootstrapping
Title: [Installer] First-run wizard
Body: Buat wizard first-run (path install, ports, defaults). Priority: P0
Labels: epic, installer
Milestone: MVP

Title: [Installer][Task] UI flow wizard
Body: Definisikan langkah-langkah wizard dan layout UI. Priority: P0
Labels: task, installer
Milestone: MVP

Title: [Installer][Task] Persist pilihan wizard
Body: Simpan pilihan wizard ke config awal. Priority: P0
Labels: task, installer
Milestone: MVP

Title: [Installer] Download/cache binaries + checksum
Body: Download/caching binaries per OS, validasi checksum, dan mekanisme retry. Priority: P0
Labels: epic, installer
Milestone: MVP

Title: [Installer][Task] Downloader dengan resume
Body: Implement downloader dengan dukungan resume dan retry. Priority: P0
Labels: task, installer
Milestone: MVP

Title: [Installer][Task] Verifikasi checksum
Body: Validasi checksum dan rollback bila gagal. Priority: P0
Labels: task, installer
Milestone: MVP

Title: [Installer] Offline pack dan extractor
Body: Definisikan format offline pack dan implementasi extractor. Priority: P1
Labels: epic, installer
Milestone: V1

Title: [Installer][Task] Spesifikasi offline pack
Body: Tentukan format offline pack dan metadata. Priority: P1
Labels: task, installer
Milestone: V1

Title: [Installer][Task] Extractor offline pack
Body: Implement extractor dan integrasi ke installer. Priority: P1
Labels: task, installer
Milestone: V1

Title: [Installer] Update mechanism + rollback
Body: Implement update mechanism dan rollback artifacts. Priority: P1
Labels: epic, installer
Milestone: V1

Title: [Installer][Task] Update pipeline
Body: Implement pipeline update dan staging area. Priority: P1
Labels: task, installer
Milestone: V1

Title: [Installer][Task] Rollback artifacts
Body: Simpan artifact sebelumnya dan restore bila gagal. Priority: P1
Labels: task, installer
Milestone: V1

## Epic: UI/UX Dashboard
Title: [UI] Service status cards real-time
Body: Komponen status layanan real-time (running, error, restarting). Priority: P0
Labels: epic, ui
Milestone: MVP

Title: [UI][Task] Component status card
Body: Implement komponen status card dan state variants. Priority: P0
Labels: task, ui
Milestone: MVP

Title: [UI][Task] Realtime status polling
Body: Integrasi realtime status (polling atau event stream). Priority: P0
Labels: task, ui
Milestone: MVP

Title: [UI] Project manager (CRUD)
Body: CRUD project (path, domain, stack) dan integrasi dengan runtime. Priority: P0
Labels: epic, ui
Milestone: MVP

Title: [UI][Task] Form project create/edit
Body: Buat form create/edit project dengan validasi. Priority: P0
Labels: task, ui
Milestone: MVP

Title: [UI][Task] Integrasi runtime project
Body: Hook UI project ke runtime manager. Priority: P0
Labels: task, ui
Milestone: MVP

Title: [UI] Quick actions + logs
Body: Tombol start/stop/restart dan shortcut ke log viewer. Priority: P1
Labels: epic, ui
Milestone: V1

Title: [UI][Task] Action buttons
Body: Implement tombol start/stop/restart untuk tiap layanan. Priority: P1
Labels: task, ui
Milestone: V1

Title: [UI][Task] Shortcut ke log viewer
Body: Tambahkan shortcut dari card ke log viewer. Priority: P1
Labels: task, ui
Milestone: V1

Title: [UI] Mailpit integration
Body: Integrasi Mailpit viewer di dashboard. Priority: P1
Labels: epic, ui
Milestone: V1

Title: [UI][Task] Embed Mailpit
Body: Integrasi viewer Mailpit via webview/iframe. Priority: P1
Labels: task, ui
Milestone: V1

## Epic: Tooling Dev & Proxy
Title: [Tooling] Virtual host manager
Body: Mapping domain ke project dengan konfigurasi proxy. Priority: P1
Labels: epic, tooling
Milestone: V1

Title: [Tooling][Task] Domain mapping config
Body: Definisikan format mapping domain -> project. Priority: P1
Labels: task, tooling
Milestone: V1

Title: [Tooling][Task] Apply mapping ke proxy
Body: Sinkronkan mapping ke konfigurasi proxy. Priority: P1
Labels: task, tooling
Milestone: V1

Title: [Tooling] Hosts file helper (opt-in)
Body: Helper update hosts file per OS dengan flow opt-in dan rollback. Priority: P1
Labels: epic, tooling
Milestone: V1

Title: [Tooling][Task] Flow opt-in hosts helper
Body: Implement flow persetujuan sebelum update hosts file. Priority: P1
Labels: task, tooling
Milestone: V1

Title: [Tooling][Task] Rollback hosts update
Body: Simpan backup hosts file dan rollback jika gagal. Priority: P1
Labels: task, tooling
Milestone: V1

Title: [Tooling] Dev TLS certs
Body: Generate dev TLS certs dan trust flow per OS. Priority: P1
Labels: epic, tooling
Milestone: V1

Title: [Tooling][Task] Generator certs
Body: Implement generator certs untuk domain lokal. Priority: P1
Labels: task, tooling
Milestone: V1

Title: [Tooling][Task] Trust flow per OS
Body: Implement trust flow untuk Windows/macOS/Linux. Priority: P1
Labels: task, tooling
Milestone: V1

Title: [Tooling] Reverse proxy router
Body: Router dan rules editor untuk reverse proxy. Priority: P1
Labels: epic, tooling
Milestone: V1

Title: [Tooling][Task] Proxy rules format
Body: Definisikan format rules dan validator. Priority: P1
Labels: task, tooling
Milestone: V1

Title: [Tooling][Task] Rules editor UI
Body: Implement UI editor untuk rules proxy. Priority: P1
Labels: task, tooling
Milestone: V1

## Epic: Packaging & Distribusi
Title: [Packaging] Tauri bundling per OS
Body: Bundling per OS dan optimasi ukuran. Priority: P0
Labels: epic, packaging
Milestone: MVP

Title: [Packaging][Task] Audit ukuran bundle
Body: Audit ukuran bundle dan identifikasi ukuran terbesar. Priority: P0
Labels: task, packaging
Milestone: MVP

Title: [Packaging][Task] Optimasi aset bundling
Body: Implement optimasi bundling untuk mengurangi ukuran. Priority: P1
Labels: task, packaging
Milestone: V1

Title: [Packaging] Signing/notarization pipeline
Body: Pipeline signing untuk Windows/macOS dan notarization macOS. Priority: P1
Labels: epic, packaging
Milestone: V1

Title: [Packaging][Task] Setup signing Windows
Body: Setup signing Windows dan dokumentasi kunci. Priority: P1
Labels: task, packaging
Milestone: V1

Title: [Packaging][Task] Setup notarization macOS
Body: Setup notarization macOS dan dokumentasi. Priority: P1
Labels: task, packaging
Milestone: V1

Title: [Packaging] Auto-update config
Body: Setup feed dan konfigurasi auto-update. Priority: P1
Labels: epic, packaging
Milestone: V1

Title: [Packaging][Task] Feed config
Body: Tentukan format feed dan endpoint update. Priority: P1
Labels: task, packaging
Milestone: V1

Title: [Packaging][Task] Update verifier
Body: Verifikasi signature update dan rollback bila gagal. Priority: P1
Labels: task, packaging
Milestone: V1

## Epic: Observability & Diagnostics
Title: [Obs] Log viewer + filter + export
Body: Log viewer dengan filter dan export. Priority: P2
Labels: epic, observability
Milestone: V2

Title: [Obs][Task] UI log viewer
Body: Implement UI log viewer dengan filter. Priority: P2
Labels: task, observability
Milestone: V2

Title: [Obs][Task] Export log
Body: Implement export log ke file. Priority: P2
Labels: task, observability
Milestone: V2

Title: [Obs] Diagnostic bundle generator
Body: Generator bundle diagnostik untuk support. Priority: P2
Labels: epic, observability
Milestone: V2

Title: [Obs][Task] Spesifikasi diagnostic bundle
Body: Tentukan isi bundle dan format output. Priority: P2
Labels: task, observability
Milestone: V2

Title: [Obs][Task] Generator bundle
Body: Implement generator bundle dan integrasi UI. Priority: P2
Labels: task, observability
Milestone: V2

Title: [Obs] Metrics ringan
Body: Metrics uptime, port usage, resource usage. Priority: P2
Labels: epic, observability
Milestone: V2

Title: [Obs][Task] Collector metrics ringan
Body: Implement collector untuk uptime, port usage, resource usage. Priority: P2
Labels: task, observability
Milestone: V2

Title: [Obs][Task] UI metrics panel
Body: UI panel untuk menampilkan metrics. Priority: P2
Labels: task, observability
Milestone: V2

## Epic: Testing & QA
Title: [QA] Smoke tests per OS
Body: Smoke test start/stop seluruh layanan per OS. Priority: P0
Labels: epic, qa
Milestone: MVP

Title: [QA][Task] Script smoke tests
Body: Implement script smoke tests untuk layanan utama. Priority: P0
Labels: task, qa
Milestone: MVP

Title: [QA][Task] Matrix OS setup
Body: Setup matrix OS di CI untuk smoke tests. Priority: P0
Labels: task, qa
Milestone: MVP

Title: [QA] Integration tests antar layanan
Body: Tes integrasi antar layanan (DB, mail, app). Priority: P1
Labels: epic, qa
Milestone: V1

Title: [QA][Task] Rencana test integrasi
Body: Definisikan skenario test integrasi antar layanan. Priority: P1
Labels: task, qa
Milestone: V1

Title: [QA][Task] Implement test integrasi
Body: Implement test integrasi untuk DB dan Mailpit. Priority: P1
Labels: task, qa
Milestone: V1

Title: [QA] Upgrade/downgrade validation
Body: Validasi upgrade/downgrade. Priority: P2
Labels: epic, qa
Milestone: V2

Title: [QA][Task] Test plan upgrade/downgrade
Body: Rancang test plan upgrade/downgrade dan data migration. Priority: P2
Labels: task, qa
Milestone: V2

Title: [QA][Task] Implement validation
Body: Implement validation upgrade/downgrade di CI. Priority: P2
Labels: task, qa
Milestone: V2

## Epic: Dokumentasi & Support
Title: [Docs] Getting started + troubleshooting
Body: Dokumentasi awal dan troubleshooting. Priority: P1
Labels: epic, docs
Milestone: V1

Title: [Docs][Task] Getting started draft
Body: Buat draft getting started untuk user baru. Priority: P1
Labels: task, docs
Milestone: V1

Title: [Docs][Task] Troubleshooting common issues
Body: Daftar issue umum dan solusi. Priority: P1
Labels: task, docs
Milestone: V1

Title: [Docs] FAQ + known issues
Body: FAQ dan known issues. Priority: P2
Labels: epic, docs
Milestone: V2

Title: [Docs][Task] Kumpulkan FAQ awal
Body: Kumpulkan FAQ dari kebutuhan pengguna. Priority: P2
Labels: task, docs
Milestone: V2

Title: [Docs][Task] Known issues list
Body: Buat daftar known issues awal. Priority: P2
Labels: task, docs
Milestone: V2

Title: [Docs] Guide kontribusi
Body: Panduan kontribusi dan workflow. Priority: P1
Labels: epic, docs
Milestone: V1

Title: [Docs][Task] Contribution workflow
Body: Definisikan workflow kontribusi dan code style. Priority: P1
Labels: task, docs
Milestone: V1

Title: [Docs][Task] PR checklist
Body: Buat checklist PR untuk kontribusi. Priority: P1
Labels: task, docs
Milestone: V1
