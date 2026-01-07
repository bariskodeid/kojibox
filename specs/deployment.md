# Deployment & Release Pipeline

This document defines CI/CD, signing, and release flow for Kojibox.

## CI/CD Stages

Stages:
- lint
- build
- test
- package
- sign
- publish

Matrix:
- OS: windows, macos, linux
- Arch: x64, arm64 (macOS)

Artifacts:
- app bundle per OS
- runtime bundle per OS
- checksums (sha256)
- update feed metadata

## Build Flow

1) Install dependencies
2) Build UI (Vite)
3) Build backend (Tauri)
4) Package resources (runtime + config templates)
5) Run tests (smoke + integration where available)

## Signing

Windows:
- Sign executable with code signing certificate.
- Verify signature post-sign.

macOS:
- Sign app bundle with Developer ID.
- Notarize and staple ticket.
- Verify notarization status.

Linux:
- Optional GPG signing for artifacts.

## Release Flow

Channels:
- stable
- beta

Release steps:
1) Tag release (semver).
2) Build artifacts per OS.
3) Sign and verify.
4) Publish to release storage.
5) Update feed metadata.

## Update Feed

Feed format:
- version
- pubDate
- notes
- platforms (os, arch, url, checksum)

Example feed entry:
```json
{
  "version": "1.1.0",
  "pubDate": "2025-01-01T00:00:00Z",
  "notes": "Bug fixes and stability improvements",
  "platforms": [
    {"os": "windows", "arch": "x64", "url": "https://example.com/kojibox-win.zip", "checksum": "sha256:abc"},
    {"os": "macos", "arch": "arm64", "url": "https://example.com/kojibox-mac.zip", "checksum": "sha256:def"}
  ]
}
```

## Rollback Strategy

- Keep previous release artifacts in storage.
- Allow users to downgrade to previous stable version.
- Maintain migration compatibility for last 2 versions.
