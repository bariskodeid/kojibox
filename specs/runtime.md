# Runtime & Bundle

Goals:
- Portable runtime with no system-level install.
- Deterministic versions per service and OS/arch.

Non-goals:
- Installing services globally.
- Modifying system PATH or system services.

Directory layout (concept):
- runtime/
  - bin/{service}/{version}/{os}-{arch}/
  - data/{service}/
  - logs/{service}/
  - config/{service}/
  - temp/
  - manifest.json

Manifest (runtime/manifest.json):
- version: "1"
- services:
  - name, version, os, arch, checksum, size, binPath, defaultPorts, env, args
- bundle:
  - createdAt, source, signature

Manifest example fields:
- checksum: sha256 hex for binary or archive
- binPath: relative path under runtime/
- defaultPorts: list of {name, port, protocol}
- env: map of default environment variables
- args: list of default args per service

Runtime execution:
- All services run with a scoped PATH that prepends runtime/bin paths.
- No modification to system PATH or global env.
- Service-specific env is merged from:
  1) base env (safe defaults),
  2) service template config,
  3) project overrides,
  4) user overrides.

Binary acquisition:
- Downloaded archives are stored in runtime/cache/.
- Extracted binaries are placed in runtime/bin/ with checksum validation.
- Cache cleanup based on size threshold and last used time.

Permissions:
- Ensure runtime/bin is executable on macOS/Linux.
- Data and logs must be user-writable.

Disk usage:
- Track per service usage and expose totals to diagnostics.

OS notes:
- Windows: prefer .exe, include required DLLs in same bin dir.
- macOS: use codesigned binaries if required by notarization.
- Linux: bundle glibc-compatible builds.

Data model:
- RuntimeManifest (see `specs/api.md`)
- ServiceBinary (see `specs/api.md`)
- PortDef (see `specs/api.md`)

API contract:
- runtime.getManifest(): RuntimeManifest
- runtime.ensureService(name, version): ServiceBinary

Sequence flow: Acquire runtime binary
```txt
UI -> Backend: runtime.ensureService("php", "8.3.x")
Backend -> Runtime Manager: check runtime/cache
Runtime Manager -> Downloader: fetch archive if missing
Downloader -> Runtime Manager: checksum verified
Runtime Manager: extract to runtime/bin
Backend -> UI: ServiceBinary
```
