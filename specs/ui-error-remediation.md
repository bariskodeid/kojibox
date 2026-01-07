# UI Error Remediation Copy & Actions

This document defines error UI copy and recommended user actions.

PORT_CONFLICT:
- Message: "Port already in use"
- Detail: "Port {port} is used by another process."
- Actions: "Auto-assign", "Change port", "Retry"

CONFIG_INVALID:
- Message: "Configuration invalid"
- Detail: "Config file {path} failed validation."
- Actions: "Open config", "Reset to defaults"

RUNTIME_MISSING:
- Message: "Runtime missing"
- Detail: "Required runtime {service} {version} not found."
- Actions: "Download runtime", "Retry"

PROJECT_PATH_INVALID:
- Message: "Project path not found"
- Detail: "{path} is missing or unreadable."
- Actions: "Select folder", "Retry"

UPDATE_SIGNATURE_INVALID:
- Message: "Update verification failed"
- Detail: "Signature mismatch. Update blocked."
- Actions: "Retry", "Contact support"

SECRET_ACCESS_DENIED:
- Message: "Secret access denied"
- Detail: "Unable to read stored secret for {scope}."
- Actions: "Re-enter secret", "Unlock keychain"
