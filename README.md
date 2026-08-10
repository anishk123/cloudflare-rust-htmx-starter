# Rust + HTMX Cloudflare Starter

[![CI](https://github.com/anishk123/cloudflare-rust-htmx-starter/actions/workflows/ci.yml/badge.svg)](https://github.com/anishk123/cloudflare-rust-htmx-starter/actions/workflows/ci.yml)
[![Security](https://github.com/anishk123/cloudflare-rust-htmx-starter/actions/workflows/security.yml/badge.svg)](https://github.com/anishk123/cloudflare-rust-htmx-starter/actions/workflows/security.yml)
![Rust](https://img.shields.io/badge/Rust-1.97.1-000000?logo=rust)
![workers-rs](https://img.shields.io/badge/workers--rs-0.8.5-F38020?logo=cloudflare)
![HTMX](https://img.shields.io/badge/HTMX-2.0.10-3366CC)
![Askama](https://img.shields.io/badge/Askama-0.16.0-6B7280)
![Wrangler](https://img.shields.io/badge/Wrangler-4.120.0-F38020?logo=cloudflare)
![License](https://img.shields.io/badge/license-MIT-green)

A cloneable, mobile-first Cloudflare application foundation where **Rust renders external Askama templates**, **HTMX handles server interaction**, and a small starter-owned design system makes ordinary product screens polished without a frontend build pipeline. Tiny vanilla JavaScript is reserved for browser-only capabilities such as PWA/offline state.

There is **no Python source, package.json, frontend framework, JavaScript bundler, CSS compiler, ORM, or Node application runtime**. Wrangler still requires Cloudflare's Node-based tooling; the Rust `xtask` hides that implementation detail behind one command surface.

## The stack — and why each part exists

| Layer | Choice | Why |
|---|---|---|
| Edge runtime | Cloudflare Workers | global serverless runtime, native bindings, local `workerd` simulation |
| Application language | Rust → Wasm | small deterministic runtime artifact, strong types, one backend/tooling language |
| Worker SDK | `workers-rs` 0.8.5 | Cloudflare-maintained Rust bindings for fetch, D1, R2, Queues and more |
| Router | `workers-rs::Router` | avoids adding another web framework |
| HTML | Askama 0.16 | ordinary `.html` files, compile-time checking, escaping by default, typed page models |
| Hypermedia | HTMX 2.0.10 | server-owned state and partial HTML updates without hydration |
| HTTP error targeting | response-targets 2.0.4 | maps 4xx/5xx responses to error regions without custom request JS |
| CSS | starter-owned `app.css` | explicit tokens, cascade layers, accessible components, no framework contract to fight |
| Browser JS | `app.js` | only IndexedDB, PWA lifecycle, network status, install prompt; lifecycle-safe |
| Database | D1 + prepared SQL | Cloudflare-native SQLite semantics, no Wasm ORM overhead |
| Objects | R2 | large uploads/artifacts belong outside relational rows |
| Async work | Cloudflare Queues | retries, batching, DLQ; separate Worker isolates background work |
| Contracts | Serde + Schemars | one Rust model for validation/types plus machine-readable JSON Schema |
| Observability | Workers Logs + structured Rust events | persisted platform logs with request/job correlation; tune sampling for volume |
| Automation | Rust `xtask` | one intuitive command surface for humans and coding agents |

## Architecture

```text
phone / tablet / desktop / crawler
                 │
       semantic HTML + HTMX
                 │
                 ▼
          Rust app Worker
       ┌─────────┼──────────┐
       ▼         ▼          ▼
      D1         R2      Queue producer
                              │
                              ▼
                       Rust jobs Worker
                              │
                              ▼
                             D1
```

Cloudflare Static Assets serves CSS, JavaScript, icons, the manifest, and the offline page ahead of Rust. One HTTP Worker remains both page renderer and API boundary: with HTMX, splitting web/API Workers adds ceremony without useful separation. Background jobs remain separate because retry/failure/scaling semantics are genuinely different.

## Quick start — local first

Prerequisites: Rust/rustup. For Cloudflare local simulation/deploy, install Node 22+ **only so Wrangler can run**; there is no npm application dependency tree.

```bash
git clone <starter-repository-url> my-app
cd my-app

cargo xtask dev         # first run bootstraps tooling/assets automatically
```

Open `http://localhost:8787`.

`cargo xtask dev` checks/bootstrap prerequisites, vendors the pinned browser assets, applies local migrations, and starts both Workers with persistent local D1/R2/Queue state under `.wrangler/state`. You do **not** need a Cloudflare account for ordinary feature development.

Reset local infrastructure:

```bash
rm -rf .wrangler/state
cargo xtask dev
```

## One-command developer experience

```bash
cargo xtask help
cargo xtask versions
cargo xtask vendor
cargo xtask dev         # first run bootstraps tooling/assets automatically
cargo xtask test
cargo xtask e2e
cargo xtask verify
```

The same commands are the contract for coding agents. See `AGENTS.md`.

## Create a new app from the starter

```bash
cargo xtask new my-product \
  --title "My Product" \
  --dir ../my-product \
  --github my-org/my-product

cd ../my-product
cargo xtask dev
```

The generator copies the starter while excluding `.git`, build/local Wrangler state, and local credential files; it rewrites Worker/resource names, app copy, PWA identifiers and CI badges, and resets Cloudflare database IDs so a new app can never accidentally point at the source project's D1 database.

For a normal Git clone that you want to rename in place:

```bash
cargo xtask configure my-product \
  --title "My Product" \
  --github my-org/my-product
```

## Cloudflare API token — exact setup

The starter can deploy using Cloudflare interactive login or a scoped API token. For repeatable automation, use a scoped token.

Cloudflare Dashboard → **My Profile → API Tokens → Create Token → Create Custom Token**.

Grant **Edit** permissions at the Account scope:

| Permission | Why |
|---|---|
| Workers Scripts | deploy app and jobs Workers |
| D1 | create DB and apply migrations |
| Workers R2 Storage | create/manage attachment bucket |
| Queues | create jobs Queue and DLQ |

Scope the token to **only the intended account**. Zone/DNS permissions are not needed unless you later automate domains or routes.

```bash
export CLOUDFLARE_API_TOKEN="..."
export CLOUDFLARE_ACCOUNT_ID="..."

cargo xtask whoami
cargo xtask deploy
```

`cargo xtask deploy` runs verification, provisions/reuses resources, applies migrations, then deploys the jobs Worker followed by the app Worker. The lower-level `provision` and `migrate` commands remain available when you want to run those steps separately.

Provisioning creates or reuses:

```text
<app>-db
<app>-attachments
<app>-jobs
<app>-jobs-dlq
```

and writes the real D1 UUID into both Worker configs.

**Never put the Cloudflare control-plane token in `.dev.vars` or Worker bindings.** It belongs only in the developer/CI environment. For GitHub deployment workflows, store `CLOUDFLARE_API_TOKEN` and `CLOUDFLARE_ACCOUNT_ID` as protected repository/environment secrets.

## Authentication and application secrets

The example is intentionally **unauthenticated** so the generic starter does not force an identity vendor. `no-store` prevents caching; it does not make a route private. Before using real user data, add an explicit authorization boundary. `docs/AUTH.md` covers Cloudflare Access vs. public SaaS/OIDC choices, cookie/CSRF requirements, and server-side authorization rules.

Keep Cloudflare control-plane credentials separate from application runtime secrets. Runtime API keys should use Cloudflare secrets (`wrangler secret put`) and ignored `.dev.vars` locally; see `docs/SECRETS.md`.

## PWA and cross-device behavior

The included Evidence Notes example demonstrates:

- installable manifest/icons;
- explicit service-worker update/reload lifecycle that does not replace an active editing session;
- responsive layout and 44px-friendly controls;
- safe-area/mobile viewport support;
- system dark mode and reduced motion;
- service-worker app-shell cache;
- IndexedDB mutation outbox;
- stable operation IDs for offline replay;
- explicit online/offline status;
- **no private note-page caching**.

Offline scope is deliberately conservative: local note capture is supported; AI calls, external publishing, authentication changes and other server-dependent actions should remain online-only unless a product explicitly designs their conflict semantics.

## UI rules

The hierarchy for humans and agents is:

```text
native HTML → CSS → HTMX → tiny vanilla JS → another library only with evidence it is needed
```

Prefer native `<dialog>`, popover, `<details>/<summary>`, HTML constraint validation, `<progress>`, semantic landmarks and real forms/links. `app.js` uses document-level event delegation and `htmx.onLoad()` so HTMX swaps cannot silently remove behavior.

`public/assets/app.css` is the design system. Its `reset`, `tokens`, `base`, `layout`, `components`, and `utilities` cascade layers make ownership predictable for humans and coding agents. Change tokens first, compose the documented layout/component classes second, and add a focused selector only when the component gallery cannot express the product need. Use `/design-system` as the live visual inventory.

## Progressive enhancement

Important interactions have ordinary web fallbacks:

```html
<form method="post" action="/notes"
      hx-post="/notes" hx-target="#notes">…</form>
```

A browser can still submit or navigate if HTMX fails. There are no inline `onclick` handlers or executable inline application scripts, allowing a restrictive Content Security Policy.

## D1 / R2 / Queue conventions

- D1: authoritative relational state and idempotency records.
- R2: uploads and large generated artifacts; rows store keys/metadata, not blobs.
- Queue: small versioned messages containing IDs/references.
- Queue consumers must assume at-least-once delivery and make business mutation + processed marker atomic where possible.
- Use prepared/bound SQL; never interpolate untrusted values.

## SEO and AEO

Only explicitly published records enter public surfaces. The example exposes the same canonical record as:

```text
HTML      /notes/:id/:slug
Markdown  /notes/:id.md
JSON      /notes/:id.json
JSON-LD   embedded in semantic HTML
```

It also includes `robots.txt`, `sitemap.xml`, `llms.txt`, canonical URLs and server-rendered content. Do not serve materially different facts based on crawler user-agent; multiple representations must remain faithful to the same source model.

Published HTML and data use separate browser and Cloudflare edge freshness policies. Workspace HTML, mutations, health responses, and the CSRF token endpoint are always `Cache-Control: no-store`. Static Assets owns browser files; only content-addressed vendor files receive one-year immutable caching. The service worker caches a token-free shell and never caches note/workspace routes.

## Performance contract

The fast path has no hydration, web font, runtime CDN, CSS compiler, or application bundle. HTML streams from the nearest Worker while static files use Cloudflare Static Assets and native browser caching. Structural checks enforce uncompressed budgets of 24 KiB for starter-owned CSS and 8 KiB for starter-owned JavaScript. `.github/lighthouse/budgets.json` records the transport and Core Web Vitals targets for teams that add Lighthouse CI or equivalent real-device monitoring.

These defaults create a very small rendering workload, including on slow devices, but they are budgets rather than a promise about every future app. Measure production field data, image weight, database latency, and third-party scripts as the product grows.

## Contracts

Durable/request boundaries live in `crates/contracts` and derive `Serialize`, `Deserialize`, and `JsonSchema`. `/contracts` exposes the catalog as JSON Schema for agents/integrations.

When adding a feature:

```text
contract → test → domain rule → prepared SQL → route/fragment → queue/offline only if needed
```

See `docs/CONTRACTS.md` and `AGENTS.md`.

## Testing and CI

Run locally:

```bash
cargo xtask test
cargo xtask e2e
cargo xtask verify
```

GitHub Actions runs:

- starter structural/best-practice checks;
- `cargo fmt --check`;
- Clippy with warnings denied;
- all workspace tests;
- `wasm32-unknown-unknown` check;
- release builds for app and jobs Workers;
- local multi-Worker end-to-end smoke covering D1 + Queue + publish/JSON;
- browser JS syntax checks;
- project-generator smoke test;
- `cargo audit` on PR/push/weekly schedule;
- GitHub dependency review on PRs;
- Dependabot for Cargo and Actions.

Status badges become truthful only after the generated repo is pushed and the first workflow finishes. The repository also includes a manual `Deploy` workflow using a protected `production` environment; it never deploys on ordinary pushes. Workers Logs are explicitly enabled. For high-volume production apps, tune Cloudflare log sampling rather than disabling structured application events.

Do not point preview/staging Workers at production D1/R2/Queues. If you add preview deployments, create separate resources and environment bindings first.

## Agent use

Point Codex, Claude or OmniAgent at the repository and say:

> Read `AGENTS.md` and `README.md` first. Run `cargo xtask check` before changing code. Build the requested feature within the existing Rust/HTMX/Cloudflare boundaries. Define durable contracts first, add tests before implementation, use prepared D1 SQL, preserve progressive enhancement and HTMX lifecycle safety, keep Queue handlers idempotent, and run `cargo xtask verify` before reporting completion. Do not deploy unless explicitly asked.

Tool-specific entry files (`CLAUDE.md`, `CODEX.md`, `OMNIAGENT.md`) all point back to the same authoritative contract to prevent instruction drift.

## Repository map

```text
workers/app/          small route table plus focused auth/http/route modules
workers/jobs/         Queue consumer
crates/contracts/     versioned Serde/Schemars boundaries
crates/domain/        framework-independent business rules
crates/database/      prepared D1 SQL repositories
crates/templates/     typed Askama view models and external HTML templates
crates/observability/ structured events
crates/shared/        small dependency-light helpers
crates/xtask/         all developer automation
public/assets/vendor/ versioned browser libraries served from first-party origin
public/assets/app.css starter-owned Quiet Product design system
public/assets/app.js  lifecycle-safe PWA/offline browser capabilities
public/sw.js          conservative app-shell service worker
public/offline.html   static, credential-free offline fallback
migrations/           D1 schema
```

## Security defaults

- restrictive CSP and common browser security headers;
- no inline application JS;
- no control-plane secrets in Worker runtime;
- no private-response caching;
- upload MIME allowlist with magic-byte verification + size cap;
- public/draft state boundary;
- prepared SQL;
- idempotent sync/jobs;
- Dependabot, dependency review and cargo-audit;
- minimal Cloudflare API-token scope.

Read `docs/SECURITY.md` and `SECURITY.md` before adding authentication, payments or sensitive data.

## Third-party assets

See `THIRD_PARTY_NOTICES.md`. `cargo xtask bootstrap` vendors the pinned upstream browser assets into `public/assets/vendor/`; Cloudflare Static Assets serves them from this application origin, so there is no runtime CDN dependency.

## License

MIT.
