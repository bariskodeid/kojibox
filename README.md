# Kojibox

Kojibox adalah kompetitor Laragon yang portable dan cross-platform (Windows, macOS, Linux) dengan bundling PHP, Node.js, Mailpit, Postgres, MariaDB, dan layanan pendukung lainnya. UI dibangun dengan Vue dan TailwindCSS.

## Fitur Utama (Beta)
- **Portable Runtime:** Semua binari (PHP, DB, Node) terisolasi di folder `runtime/`.
- **Service Manager:** Kontrol penuh Start/Stop/Restart dengan log real-time.
- **Smart Tooling:** Otomasi Virtual Hosts, Proxy Rules, dan TLS (HTTPS) generator.
- **Project Manager:** CRUD project dengan dukungan per-stack (PHP/Node).
- **Database Helper:** Detail koneksi instan dan launcher client database.
- **PHP Extensions:** Manajemen ekstensi PHP langsung dari UI.

## Persyaratan Sistem
- Node.js v20+ (untuk development UI)
- Rust (untuk build backend)
- OS: Windows 10+, macOS (Intel/M1), atau Linux modern.

## Panduan Development

### 1. Persiapan Awal
Klon repositori dan instal dependensi:
```bash
pnpm install
```

### 2. Menjalankan dalam Mode Dev
Jalankan aplikasi dengan hot-reload:
```bash
pnpm tauri dev
```
*Catatan: Saat pertama kali dijalankan, script `prepare-runtime.cjs` akan membuat placeholder binari di folder `runtime/`.*

### 3. Build untuk Produksi
Untuk mem-package aplikasi menjadi installer:
```bash
npm run build
# Lalu
pnpm tauri build
```

## Struktur Proyek
- `src/`: Frontend Vue (Modular components).
- `src-tauri/`: Backend Rust (Service orchestration, Proxy, Config).
- `runtime/`: Lokasi binari portable (PHP, MySQL, dll).
- `app/`: Lokasi penyimpanan konfigurasi user dan log.
- `specs/`: Spesifikasi teknis lengkap.

## Telemetri
Kojibox mengumpulkan data anonim minimal (seperti versi app dan error code) untuk pengembangan, **hanya jika user memberikan izin (opt-in)** melalui wizard awal atau pengaturan.

## Lisensi
MIT License - Lihat file `LICENSE` (jika ada) untuk detail lebih lanjut.