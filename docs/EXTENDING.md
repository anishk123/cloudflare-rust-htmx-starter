# Extending the starter

Use this guide to add product behavior without losing the boundaries that keep the repository fast to understand and safe to change.

## Feature workflow

```text
contract → failing test → domain rule → prepared SQL → Askama page/fragment
→ route policy → queue/offline only when justified → verify
```

Skip a stage only when the feature genuinely does not need that boundary. A visual copy change may stop at a template; a new Queue message must begin with a versioned contract.

## Where does this change go?

| Responsibility | Location |
|---|---|
| Request, Queue, offline or event shape | `crates/contracts/` |
| Validation and framework-independent behavior | `crates/domain/` |
| Prepared D1 queries and row mapping | `crates/database/` |
| D1 schema evolution | `migrations/` |
| Typed page/fragment data | `crates/templates/src/` |
| External HTML | `crates/templates/templates/` |
| HTTP/auth/cache coordination | `workers/app/src/` |
| Retryable asynchronous work | `workers/jobs/` |
| Design tokens and reusable components | `public/assets/app.css` |
| Browser lifecycle/offline behavior | `public/assets/app.js` and `public/sw.js` |
| Developer/generator/deployment automation | `crates/xtask/` |

Prefer extending a focused existing file over creating a new abstraction. Split a file when it owns multiple independent responsibilities—not merely because a new feature exists.

## A safe capability, step by step

1. **Define the boundary.** Add or evolve a Serde/Schemars contract when data crosses HTTP, Queue, offline or long-lived storage boundaries.
2. **Prove the rule.** Write a focused domain/contract test and confirm it fails for the expected missing behavior.
3. **Implement domain behavior.** Keep it independent of workers-rs, HTML and SQL when possible.
4. **Persist deliberately.** Add a migration when needed and use prepared/bound D1 parameters in the repository crate.
5. **Render externally.** Add the smallest typed view model and edit ordinary HTML templates.
6. **Coordinate HTTP.** Route handlers load identity/data, call domain/repository functions, render, and choose the central response policy.
7. **Add asynchronous/offline behavior only with semantics.** Define idempotency, retry, conflict and credential handling first.
8. **Verify.** Run focused tests, then the relevant e2e/smoke boundary and `cargo xtask verify`.

## Templates and UI

Edit page and fragment markup in `crates/templates/templates/`. Rust structs and render adapters in `crates/templates/src/` should provide only the values the template needs. Keep route handlers free of large HTML trees.

Example boundary:

```text
workers/app/src/routes/notes.rs
  → loads owner-scoped Note values
  → starter_templates::render_note_card(...)
  → crates/templates/templates/fragments/note_card.html
```

Askama compile-checks template references and escapes ordinary values. Prepare canonical URLs, formatted dates and safely serialized JSON-LD in focused Rust helpers rather than adding executable inline scripts.

For visual changes:

1. change tokens before component selectors;
2. compose existing layout/component classes before adding a new one;
3. exercise reusable states on `/design-system`;
4. check light/dark mode, reduced motion, keyboard focus and a 390-pixel viewport;
5. keep starter-owned CSS at or below 24 KiB and JavaScript at or below 8 KiB.

## Progressive enhancement

Every important link keeps `href`. Every mutation form keeps ordinary `method` and `action` alongside HTMX attributes:

```html
<form method="post" action="/notes"
      hx-post="/notes" hx-target="#notes" hx-swap="afterbegin">
  …
</form>
```

Custom browser code uses document-level event delegation or `htmx.onLoad()` so fragment swaps do not remove behavior. Do not use inline event handlers or executable inline scripts.

## Cache and publication decisions

Dynamic response policy lives in `workers/app/src/http.rs`:

- workspace/authenticated/mutation/error/CSRF responses: `CachePolicy::Private`;
- explicitly published representations: public content policy;
- sitemap/robots/`llms.txt`: discovery policy;
- static files: `public/_headers`, not route code.

Selecting a public cache policy is a publication decision. A `404` is `no-store` so a pre-publication request cannot hide newly published content at an edge location.

## Queue and offline decisions

Queue consumers receive small versioned ID/reference messages and assume at-least-once delivery. Keep the business mutation and processed-operation marker atomic where possible.

Store offline data only when the product defines replay and conflict behavior. Never persist CSRF/session credentials in IndexedDB. Replay fetches a current token from authenticated, `no-store` `/session/csrf` before mutation.

## Verification by boundary

| Change | Minimum focused evidence |
|---|---|
| contract/domain | package unit tests |
| template/fragment | `cargo test -p starter-templates` |
| Worker policy | `cargo test -p starter-app-worker` + Wasm check |
| D1/Queue/public route | `cargo xtask e2e` |
| generator/path/onboarding | `cargo xtask smoke` |
| CSS/JS/service worker | `cargo xtask check`, syntax checks and browser QA |

Finish every contribution with `cargo xtask verify` and report the exact commands.

## Next step

Read [Contracts](CONTRACTS.md) before evolving durable data, [Architecture](ARCHITECTURE.md) for boundary rationale, or [Contributing](../CONTRIBUTING.md) before opening a pull request.
