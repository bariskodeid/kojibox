# Runtime Setup Guide

Untuk membuat Kojibox berfungsi penuh, Anda perlu mengisi folder `runtime/` dengan binari asli.

## Struktur Direktori

Pastikan struktur folder mengikuti pola ini:

```text
runtime/
└── bin/
    ├── php/
    │   └── 8.3.2/
    │       └── {platform-tag}/
    │           ├── php.exe (Windows) atau php (Linux/Mac)
    │           └── ... file pendukung lain (ext, dll)
    ├── node/
    │   └── 20.11.1/
    │       └── {platform-tag}/
    │           ├── node.exe
    │           └── npm
    ├── mariadb/
    │   └── 10.11.6/
    │       └── {platform-tag}/
    │           └── bin/
    │               └── mariadbd.exe (atau mysqld)
    └── mailpit/
        └── 1.15.0/
            └── {platform-tag}/
                └── mailpit.exe (atau mailpit)
```

## Platform Tags
- Windows: `windows-x64`
- macOS (M1/M2): `macos-arm64`
- macOS (Intel): `macos-x64`
- Linux: `linux-x64`

## Contoh Instalasi Manual (Linux)

1. **Mailpit:**
   - Download: `https://github.com/axllent/mailpit/releases/download/v1.15.0/mailpit-linux-amd64.tar.gz`
   - Ekstrak file `mailpit` ke: `runtime/bin/mailpit/1.15.0/linux-x64/mailpit`
   - Beri izin eksekusi: `chmod +x .../mailpit`

2. **Node.js:**
   - Download: `https://nodejs.org/dist/v20.11.1/node-v20.11.1-linux-x64.tar.gz`
   - Ekstrak isinya sehingga file `bin/node` berada di path yang tepat.
   - Kojibox akan mencari `runtime/bin/node/20.11.1/linux-x64/node` (atau di subfolder `bin` di dalamnya, sistem Kojibox akan memindai `PATH`).

## Otomatisasi (Production)
Untuk rilis publik, Anda sebaiknya:
1. Mengunduh semua binari ini.
2. Mengompresnya ulang menjadi file `.zip` per service (misal `php-8.3.2-windows-x64.zip`).
3. Menguploadnya ke server/bucket S3.
4. Mengupdate `runtime/sources.json` atau endpoint update feed agar Installer Kojibox bisa mengunduhnya otomatis.
