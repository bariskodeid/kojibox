# Telemetry Upload & Retry Policy

Endpoint:
- POST https://telemetry.kojibox.dev/events

Headers:
- Content-Type: application/json

Payload:
- Batch of events (see schema)

Response:
- 202 Accepted: {accepted: number}
- 400 Bad Request: invalid payload
- 429 Too Many Requests: respect Retry-After

Retry policy:
- Retry up to 5 times.
- Exponential backoff: 1s, 2s, 4s, 8s, 16s.
- Abort on 4xx (except 429).
- Respect 429 with Retry-After header.

Privacy:
- Do not include IP, user identifiers, or project paths.

Example request:
```json
{
  "events": [
    {
      "event": "app_start",
      "ts": "2025-01-01T00:00:00Z",
      "appVersion": "1.0.0",
      "os": "windows",
      "payload": {"channel": "stable"}
    }
  ]
}
```

Example response:
```json
{"accepted": 1}
```
