# FAQ

## What is Kojibox?
Kojibox is a portable local dev stack that bundles PHP, Node.js, Mailpit, Postgres, MariaDB, and supporting services in a single app.

## Does Kojibox install global services or modify system PATH?
No. Kojibox uses a scoped runtime PATH and keeps all runtime data inside the app/runtime roots.

## Where are configs stored?
- App config: `app/config/app.json`
- Service configs: `app/config/services/*.json`
- Projects: `app/projects/{id}/config.json`

## How do I fix port conflicts?
Use auto-assign in service settings or update the port in `app/config/services/{service}.json`.

## Why is the update feed rejected?
The update feed must have a valid Ed25519 signature that matches `updatePublicKeys` in `app/config/app.json`.
