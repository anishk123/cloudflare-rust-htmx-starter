# Security architecture

Keep Cloudflare control-plane tokens outside Worker runtime. Use prepared D1 SQL, explicit public/draft state, no private cache, CSP-safe external scripts, upload allowlists/limits, versioned contracts and idempotency keys. Add authentication/authorization server-side before exposing tenant data. Reassess IndexedDB data classification before storing sensitive data offline.

## Rate limiting

All mutation routes — `POST /notes` (create), `POST /notes/:id/summarize`, `POST /notes/:id/publish`, and `POST /upload` — are guarded by the Cloudflare Rate Limiting API (`RATE_LIMITER` binding in `workers/app/wrangler.jsonc`). Limits are keyed on the owner id from the request's user context, so they are per-user rather than per-IP.

- The default is 60 calls per minute per user across those routes. Tune `simple.limit`/`simple.period` (period must be 10 or 60) to your traffic.
- `namespace_id` is a string integer you define that must be unique within your Cloudflare account; change `"1001"` if you already use it.
- Limited requests get `429` with a `Retry-After` header and an `ApiError` body.
- Rate limits are **local to the Cloudflare location** serving the request and intentionally approximate (permissive, eventually consistent) — treat them as an abuse brake, not exact accounting.

## Upload content verification

`POST /upload` verifies that the client-declared content type matches the file's actual bytes (`crates/domain::verify_upload`, allowlist in `ALLOWED_UPLOAD_TYPES`):

- Binary formats (`image/png`, `image/jpeg`, `image/webp`, `application/pdf`) must start with their magic signatures — PNG `\x89PNG\r\n\x1a\n`, JPEG `\xFF\xD8\xFF`, WebP `RIFF....WEBP`, PDF `%PDF-`.
- Text formats (`text/plain`, `text/markdown`) have no reliable signature, so they are checked heuristically: valid UTF-8, no NUL bytes, and no HTML/script markers (`<script`, `<!doctype`, `<html`) — the stored-XSS smuggling vector.
- Mismatches return `415` and nothing is stored.

The same rule set is unit-tested for every accepted format in `crates/domain`.

## Upload quotas

`POST /upload` enforces a per-owner storage quota in addition to the per-file 10 MiB cap and content-type verification above. `crates/domain::UPLOAD_QUOTA_BYTES` defaults to 100 MiB per owner, tracked in the `upload_usage` D1 table (migration `0003_upload_quotas.sql`). The check runs before the R2 write (`413` on over-quota); usage is recorded after. Concurrent uploads can both pass the check-then-write race, and a failed usage record leaves an uncounted object — both cause accounting drift only, never a quota bypass.
