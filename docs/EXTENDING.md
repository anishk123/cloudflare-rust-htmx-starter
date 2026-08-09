# Extending the starter

For each capability: define/extend a contract → write a failing domain/contract test → implement domain rule → add prepared D1 query if persistence is needed → add Maud page/fragment with progressive enhancement → add Queue work only for genuinely asynchronous operations → define offline semantics only when conflicts/replay are safe → run `cargo xtask verify`.
