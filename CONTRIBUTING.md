# Contributing

Thank you for helping make the starter more useful. Humans and coding agents follow the same workflow so contributions stay understandable, secure and easy to review.

Read [README.md](README.md) for orientation and [AGENTS.md](AGENTS.md) for the authoritative architecture and command contract before editing.

## First contribution

1. Install Rust/rustup and Node.js 22 or newer.
2. Start the complete local stack:

   ```bash
   cargo xtask dev
   ```

3. Open `http://localhost:8787` and `http://localhost:8787/design-system`.
4. Run the structural baseline before changing code:

   ```bash
   cargo xtask check
   ```

5. Pick a focused issue or one product boundary. Avoid mixing dependency upgrades, broad refactors and feature behavior in one pull request.

No Cloudflare account is required for local development. See [Local development](docs/LOCAL_DEVELOPMENT.md) if setup does not become ready.

## Find the right layer

| Change | Start here |
|---|---|
| Durable request, Queue or offline shape | `crates/contracts/` |
| Validation or framework-independent rule | `crates/domain/` |
| D1 read/write | `crates/database/` and `migrations/` |
| Page or HTMX fragment markup | `crates/templates/templates/` |
| Typed template data | `crates/templates/src/` |
| HTTP coordination and response policy | `workers/app/src/routes/` and `workers/app/src/http.rs` |
| Background retryable work | `workers/jobs/` |
| UI tokens or reusable component | `public/assets/app.css` and `/design-system` |
| Browser-only lifecycle/offline behavior | `public/assets/app.js` or `public/sw.js` |
| Developer automation or generator | `crates/xtask/` |

Read [Extending the starter](docs/EXTENDING.md) for the complete file-by-file workflow.

## Feature workflow

```text
contract → failing test → domain rule → prepared SQL → Askama page/fragment
→ response cache policy → queue/offline only when justified → verify
```

- Write the smallest focused test that describes the intended behavior and confirm it fails for the expected reason.
- Keep route handlers as coordinators; do not hide HTML, large SQL strings or browser assets inside them.
- Keep ordinary `href`, `method` and `action` fallbacks when HTMX enhances an interaction.
- Treat public caching, offline storage and Queue delivery as data-consistency decisions—not incidental optimizations.
- Update documentation when commands, paths, contracts, security boundaries or contributor expectations change.

## Pull request checklist

- [ ] The change has a clear purpose and avoids unrelated refactoring.
- [ ] New or changed behavior has a focused test that failed before implementation.
- [ ] Durable contracts are versioned and producers/consumers change together.
- [ ] D1 queries use prepared/bound parameters.
- [ ] Public/private publication, cache, CSRF and offline boundaries were reviewed.
- [ ] Queue handlers and replay remain idempotent and at-least-once safe.
- [ ] Relevant documentation and the component gallery were updated.
- [ ] `cargo xtask verify` passes.
- [ ] `cargo xtask e2e` and `cargo xtask smoke` pass when those boundaries changed.
- [ ] No secrets, generated `build/`, local Wrangler state or unrelated files are committed.
- [ ] The pull request reports the exact commands that actually ran.

## Review-friendly changes

Prefer small commits with intent-revealing messages. Explain why the change belongs at the selected boundary, call out security or migration implications, and include screenshots only when visual comparison adds information that tests cannot.

Do not deploy from a feature branch unless a human explicitly requests it. New client frameworks, build pipelines, ORMs, services or broad dependencies need an approved design with evidence that the existing stack cannot meet the need.
