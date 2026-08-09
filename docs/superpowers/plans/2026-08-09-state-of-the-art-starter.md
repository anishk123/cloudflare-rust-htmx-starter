# State-of-the-Art Starter Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Deliver an external-template, starter-owned design system with safe Cloudflare caching, corrected SEO/AEO surfaces, and a readable Worker architecture.

**Architecture:** Askama compiles ordinary HTML files from explicit Rust view models. Cloudflare Static Assets serves browser files ahead of the Rust Worker, while a centralized HTTP policy distinguishes private `no-store` responses from explicitly published cacheable content. The existing Worker remains the single HTTP boundary but delegates auth, response policy, and route families to focused modules.

**Tech Stack:** Rust 1.97.1, workers-rs 0.8.5, Askama 0.16.0, HTMX 2.0.10, response-targets 2.0.4, Cloudflare Workers/D1/R2/Queues/Static Assets/Workers Caching, native HTML/CSS/JavaScript.

## Global Constraints

- Do not add React, Vue, Svelte, Alpine, Tailwind, npm application tooling, a CSS compiler, web fonts, or runtime CDN requests.
- Do not deploy.
- Preserve progressive enhancement: important links keep `href`; mutation forms keep `method` and `action` plus HTMX attributes.
- Preserve public/draft, owner, CSRF, cache, upload, queue-idempotency, and CSP boundaries.
- Keep starter-owned CSS at or below 24 KiB and starter-owned JavaScript at or below 8 KiB, uncompressed.
- Keep private/workspace responses `no-store`; cache only explicitly public representations.
- Use prepared D1 SQL and versioned Serde/Schemars contracts.

---

### Task 1: External typed templates and SEO view models

**Files:**
- Modify: `Cargo.toml`
- Modify: `crates/templates/Cargo.toml`
- Replace: `crates/templates/src/lib.rs`
- Create: `crates/templates/src/models.rs`
- Create: `crates/templates/src/pages.rs`
- Create: `crates/templates/src/fragments.rs`
- Create: `crates/templates/src/seo.rs`
- Create: `crates/templates/templates/layouts/base.html`
- Create: `crates/templates/templates/pages/home.html`
- Create: `crates/templates/templates/pages/public_note.html`
- Create: `crates/templates/templates/pages/error.html`
- Create: `crates/templates/templates/pages/design_system.html`
- Create: `crates/templates/templates/fragments/note_card.html`
- Create: `crates/templates/templates/fragments/error.html`

**Interfaces:**
- Produces: `PageMeta`, `NoteView`, `HomePage`, `PublicNotePage`, `ErrorPage`, `DesignSystemPage`.
- Produces: `render_home`, `render_public_note`, `render_error_page`, `render_design_system`, `render_note_card`, and `render_error_fragment`, each returning `Result<String, askama::Error>`.
- Consumes: `starter_contracts::Note`, `starter_domain::slugify`, absolute origin strings, and CSRF tokens supplied by routes.

- [ ] **Step 1: Write failing template behavior tests**

Add tests to the new `crates/templates/src/lib.rs` that demand external-template behavior:

```rust
#[test]
fn public_note_has_absolute_canonical_description_and_safe_json_ld() {
    let note = Note {
        id: Uuid::nil(),
        title: "</script><script>alert(1)</script>".into(),
        body: "Evidence body".into(),
        status: NoteStatus::Published,
        version: 1,
        created_at_ms: 1_700_000_000_000,
        updated_at_ms: 1_700_000_000_000,
        summary: None,
    };
    let html = render_public_note(&note, "https://example.test").unwrap();
    assert!(html.contains("rel=\"canonical\" href=\"https://example.test/notes/"));
    assert!(html.contains("name=\"description\""));
    assert!(html.contains("application/ld+json"));
    assert!(!html.contains("</script><script>alert(1)</script>"));
}

#[test]
fn note_fragment_preserves_ordinary_form_fallbacks() {
    let note = Note {
        id: Uuid::nil(),
        title: "Draft".into(),
        body: "Body".into(),
        status: NoteStatus::Draft,
        version: 1,
        created_at_ms: 1_700_000_000_000,
        updated_at_ms: 1_700_000_000_000,
        summary: None,
    };
    let html = render_note_card(&note, "csrf").unwrap();
    assert!(html.contains("method=\"post\""));
    assert!(html.contains("action=\"/notes/"));
    assert!(html.contains("hx-post=\"/notes/"));
}
```

