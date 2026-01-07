# UI Error State Checklist

Checklist for consistent error state handling in UI.

General:
- Show error title and short detail.
- Provide at least one primary action.
- Keep destructive actions behind confirmation.
- Log error to diagnostics bundle.

Port conflict:
- Offer auto-assign action.
- Provide manual override.

Config invalid:
- Show file path and schema version.
- Offer reset to defaults.

Runtime missing:
- Show service name and version.
- Provide download action.

Update signature invalid:
- Block update and show contact support link.
- Offer retry after network change.
