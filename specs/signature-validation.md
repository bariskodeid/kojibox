# Client Signature Validation (Spec + Pseudocode)

Goal: Ensure update feed is signed by trusted Ed25519 keys.

Inputs:
- feed JSON (with signature field)
- updatePublicKeys (array of base64 public keys)

Rules:
- Remove signature field before verification.
- Verify against each public key until one matches.
- Reject update if signature missing or invalid.

Pseudocode:
```txt
function verifyFeed(feed, publicKeys):
  signature = base64_decode(feed.signature)
  payload = JSON.stringify(feed without signature)
  for key in publicKeys:
    if ed25519_verify(key, payload, signature):
      return true
  return false
```

Error handling:
- If invalid: raise UpdateError (UPDATE_SIGNATURE_INVALID)
- If missing: raise UpdateError (UPDATE_SIGNATURE_MISSING)

## Example Implementation (Node.js)

```js
const crypto = require("crypto");

function verifyFeed(feed, publicKeys) {
  if (!feed.signature) return false;
  const signature = Buffer.from(feed.signature, "base64");
  const payload = JSON.stringify({ ...feed, signature: undefined });
  for (const keyB64 of publicKeys) {
    const key = Buffer.from(keyB64, "base64");
    const ok = crypto.verify(null, Buffer.from(payload), { key }, signature);
    if (ok) return true;
  }
  return false;
}
```

## Example Implementation (Rust)

```rust
use ed25519_dalek::{Signature, VerifyingKey};
use base64::{engine::general_purpose, Engine as _};

fn verify_feed(feed_json: &str, signature_b64: &str, public_keys: &[String]) -> bool {
    let signature_bytes = match general_purpose::STANDARD.decode(signature_b64) {
        Ok(b) => b,
        Err(_) => return false,
    };
    let signature = match Signature::from_slice(&signature_bytes) {
        Ok(s) => s,
        Err(_) => return false,
    };

    for key_b64 in public_keys {
        let key_bytes = match general_purpose::STANDARD.decode(key_b64) {
            Ok(b) => b,
            Err(_) => continue,
        };
        let Ok(key) = VerifyingKey::from_bytes(&key_bytes) else { continue };
        if key.verify_strict(feed_json.as_bytes(), &signature).is_ok() {
            return true;
        }
    }
    false
}
```
