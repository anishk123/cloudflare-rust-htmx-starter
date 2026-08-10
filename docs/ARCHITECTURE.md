# Architecture

Cloudflare Static Assets serves `public/` before the application Worker. Versioned vendor files are immutable; starter CSS/JavaScript and the token-free offline shell revalidate. The service worker caches only that public shell and never workspace pages, authenticated responses, CSRF data, or note routes.

The Rust/Wasm app Worker renders complete HTML and HTMX fragments and owns D1/R2/Queue-producer access. `src/lib.rs` is only the route table. Authentication/CSRF, response policy, rate limiting, and each route family live in focused modules. `CachePolicy` is the single dynamic-response cache boundary: owner-scoped pages and every mutation are `no-store`; only explicitly published records and public discovery documents are cacheable.

`crates/templates/templates/` contains ordinary Askama HTML files. Rust view models in `crates/templates/src/` prepare URLs, status flags, descriptions, dates, and safe JSON-LD before rendering. Route modules select data and policies; templates own markup; domain/database crates remain independent of presentation.

One separate Rust/Wasm jobs Worker consumes Queues because asynchronous retry semantics are distinct. Browser state stays deliberately tiny: HTMX + response-targets, the starter-owned layered design system, lifecycle-safe `app.js`, IndexedDB outbox, and service worker. The app/API boundary is not split because HTMX already makes HTTP plus HTML the interface.
