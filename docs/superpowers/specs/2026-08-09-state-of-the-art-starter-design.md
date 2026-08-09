# State-of-the-Art Starter Design

## Objective

Turn the repository into a polished, measurable Rust/HTMX application foundation that is pleasant to modify, fast on first and repeat visits, secure by default, and legible to both humans and coding agents.

The work is a focused starter-foundation refactor. It does not add product-specific authentication, payments, a frontend framework, npm application tooling, or a general-purpose component runtime.

## Product principles

1. Rust owns HTTP, domain behavior, durable contracts, persistence, jobs, and typed template context.
2. HTML remains server-rendered, semantic, and progressively enhanced.
3. Templates are recognizable HTML files, not a Rust-embedded HTML DSL.
4. The browser receives only HTML, starter-owned CSS, vendored HTMX extensions, and small lifecycle-safe JavaScript.
5. Static delivery and public content use Cloudflare's current cache primitives; authenticated and workspace responses remain `no-store`.
6. Performance, accessibility, SEO, AEO, and asset sizes are enforced by repeatable checks rather than README claims.
7. Files are organized around one responsibility and expose small interfaces.

## Approved visual direction

The default is **Quiet Product**: a neutral, polished application shell with a restrained blue accent, clear hierarchy, comfortable density, and automatic light/dark themes. It must suit CRUD products, small SaaS applications, and internal tools without pretending to be a product-specific brand.

Pico CSS is removed and is not replaced by another CSS framework. Bulma and Bootstrap introduce class systems broader than the starter needs. Web-component systems introduce additional JavaScript and non-native controls. Token-only libraries do not remove enough work to justify another dependency.

## Template architecture

Replace Maud with pinned Askama templates. Askama preserves compile-time template parsing, typed Rust contexts, automatic HTML escaping, and compiled output while moving markup into ordinary `.html` files.

`crates/templates` is split into focused units:

- `src/lib.rs`: public rendering API and error type exports.
- `src/models.rs`: template-only view models such as `PageMeta`, `NoteView`, and navigation state.
- `src/pages.rs`: Askama page template structs and page rendering functions.
- `src/fragments.rs`: HTMX fragment structs and rendering functions.
- `src/seo.rs`: safe JSON-LD construction and date formatting.
- `templates/layouts/base.html`: document shell, metadata, application chrome, and external assets.
- `templates/pages/*.html`: home, public note, design-system gallery, and error pages.
- `templates/fragments/*.html`: note cards, validation errors, and upload results.

The Worker passes explicit absolute canonical URLs and view models. Template code does not query databases, inspect requests, or make authorization decisions.

All render functions return `Result<String, askama::Error>`. Worker route helpers translate template errors to `worker::Error` in one place. Tests assert escaping, progressive-enhancement attributes, canonical metadata, JSON-LD safety, and public/draft boundaries.

## CSS architecture

`public/assets/app.css` is one readable, unminified, cacheable file. Avoiding CSS imports prevents a render-blocking request waterfall and avoiding a compiler preserves the no-build-tool contract.

The file uses cascade layers and named sections:

1. `reset`: box sizing, inherited fonts, media defaults, and native control normalization.
2. `tokens`: color, typography, spacing, size, radius, shadow, motion, and z-index variables.
3. `base`: semantic document, headings, links, lists, tables, forms, and focus behavior.
4. `layout`: container, stack, cluster, grid, app shell, sidebar, header, and responsive rules.
5. `components`: buttons, cards, badges, alerts, field groups, toolbars, empty states, toasts, navigation, pagination, dialogs, details, and progress.
6. `utilities`: visually hidden, mobile/desktop visibility, and bounded helpers only.

Native HTML remains the behavior layer. CSS must cover hover, focus-visible, active, invalid, disabled, busy, empty, success, warning, and destructive states. Controls meet 44-pixel coarse-pointer targets. Motion respects `prefers-reduced-motion`; colors support `prefers-color-scheme`; layouts support safe-area insets.

The uncompressed starter-owned CSS budget is 24 KiB. The existing `xtask check` command enforces the budget and ensures Pico is absent.

## Static assets and browser caching

Configure the app Worker with Cloudflare Static Assets using `public/` as the asset directory and asset-first routing. Static files are served without executing the Rust Worker and receive Cloudflare's distributed asset cache and strong content ETags.

Use these paths:

- `/assets/app.css`
- `/assets/app.js`
- `/assets/vendor/htmx-2.0.10.min.js`
- `/assets/vendor/response-targets-2.0.4.js`
- `/manifest.webmanifest`
- `/sw.js`
- versioned icons
- `/offline.html`

`public/_headers` supplies security headers for static responses. Versioned vendor assets receive `Cache-Control: public, max-age=31556952, immutable`. Starter-owned files retain Cloudflare's revalidation behavior so clones can change them without manually updating a fingerprint. The service worker remains an app-shell cache, but it never stores authenticated HTML, workspace records, or CSRF tokens.

The Rust Worker no longer embeds or routes CSS, JavaScript, the manifest, icons, or service-worker files. This reduces Worker Wasm size and removes duplicated MIME/cache logic.

## Offline and CSRF flow

`/offline.html` is a static, non-sensitive capture page. Its form stores only the versioned operation ID, title, body, and creation time in IndexedDB.

When connectivity returns:

1. `app.js` requests `GET /session/csrf` using current credentials.
2. The Worker authenticates the request, returns a fresh CSRF token, and sets the double-submit cookie when necessary.
3. The browser replays queued operations with that token and the stable operation ID.
4. Successful operations are removed. Retryable failures remain queued. A terminal 4xx records an actionable sync error without discarding user data.

