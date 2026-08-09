# Third-party browser assets

The starter pins browser dependencies and serves them from the Worker origin rather than loading a third-party CDN at runtime. `cargo xtask dev`/`cargo xtask vendor` refresh the exact pinned artifacts before build/development.

- HTMX 2.0.10 — Zero-Clause BSD — https://github.com/bigskysoftware/htmx
- HTMX response-targets 2.0.4 — official HTMX extension — https://htmx.org/extensions/response-targets/
- Pico CSS 2.1.1 — MIT — https://github.com/picocss/pico

Before redistributing a modified third-party asset, preserve the upstream copyright/license notices supplied with that asset/package.