- [ ] **Step 2: Run the focused tests and verify RED**

Run: `cargo test -p starter-templates`

Expected: compilation fails because the Askama-backed render API and view models do not exist.

- [ ] **Step 3: Implement the typed template boundary**

Pin Askama and time formatting:

```toml
askama = "=0.16.0"
time = { version = "=0.3.44", default-features = false, features = ["formatting"] }
```

Use explicit models:

```rust
pub struct PageMeta {
    pub title: String,
    pub description: String,
    pub canonical_url: String,
    pub og_type: &'static str,
    pub json_ld: Option<String>,
}

pub struct NoteView {
    pub id: String,
    pub title: String,
    pub body: String,
    pub summary: Option<String>,
    pub status_label: &'static str,
    pub version: u32,
    pub public_path: Option<String>,
}
```

Derive `askama::Template` for page/fragment structs, use template inheritance, and keep JSON-LD escaping in `seo.rs` by replacing `<`, `>`, and `&` with JSON Unicode escapes before using Askama's `safe` filter.

- [ ] **Step 4: Run template tests and verify GREEN**

Run: `cargo test -p starter-templates`

Expected: all template tests pass, including escaping and progressive-enhancement assertions.

- [ ] **Step 5: Commit the template migration**

```bash
git add Cargo.toml Cargo.lock crates/templates
git commit -m "refactor: move pages to typed HTML templates"
```

### Task 2: Quiet Product CSS and static asset delivery

**Files:**
- Replace: `public/app.css` with `public/assets/app.css`
- Move: `public/app.js` to `public/assets/app.js`
- Rename: `public/vendor/htmx.min.js` to `public/assets/vendor/htmx-2.0.10.min.js`
- Rename: `public/vendor/response-targets.js` to `public/assets/vendor/response-targets-2.0.4.js`
- Delete: `public/vendor/pico.min.css`
- Modify: `public/sw.js`
- Create: `public/offline.html`
- Create: `public/_headers`
- Create: `public/.assetsignore`
- Modify: `workers/app/wrangler.jsonc`
- Modify: `crates/xtask/src/main.rs`

**Interfaces:**
- Produces: asset-first `/assets/*`, `/offline.html`, icons, manifest, and service worker.
- Consumes: template asset paths from Task 1.
- Enforces: `APP_CSS_MAX_BYTES = 24 * 1024`, `APP_JS_MAX_BYTES = 8 * 1024` in `xtask check`.

- [ ] **Step 1: Add failing structural and budget checks**

Extend `template_checks()` so it requires:

```rust
for banned in ["public/vendor/pico.min.css", "maud.workspace = true"] {
    if workspace_contains(banned)? {
        return Err(format!("removed dependency still present: {banned}"));
    }
}
check_size("public/assets/app.css", 24 * 1024)?;
check_size("public/assets/app.js", 8 * 1024)?;
require_text("workers/app/wrangler.jsonc", "\"assets\"")?;
require_text("public/_headers", "immutable")?;
reject_text("public/sw.js", "/notes")?;
reject_text("public/sw.js", "csrf")?;
```

- [ ] **Step 2: Run structural checks and verify RED**

Run: `cargo xtask check`

Expected: failure because Pico exists, new asset paths and headers are absent, and Wrangler has no asset configuration.

- [ ] **Step 3: Implement the starter-owned design system and assets config**

Create one `@layer reset, tokens, base, layout, components, utilities` stylesheet. Include semantic controls and the component/state selectors exercised by `/design-system`. Configure:

```json
"assets": {
  "directory": "../../public",
  "run_worker_first": false
},
"cache": {
  "enabled": true
}
```

Use `_headers` to apply the CSP/security boundary to static files and one-year immutable browser caching only to `/assets/vendor/*` and versioned icons. Leave `app.css`, `app.js`, `sw.js`, and `offline.html` on ETag revalidation.

- [ ] **Step 4: Run structural and browser syntax checks and verify GREEN**

Run: `cargo xtask check`

Run: `node --check public/assets/app.js && node --check public/sw.js && node --check public/assets/vendor/response-targets-2.0.4.js`

Expected: all commands exit zero and asset budgets are reported within limits.

- [ ] **Step 5: Commit the asset/design-system foundation**

