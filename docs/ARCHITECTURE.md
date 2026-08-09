# Architecture

One Rust/Wasm app Worker renders complete HTML and HTMX fragments and owns D1/R2/Queue-producer access. One separate Rust/Wasm jobs Worker consumes Queues. Browser state is deliberately tiny: HTMX + response-targets, Pico CSS, app.css, app.js and service worker. The app/API boundary is not split because HTMX already makes HTTP+HTML the interface; jobs remain separate because asynchronous retry semantics are distinct.
