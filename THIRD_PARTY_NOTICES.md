# Third-party browser assets

The starter pins browser dependencies and serves them from the application origin rather than loading a third-party CDN at runtime. `cargo xtask dev` and `cargo xtask vendor` refresh the exact pinned artifacts; see the [command reference](docs/COMMANDS.md).

- HTMX 2.0.10 — Zero-Clause BSD — https://github.com/bigskysoftware/htmx
- HTMX response-targets 2.0.4 — official HTMX extension — https://htmx.org/extensions/response-targets/

`public/assets/app.css`, `public/assets/app.js`, and `public/sw.js` are starter-owned source files, not vendored frameworks.

Before redistributing a modified third-party asset, preserve the upstream copyright/license notices supplied with that asset/package.