```bash
git add public workers/app/wrangler.jsonc crates/xtask/src/main.rs
git commit -m "feat: add starter-owned product design system"
```

### Task 3: Central HTTP policy, online CSRF replay, and route modules

**Files:**
- Replace: `workers/app/src/lib.rs`
- Create: `workers/app/src/auth.rs`
- Create: `workers/app/src/http.rs`
- Create: `workers/app/src/rate_limit.rs`
- Create: `workers/app/src/routes/mod.rs`
- Create: `workers/app/src/routes/notes.rs`
- Create: `workers/app/src/routes/public.rs`
- Create: `workers/app/src/routes/system.rs`
- Create: `workers/app/src/routes/uploads.rs`
- Modify: `public/assets/app.js`

**Interfaces:**
- Produces: `CachePolicy::{Private, PublicShort, PublicDiscovery, Immutable}` and `secured(response, request_id, policy)`.
- Produces: `CsrfState`, `csrf_state`, `csrf_guard`, and `GET /session/csrf`.
- Produces: route handlers registered from the small `lib.rs` route table.

- [ ] **Step 1: Add failing pure policy and CSRF response tests**

In `http.rs` and `auth.rs`, write tests before implementations:

```rust
#[test]
fn private_policy_is_never_cacheable() {
    assert_eq!(CachePolicy::Private.browser_control(), "no-store");
    assert_eq!(CachePolicy::Private.cdn_control(), None);
}

#[test]
fn published_policy_separates_browser_and_edge_freshness() {
    assert_eq!(CachePolicy::PublicShort.browser_control(),
        "public, max-age=60, stale-while-revalidate=300");
    assert_eq!(CachePolicy::PublicShort.cdn_control(),
        Some("public, max-age=3600, stale-while-revalidate=86400"));
}
```

- [ ] **Step 2: Run the Worker Wasm check and verify RED**

Run: `cargo check -p starter-app-worker --target wasm32-unknown-unknown`

Expected: compilation fails because `CachePolicy` and split modules are not implemented.

- [ ] **Step 3: Extract adapters and route families without behavior drift**

Move existing functions by responsibility, keep route paths stable, and reduce `lib.rs` to module declarations, `#[event(fetch)]`, and router registration. Apply `CachePolicy::Private` to all owner-scoped and mutation responses. Apply public policies only inside public/system route modules.

Add `/session/csrf` returning:

```json
{"csrf_token":"<fresh-or-existing-token>"}
```

with authentication, `no-store`, and the double-submit cookie. Change offline IndexedDB operations to omit CSRF data and make replay fetch this endpoint before POSTing queued operations.

- [ ] **Step 4: Run focused tests/checks and verify GREEN**

Run: `cargo test -p starter-domain -p starter-templates`

Run: `cargo check -p starter-app-worker --target wasm32-unknown-unknown`

Run: `node --check public/assets/app.js`

Expected: all commands exit zero with no warnings.

- [ ] **Step 5: Commit the Worker boundary refactor**

```bash
git add workers/app/src public/assets/app.js
git commit -m "refactor: split Worker routes and response policies"
```

### Task 4: Public SEO/AEO discovery and cache assertions

**Files:**
- Modify: `workers/app/src/routes/public.rs`
- Modify: `workers/app/src/routes/system.rs`
- Modify: `crates/database/src/lib.rs`
- Modify: `crates/xtask/src/main.rs`
- Modify: `crates/templates/templates/pages/public_note.html`

**Interfaces:**
- Produces faithful HTML/Markdown/JSON public representations.
- Produces dynamic absolute `robots.txt`, `sitemap.xml` with `lastmod`, and `llms.txt` with Markdown/public discovery links.
- Extends the e2e harness with header and discovery assertions.

- [ ] **Step 1: Extend e2e assertions before route changes**

After publishing the smoke note, require the harness to assert:

```rust
assert_contains(get_text("/robots.txt")?, "/sitemap.xml")?;
assert_contains(get_text("/sitemap.xml")?, "<lastmod>")?;
assert_contains(get_text("/llms.txt")?, &format!("/notes/{note_id}.md"))?;
assert_header(public_html, "cache-control", "public")?;
assert_header(private_home, "cache-control", "no-store")?;
assert_contains(public_html_body, "https://")?;
```

- [ ] **Step 2: Run e2e and verify RED**

Run: `cargo xtask e2e`

