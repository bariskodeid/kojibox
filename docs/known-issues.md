# Known Issues

- Hosts helper requires admin privileges to modify the OS hosts file.
- Some runtime binaries may be missing until the installer downloads the runtime pack.
- Update checks fail if `updatePublicKeys` is empty or the feed signature is invalid.
- Local ports can conflict with other services; use auto-assign or manual override.
