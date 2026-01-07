# Validation Errors & Remediation

This document defines runtime validation error mapping and remediation steps.

## Error Mapping

PORT_CONFLICT:
- Cause: Requested port is already in use.
- Remediation: Offer auto-assign next available port or manual override.

CONFIG_INVALID:
- Cause: Config schema validation failed.
- Remediation: Show file path and expected schema version.

RUNTIME_MISSING:
- Cause: Required runtime binary not found.
- Remediation: Trigger runtime.ensureService and retry.

PROJECT_PATH_INVALID:
- Cause: Project path missing or not readable.
- Remediation: Prompt user to select a valid folder.

UPDATE_SIGNATURE_INVALID:
- Cause: Update feed signature mismatch.
- Remediation: Abort update and prompt retry from trusted network.

SECRET_ACCESS_DENIED:
- Cause: Secret retrieval failed or access blocked.
- Remediation: Prompt user to re-enter secret or unlock keychain.

## UX Guidance

- Always show the affected module and suggested action.
- Provide a "Retry" button when retryable.
- Log error details to diagnostics bundle.
