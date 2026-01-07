# Installer & Bootstrapping

Goals:
- Smooth first-run setup.
- Fast updates and rollback safety.

Non-goals:
- System-wide installers or admin-required installs.

First-run wizard:
- Install path, default ports, optional services.
- Writes initial config and runtime manifest.

Wizard outputs:
- app/config/app.json
- app/config/services/*.json
- runtime/manifest.json

Downloader:
- Resume + retry.
- Checksum verification and rollback on failure.

Cache policy:
- Keep latest N versions per service.
- Purge on size threshold.

Offline pack:
- archive containing runtime/ + manifest.json
- import flow verifies checksum and version.

Offline pack format:
- zip or tar.gz with top-level runtime/
- manifest.json at root

Update:
- staged download to temp/
- verify signature/checksum
- swap version and retain previous for rollback

Update channels:
- stable, beta (configurable)

Data model:
- DownloadJob: {id, url, size, progress, state}
- InstallState: {phase, progress, error}
- OfflinePack: {path, version, checksum}

API contract:
- installer.status(): {phase, progress}
- installer.start(): void
- updater.check(): {available, version}
- updater.apply(version): void

Sequence flow: First-run installation
```txt
UI -> Backend: installer.start()
Backend -> Installer: init directories
Installer -> Downloader: fetch runtime archives
Downloader -> Installer: checksums verified
Installer: extract runtime and write configs
Backend -> UI: update:progress(complete)
```
