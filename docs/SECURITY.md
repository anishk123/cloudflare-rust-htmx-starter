# Security architecture

The starter provides strong defaults, not a complete threat model for every product. Review this guide before adding authentication, tenant data, payments, untrusted rich content or privileged integrations.

## Defaults already enforced

- restrictive CSP and common browser security headers;
- no executable inline application scripts or event handlers;
- prepared/bound D1 SQL;
- owner-scoped private repositories and explicit draft/public state;
- fail-closed double-submit CSRF on mutations;
- `no-store` private, mutation, CSRF, error and negative responses;
- current-slug canonical redirects and published-only discovery;
- mutation rate limits keyed by owner context;
- upload size, quota, content-type and byte-signature checks;
- versioned durable contracts and idempotent Queue/offline operations;
- control-plane/runtime secret separation;
- conservative PWA and IndexedDB scope.

## Cache and offline boundary

All dynamic responses pass through `workers/app/src/http.rs`. `CachePolicy::Private` sets `Cache-Control: no-store` and never emits a CDN cache directive. Only published note representations and public discovery routes opt into browser plus edge freshness.

Treat selecting a public policy as a data-publication decision, not a performance tweak. Static file caching belongs in `public/_headers`. The service worker stores a public shell and static offline page; it must never store workspace HTML, note routes, authenticated responses or CSRF values.

IndexedDB entries contain intentionally offline-capable content and stable operation IDs only. Replay obtains a current token from authenticated, `no-store` `/session/csrf` after reconnecting.

## Authentication and authorization

The demo development identity is not a credential. Before real data, add verified authentication and enforce authorization in every repository read/mutation. UI visibility, route obscurity, robots rules and `no-store` are not authorization.

See [Authentication](AUTH.md) for ownership, CSRF and session integration requirements.

## Rate limiting

All mutation routes use the Cloudflare Rate Limiting API binding from `workers/app/wrangler.jsonc`. The default is 60 calls per minute per owner across create, summarize, publish and upload.

Cloudflare limits are local to the serving location and intentionally approximate. Use them as an abuse brake, not exact billing, quota or authorization enforcement. Limited requests return `429` with `Retry-After` and an `ApiError` body.

## Upload verification

`POST /upload` requires an allowed declared type and verifies actual content:

- PNG, JPEG, WebP and PDF require the expected magic signatures;
- plain text and Markdown must be valid UTF-8 without NUL bytes or obvious HTML/script document markers;
- each file is capped at 10 MiB;
- owner usage defaults to a 100 MiB quota tracked in D1;
- R2 keys are owner-namespaced.

Concurrent uploads and a failed post-write usage record can create accounting drift. The starter reduces accidental overuse but is not a transactional billing-grade quota system; products requiring strict enforcement should add reservation/reconciliation semantics.

## Public content and SEO/AEO

HTML, Markdown, JSON, JSON-LD, sitemap and `llms.txt` must remain faithful to the same explicitly published model. Do not serve different facts to crawlers. Keep owner identity and draft fields out of public contracts.

## Production-readiness checklist

- [ ] Replace the development identity with verified authentication and tested authorization.
- [ ] Classify tenant, offline, upload and log data.
- [ ] Scope/rotate control-plane and runtime secrets independently.
- [ ] Review CSP additions and every new third-party origin.
- [ ] Add product-specific abuse limits and strict quota accounting when required.
- [ ] Define backup, migration, incident and rollback procedures.
- [ ] Use isolated resources for development, preview/staging and production.
- [ ] Review observability sampling and ensure logs exclude secrets/private content.
- [ ] Run dependency/security workflows and address findings before release.
- [ ] Measure deployed performance and security behavior in the actual environment.

## Reporting a vulnerability

Follow the root [security policy](../SECURITY.md). Do not place live credentials or exploitable private-environment details in a public issue.

## Next step

Read [Secrets](SECRETS.md) for credential placement, [Deployment](DEPLOYMENT.md) for environment isolation, and [Contracts](CONTRACTS.md) before adding a new durable boundary.
