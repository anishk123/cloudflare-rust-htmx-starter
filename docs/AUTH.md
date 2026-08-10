# Authentication boundary

The Evidence Notes example is intentionally **unauthenticated** so the starter does not force a universal identity provider or session model. `Cache-Control: no-store` is a caching policy, not access control.

## What is already enforced: ownership scoping

The data model is owner-scoped even though no identity provider is wired up. Every note carries an `owner_id` (see `migrations/0002_add_owner_id.sql`) and every private query in `crates/database` filters on it:

- `GET /` lists only the caller's notes; `create`, `publish`, and `summarize` act only on the caller's notes (a foreign or missing note is a uniform 404).
- `summarize` gates on ownership **before** enqueuing, so a foreign note never reaches the jobs queue.
- R2 upload keys are namespaced per owner: `attachments/<owner_id>/<uuid>`.
- Idempotency (`processed_operations`) is scoped per owner, so one user's `operation_id` can never collide with another's.
- Public surfaces (published HTML/Markdown/JSON, sitemap, robots) intentionally remain owner-independent.
- `owner_id` is deliberately **not** part of the public `Note` contract — published representations must not leak the owner's identity.

Until real auth lands, the caller presents an `x-user-id` header that the app worker resolves into the request's owner context (`current_user` in `workers/app/src/auth.rs`). This is a **developer placeholder, not a credential** — anyone can set it. Replacing it with real authentication must keep the same shape: resolve a user identity per request, then pass it through the same repository functions. The offline outbox replay (`public/assets/app.js`) forwards the same header from `localStorage` when present, so it automatically carries whatever auth context the browser has once real auth populates it.

For local development the worker also reads a `DEV_USER_ID` var (empty in the committed `wrangler.jsonc`; override it in the ignored `workers/app/.dev.vars`, e.g. `DEV_USER_ID=11111111-1111-4111-8111-111111111111`) so the browser demo needs no header injection. Wrangler loads `.dev.vars` from the config file's directory, so the file lives next to `workers/app/wrangler.jsonc`. Never set `DEV_USER_ID` in a production deployment — it is a dev-only stand-in for the header.

## CSRF protection (double-submit cookie)

State-changing routes (`create`, `summarize`, `publish`, `upload`) are protected by a stateless double-submit token — no session table, no crypto dependency:

1. **Mint** — the worker generates a 64-hex-char token (two UUIDv4 values from Web Crypto's CSPRNG) and sets it as a `csrf_token` cookie: `HttpOnly; SameSite=Lax; Secure; Path=/`.
2. **Embed** — every online state-changing form renders the same token as a hidden `csrf_token` field (`crates/templates/templates/`). No inline scripts are involved, so the CSP is unaffected.
3. **Validate** — on POST the worker compares the submitted field to the cookie in constant time (`crates/domain::csrf_token_valid` / `constant_time_eq`) and returns a `403` `ApiError` on mismatch, before any business logic runs.
4. **Enforce when it matters** — the check runs only when a `csrf_token` cookie is present on the request. That is exactly when cookie-session auth is live: with the dev `x-user-id` header no cookie is sent, so nothing is skipped in production once sessions exist. No feature flag needed — cookie presence is the flag.

**HTMX compatibility:** hidden inputs are serialized by HTMX like any field, and every fragment (`note_card`) is rendered per-request with the session's current token, so swapped-in forms are always valid. Do not add a token-carrying header — that is the kind of "HTMX header as CSRF" shortcut this document warns against.

**Offline replay:** IndexedDB stores only the operation id and intentionally offline-capable note fields; it never stores CSRF or session credentials. When connectivity returns, replay first calls authenticated `GET /session/csrf`, which returns a current token with `Cache-Control: no-store`, then submits each queued operation with that token and its stable operation id. A failed 4xx replay remains in the outbox as "sync needs attention" rather than being silently discarded.

**When cookie auth lands:** set the session cookie alongside this one (or reuse it as the session marker), and swap `current_user` to read the session instead of `x-user-id`. The CSRF layer, templates, and replay need no further changes.

Before storing real user/workspace data, choose authentication explicitly:

- **Internal/team app:** Cloudflare Access is often the smallest operational surface.
- **Public SaaS:** use an OIDC/OAuth provider or a deliberately implemented session layer; keep identity-provider specifics outside domain crates.
- **Cookie sessions:** use `Secure`, `HttpOnly`, and an appropriate `SameSite` policy. Protect state-changing requests against CSRF; do not rely on HTMX headers as authentication or CSRF protection.

Authorization must be enforced in the Rust Worker/repository boundary on every read and mutation. Never rely on hidden buttons, route obscurity, robots rules, or client-side checks.

Do not place application secrets in source control. For deployed Workers use Cloudflare secrets (for example `wrangler secret put NAME`); for local runtime secrets use an ignored `.dev.vars`. Keep `CLOUDFLARE_API_TOKEN` separate: it is a control-plane credential and must never become a Worker runtime binding.
