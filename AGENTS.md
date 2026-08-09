# AGENTS.md — authoritative agent contract

Read this file and `README.md` before editing. This applies to Codex, Claude, OmniAgent, and other coding agents.

## Canonical commands

```bash
cargo xtask bootstrap
cargo xtask vendor
cargo xtask dev
cargo xtask test
cargo xtask e2e
cargo xtask verify
cargo xtask check
cargo xtask smoke
cargo xtask new my-product --title "My Product" --dir ../my-product --github owner/my-product
cargo xtask configure my-product --title "My Product" --github owner/my-product
cargo xtask auth
cargo xtask whoami
cargo xtask provision
cargo xtask migrate
cargo xtask deploy
```

**Never deploy unless the human explicitly asks.**

## Architecture rules

1. Rust owns HTTP, domain logic, contracts, database access, templates, jobs, and developer automation.
2. Browser code is limited to vendored HTMX, response-targets, Pico CSS, `app.css`, `app.js`, and `sw.js`. Do not add React/Vue/Svelte/Alpine/Tailwind/npm build tooling without explicit approval.
3. Native HTML first: use `dialog`, popover, `details/summary`, semantic controls and native validation before JS.
4. Progressive enhancement: every important link has `href`; every mutation form has ordinary `method` + `action` in addition to HTMX attributes.
5. HTMX lifecycle: custom JS must use document-level event delegation or `htmx.onLoad()`. Do not attach component listeners only on initial page load.
6. Durable boundaries use versioned Serde/Schemars contracts before implementation.
7. D1 uses prepared/bound SQL. Do not interpolate untrusted input.
8. Queue consumers are idempotent and at-least-once safe; messages carry IDs/references, not large artifacts.
9. Public publication routes may return only explicitly published data. The demo itself is unauthenticated; `no-store` is not authorization. Read `docs/AUTH.md` before adding real user/workspace data.
10. PWA cache must never cache authenticated or workspace-data page responses. IndexedDB stores only intentionally offline-capable data.
11. CSP-safe: no inline event handlers or executable inline scripts.
12. Cloudflare control-plane tokens never become Worker bindings; runtime secrets use Cloudflare Secrets. Read `docs/SECRETS.md`.
13. Prefer focused crates/files and YAGNI.

## Feature workflow

contract → failing test → domain logic → prepared SQL → Maud route/fragment → queue if needed → offline semantics if appropriate → `cargo xtask verify`

## Completion

Report exactly which commands ran. If network/tooling blocks Rust/Wrangler verification, say so; never call unexecuted checks passing.
