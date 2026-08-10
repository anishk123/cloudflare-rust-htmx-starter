# Changelog

## 0.3.0 — 2026-08-10
- Moved page and fragment markup into compile-checked external Askama templates.
- Replaced the external semantic CSS layer with a starter-owned responsive product design system and live component gallery.
- Split the application Worker into focused auth, response-policy, rate-limit and route modules.
- Hardened CSRF, public/negative caching, canonical publication, SEO/AEO discovery and offline replay boundaries.
- Added enforced CSS/JavaScript budgets, generator branding checks and progressive human/agent onboarding.

## 0.2.0 — 2026-08-08
- Reduced repository automation to Rust `xtask`; removed Python/package.json application tooling.
- Introduced the first semantic UI layer and response-targets integration.
- Added native-HTML-first, progressive-enhancement, CSP and HTMX lifecycle rules.
- Consolidated local/provision/deploy/generator commands behind `cargo xtask`.
- Hardened CI/security and human/agent documentation.
