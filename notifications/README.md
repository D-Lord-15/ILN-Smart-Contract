# @iln/notifications

Webhook and email notification service for the Invoice Liquidity Network.

## Features

- Webhook delivery with HMAC-SHA256 payload signing (`x-iln-signature`)
- Per-endpoint circuit breaker (opens after 5 consecutive failures, 10-minute cooldown, single half-open probe)
- Per-endpoint sliding-window rate limiter (1000 deliveries / hour by default)
- Subscription CRUD (`POST /webhooks`, `GET /webhooks/:id`, `DELETE /webhooks/:id`)
- Email subscription registration with verification and signed unsubscribe links
- Email delivery via the Resend SDK with invoice.funded, invoice.paid, invoice.expiring_soon, and invoice.disputed templates
- `POST /notify/email` fan-out for active email subscriptions
- 72-hour and 24-hour expiring-soon reminders derived from the invoice due date
- Vitest unit tests with a 90% coverage threshold

## Local development

```bash
cd notifications
npm install
npm run dev            # tsx src/index.ts
npm run test:coverage  # vitest with 90% threshold + lcov report
```

## Environment

| Variable | Default | Description |
|----------|---------|-------------|
| `PORT`   | `3001`  | HTTP listen port (also serves `/health`) |
| `NOTIFICATIONS_DB_PATH` | tmpdir SQLite file | Persistence path for webhook and email subscriptions |
| `NOTIFICATIONS_PUBLIC_URL` | `http://localhost:3001` | Base URL used in verification and unsubscribe links |
| `EMAIL_FROM` | `ILN Notifications <noreply@iln.dev>` | Sender address used for outgoing email |
| `EMAIL_TOKEN_SECRET` | development secret | HMAC secret for verification and unsubscribe tokens |
| `RESEND_API_KEY` | unset | Optional API key for real email delivery via Resend; unset values use a logged preview transport |

## Email subscriptions

- `POST /subscriptions/email` with `{ address, email, events }`
- `GET /subscriptions/verify?token=...` to activate a pending subscription
- `DELETE /subscriptions/email?token=...` to unsubscribe with the signed footer token
- `POST /notify/email` with `{ type, invoiceId, token, amount, dueDate, freelancer?, payer?, funder?, invoiceUrl? }`

The verification and notification emails both include a signed unsubscribe link in the footer for compliance.

## Docker

```bash
# Build the image (multi-stage; final image runs as non-root)
docker build -t iln/notifications:latest ./notifications

# Run standalone
docker run --rm -p 3001:3001 iln/notifications:latest

# Or via docker-compose (includes Postgres dependency)
docker compose up -d notifications
```

The image exposes a `HEALTHCHECK` against `GET /health` and listens on port `3001`.
