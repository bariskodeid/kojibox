# Update Feed & Signing Key Management

This document defines update feed configuration and signing key handling.

## Update Feed Config

Feed URL:
- app/config/app.json: updateFeedUrl
- Example: "https://updates.kojibox.dev/feed.json"

Public keys:
- app/config/app.json: updatePublicKeys (array of base64 strings)

Metadata:
- version
- pubDate
- notes
- platforms: [{os, arch, url, checksum}]

Client rules:
- Only apply updates matching OS/arch.
- Verify checksum and signature before apply.
- Support stable/beta channels via separate feeds.

## Signing Key Management

Key types:
- Windows: code signing cert (PFX)
- macOS: Developer ID cert + notarization key
- Update signing: Ed25519 private key

Storage:
- CI secrets store for signing keys.
- Never store private keys in repo.

Rotation:
- Rotate update signing key every 6-12 months.
- Keep previous public keys for update validation.

Verification:
- Embed public keys in app binary.
- Verify update signature before apply.

Key configuration example:
```json
{
  "updateFeedUrl": "https://updates.kojibox.dev/feed.json",
  "updatePublicKeys": [
    "base64-encoded-ed25519-public-key-v1",
    "base64-encoded-ed25519-public-key-v2"
  ]
}
```

Public key embedding:
- Place updatePublicKeys in app/config/app.json at build time.
- Rotate by appending new key before removing old key.

Rotation rules:
- Keep at least one previous public key during rotation window.
- Remove old keys after all supported versions have updated.

Signing steps (CI):
- Windows: signtool sign with timestamp server.
- macOS: codesign + notarytool submit.
- Update feed: sign metadata payload and include signature field (Ed25519).

Signature:
- Field: signature (base64)
- Payload: JSON string of feed without signature field
- Verification: use updatePublicKeys in app config
