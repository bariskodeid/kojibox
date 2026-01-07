# Packaging & Distribution (Tauri)

Goals:
- Small bundle size.
- Signed builds per OS.

Non-goals:
- System package managers (brew/apt) for MVP.

Packaging:
- bundle runtime assets per OS.
- prune unused binaries.

Tauri config:
- tauri.conf.json defines bundle targets per OS.
- include runtime/ as resource.

Signing:
- Windows: code signing cert.
- macOS: notarization + hardened runtime.

Build pipeline:
- release build -> sign -> notarize -> verify -> publish.

Auto-update:
- hosted feed with version metadata.
- signature verification for updates.

Update metadata:
- version, pubDate, notes, platforms

Data model:
- BuildConfig: {os, arch, channel, resources}
- UpdateFeed: {version, pubDate, notes, platforms}

API contract:
- updater.check(): {available, version}
- updater.apply(version): void

Sequence flow: Auto-update
```txt
UI -> Backend: updater.check
Backend -> Update Feed: fetch metadata
Backend -> UI: update:available(version)
UI -> Backend: updater.apply(version)
Backend -> Updater: download, verify, swap
Backend -> UI: update:progress(complete)
```
