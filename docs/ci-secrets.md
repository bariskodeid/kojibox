# CI Secrets & Environment

This document lists secrets required by CI/CD.

## Required Secrets

Windows signing:
- WIN_CERT_PATH: path to PFX in CI
- WIN_CERT_PASS: PFX password
- WIN_TSA: timestamp server URL

macOS signing:
- MAC_CERT_ID: Developer ID identity
- MAC_PROFILE: notarytool keychain profile

Update signing:
- UPDATE_SIGNING_KEY: base64 Ed25519 private key

## Optional Secrets

Release storage:
- S3_BUCKET
- S3_ACCESS_KEY
- S3_SECRET_KEY

## Notes

- Store all secrets in GitHub Actions secrets.
- Never commit private keys to repo.

## Generate Ed25519 Keys (CLI)

Generate with OpenSSL:
```sh
openssl genpkey -algorithm ED25519 -out private.pem
openssl pkey -in private.pem -pubout -out public.pem
```

Base64 encode (single line):
```sh
base64 -w 0 private.pem > private.pem.b64
base64 -w 0 public.pem > public.pem.b64
```

Use:
- UPDATE_SIGNING_KEY = contents of private.pem.b64
- updatePublicKeys = [contents of public.pem.b64]