Expected: failure on missing `lastmod`, dynamic `llms.txt` links, absolute metadata, or explicit cache assertions.

- [ ] **Step 3: Implement public discovery from the published model**

Generate all public surfaces from `list_published_notes`, format RFC 3339 `lastmod`, build absolute URLs from the request origin, and set explicit discovery cache policy. Never include owner IDs or drafts. Keep Markdown/JSON content faithful to the same `Note` returned by the public repository function.

- [ ] **Step 4: Run e2e and template tests and verify GREEN**

Run: `cargo test -p starter-templates`

Run: `cargo check -p starter-database --target wasm32-unknown-unknown`

Run: `cargo xtask e2e`

Expected: unit tests and the full D1/Queue/publication/discovery smoke pass.

- [ ] **Step 5: Commit SEO/AEO and cache behavior**

```bash
git add workers/app/src/routes crates/database/src/lib.rs crates/xtask/src/main.rs crates/templates/templates
git commit -m "feat: harden public discovery and cache behavior"
```

### Task 5: Generator, documentation, and agent contract

**Files:**
- Modify: `README.md`
- Modify: `AGENTS.md`
- Modify: `docs/ARCHITECTURE.md`
- Modify: `docs/COMMANDS.md`
- Modify: `docs/AUTH.md`
- Modify: `docs/SECURITY.md`
- Modify: `THIRD_PARTY_NOTICES.md`
- Modify: `.github/dependabot.yml`
- Modify: `crates/xtask/src/main.rs`
- Create: `.github/lighthouse/budgets.json`

**Interfaces:**
- Documents exact template, CSS, cache, offline, and performance contracts.
- Keeps `cargo xtask new`, `configure`, `versions`, `check`, and `smoke` aligned with the new structure.

- [ ] **Step 1: Add failing generator/documentation checks**

Require `template_checks()` and generator smoke to reject `Maud`, `Pico`, old asset paths, missing Askama version, missing `design-system`, and missing Lighthouse budgets.

- [ ] **Step 2: Run checks and verify RED**

Run: `cargo xtask check && cargo xtask smoke`

Expected: at least one command fails until documentation, generator paths, and dependency metadata are updated.

- [ ] **Step 3: Update documentation and generator contracts**

Describe Askama files, cascade layers, Static Assets, Workers Caching, public/private response policy, offline CSRF replay, performance budgets, and the component gallery. Remove Pico/Maud vendoring and version output. Add machine-readable Lighthouse resource and Core Web Vitals budgets without adding `package.json`.

- [ ] **Step 4: Run checks and generator smoke and verify GREEN**

Run: `cargo xtask check`

Run: `cargo xtask smoke`

Expected: both commands exit zero and the generated project contains the new template/assets structure with reset Cloudflare IDs.

- [ ] **Step 5: Commit docs and automation**

```bash
git add README.md AGENTS.md docs THIRD_PARTY_NOTICES.md .github crates/xtask/src/main.rs
git commit -m "docs: document the external-template starter workflow"
```

### Task 6: Full verification and visual QA

**Files:**
- Modify only files implicated by verification failures.

**Interfaces:**
- Consumes every prior task.
- Produces fresh evidence for structural, unit, Wasm, release, e2e, generator, and UI claims.

- [ ] **Step 1: Run formatting and the canonical verification suite**

Run: `cargo fmt --all -- --check`

Run: `cargo xtask verify`

Expected: formatting, Clippy, unit tests, Wasm checks, release builds, browser syntax, and structural checks pass.

- [ ] **Step 2: Run integration and generator verification**

Run: `cargo xtask e2e`

Run: `cargo xtask smoke`

Expected: local multi-Worker D1/Queue/publication smoke and generated-project smoke pass.

- [ ] **Step 3: Perform browser visual QA**

Run `cargo xtask dev`, inspect `/`, `/design-system`, a published note, and `/offline.html` at desktop and 390-pixel mobile viewports. Verify no horizontal overflow, keyboard-visible focus, responsive navigation, light/dark colors, form validation, and HTMX replacement behavior.

- [ ] **Step 4: Re-run fresh verification after any QA fix**

Run: `cargo fmt --all -- --check && cargo xtask verify && cargo xtask e2e && cargo xtask smoke`

Expected: every command exits zero after the final code change.

- [ ] **Step 5: Commit final verified fixes**

```bash
git add -A
git commit -m "test: verify starter performance and UI contracts"
```