This keeps authentication and CSRF online-only while preserving offline capture. The service worker falls back to `/offline.html` only for failed navigation requests.

## Public response caching

Enable Workers Caching for the app Worker. The existing response boundary remains authoritative:

- authenticated pages, CSRF responses, health data, mutations, uploads, and workspace data: `Cache-Control: no-store`;
- explicitly published HTML, Markdown, and JSON: short browser freshness plus longer Cloudflare freshness and stale-while-revalidate;
- sitemap, robots, `llms.txt`, contract catalog, and design-system gallery: explicit public policies;
- responses with cookies are never public-cacheable.

Public cache keys use canonical URLs and never vary on user identity. The starter will not cache draft or owner-scoped data. Cache policy functions are unit-tested as pure policy decisions before being applied to Worker responses.

## SEO and AEO

Every indexable page receives:

- an absolute canonical URL;
- a unique title and meta description;
- semantic landmarks and one clear `h1`;
- Open Graph metadata;
- JSON-LD with escaped script content and RFC 3339 dates;
- faithful HTML, Markdown, and JSON representations from the same public model.

`sitemap.xml` includes published canonical URLs and `lastmod`. `robots.txt` references the absolute sitemap. `llms.txt` describes the application and links to the contract catalog, sitemap, and published Markdown representations instead of returning generic static copy. Draft records remain absent from every public surface.

The design does not promise crawler-specific content or treat `llms.txt` as a substitute for semantic HTML.

## Worker code organization

Split the oversized app Worker module without changing its single-Worker boundary:

- `src/lib.rs`: module declarations, event entrypoint, and route table only.
- `src/auth.rs`: current-user resolution and CSRF state/validation.
- `src/http.rs`: request IDs, response construction, security headers, and cache policies.
- `src/rate_limit.rs`: Cloudflare rate-limit adapter.
- `src/routes/notes.rs`: home, create, summarize, and publish handlers.
- `src/routes/public.rs`: published HTML/data plus SEO/AEO discovery handlers.
- `src/routes/uploads.rs`: upload validation, quota, and R2 storage.
- `src/routes/system.rs`: health, contracts, design system, and CSRF endpoint.

Route modules depend on contracts, domain services, repositories, template renderers, and the small HTTP/auth adapters. They do not duplicate response/security policy.

## Error handling and observability

Preserve stable `ApiError` contracts and request IDs. HTML requests receive accessible error pages or targeted HTMX fragments; JSON endpoints receive structured errors. Template failures, D1 failures, queue failures, and invalid user input remain distinguishable. Logs contain correlation IDs and event names but no CSRF tokens, note bodies, uploaded bytes, credentials, or secrets.

Expected user failures use appropriate 4xx responses. Unexpected internal failures are not converted into misleading validation errors.

## Developer experience

`cargo xtask dev` remains the only development entrypoint. Template and CSS edits are detected by Wrangler's development process; Rust context changes rebuild the Worker. `cargo xtask new` and `configure` update new metadata and asset paths. `cargo xtask versions` reports Askama and no longer reports Maud or Pico.

Add `/design-system` as living documentation for starter components and states. README and architecture docs explain where to change HTML, tokens, components, cache policies, and public metadata. `AGENTS.md` retains the contract-first/test-first workflow and explicitly directs agents to external templates and cascade-layer boundaries.

## Testing and performance gates

Development follows contract/test first and red-green-refactor. Verification includes:

- native unit tests for view models, metadata, escaping, cache policies, validation, and domain behavior;
- template tests for progressive enhancement and public metadata;
- `xtask check` structural rules for no Pico/Maud, no inline handlers/scripts, no private service-worker entries, expected template layout, and asset budgets;
- Wasm checks and release builds;
- end-to-end D1/Queue/publication smoke including HTML, Markdown, JSON, sitemap, `llms.txt`, and cache headers;
- browser syntax checks;
- generator smoke tests;
- a Lighthouse CI configuration for deployed or explicitly started local previews with mobile budgets, without adding an npm application dependency tree.

Performance acceptance budgets:

- starter-owned CSS at or below 24 KiB uncompressed;
- starter-owned JavaScript at or below 8 KiB uncompressed;
- no render-blocking JavaScript;
- no web fonts or runtime CDN requests;
- public HTML usable without JavaScript;
- target field thresholds: LCP at or below 2.5 seconds, INP at or below 200 milliseconds, and CLS at or below 0.1 at the 75th percentile;
- aspirational internal targets: LCP at or below 1.8 seconds, INP at or below 100 milliseconds, and CLS at or below 0.05.

Local timings and synthetic tests are evidence for regressions, not claims about global field performance. Production claims require real-user or deployed synthetic measurements.

## Migration and compatibility

Existing D1 migrations and durable contracts remain compatible. Public URLs remain stable. The home, note mutation, upload, contract, health, published HTML, Markdown, JSON, sitemap, robots, and manifest behaviors remain available.

Template and asset internals may change. Generated projects receive the new structure. No deploy is performed as part of this work.

## Completion criteria

The work is complete when:

1. Maud and Pico are absent from code, vendoring, documentation, and dependency checks.
2. Askama external templates render all pages and HTMX fragments.
3. Quiet Product CSS and `/design-system` cover the documented states.
4. Static assets bypass Rust and use Cloudflare static-asset caching.
5. Public content has explicit safe cache policies; private content remains `no-store`.
6. Offline fallback contains no private data or CSRF token and replay obtains a fresh online token.
7. SEO/AEO metadata is absolute, faithful, and date-correct.
8. Worker routing, response policy, auth, and templates are split into focused files.
9. Structural, unit, Wasm, end-to-end, generator, and browser checks pass.
10. Documentation describes the resulting architecture accurately.
