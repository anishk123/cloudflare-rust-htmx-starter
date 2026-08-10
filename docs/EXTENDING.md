# Extending the starter

For each capability: define/extend a contract → write a failing domain/contract test → implement the domain rule → add a prepared D1 query if persistence is needed → add an Askama page/fragment with progressive enhancement → select the central cache policy → add Queue work only for genuinely asynchronous operations → define offline semantics only when conflicts/replay are safe → run `cargo xtask verify`.

Edit markup in `crates/templates/templates/` and add only the data it needs to the focused Rust view model. Reuse tokens and components from `public/assets/app.css`; add new reusable states to `/design-system`. Route handlers should coordinate adapters, not contain HTML, large SQL strings, or browser assets.
