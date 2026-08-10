# Durable contracts

Contracts make cross-boundary behavior explicit for Rust code, integrations, tests and coding agents. Put a shape in `crates/contracts` when it must survive independent producers/consumers, retries, offline storage or external use.

## What belongs here

- HTTP request/response bodies consumed outside one route adapter
- Queue messages
- offline outbox entries
- structured event payloads intended for durable processing
- public domain representations exported as JSON Schema

Internal view models, database row helpers and one-function parameters usually do not need a durable contract.

## Contract requirements

Durable shapes derive Serde and Schemars traits. Version Queue, offline and event contracts explicitly. Keep fields semantically stable within a version and use concrete units in names such as `created_at_ms`.

`GET /contracts` exports the catalog as JSON Schema so agents and integrations can inspect the same definitions the Workers compile against.

## Safe evolution workflow

1. Write or update a round-trip/schema test in `crates/contracts`.
2. Confirm the focused test fails for the missing shape or version.
3. Add the smallest compatible contract change.
4. Update every producer and consumer in the same pull request.
5. Decide how queued/offline old-version data behaves during rollout.
6. Run package tests, Wasm checks and the relevant e2e flow.
7. Update `/contracts` consumers and documentation when externally visible.

## Queue contract checklist

- [ ] The message includes `contract_version`, a stable job ID and entity IDs/references.
- [ ] It does not carry a large artifact that belongs in R2 or D1.
- [ ] The consumer rejects or deliberately handles unsupported versions.
- [ ] Duplicate delivery produces the same business result.
- [ ] Producer and consumer are compatible during deployment order.
- [ ] Tests cover serialization round trip and the idempotent mutation path.

Deploy the jobs Worker before the app Worker when a new producer message requires new consumer behavior. The canonical `cargo xtask deploy` preserves that order.

## Offline contract checklist

- [ ] Only intentionally offline-capable data is stored.
- [ ] A stable operation ID survives retries.
- [ ] CSRF tokens, session credentials and unrelated private page data are absent.
- [ ] Replay obtains current authentication/CSRF state online.
- [ ] Conflict and failed-4xx behavior are visible and deterministic.
- [ ] Old stored versions have a migration, compatibility path or explicit rejection state.

## Compatibility guidance

Adding an optional field with a safe default can often remain compatible. Renaming, changing meaning/unit, tightening validation or changing an enum can break old producers or stored messages even when Rust compiles.

For incompatible semantics, introduce a new version and keep the old consumer path long enough for queued/offline data to drain or migrate. Do not silently reinterpret old bytes as new meaning.

## Verification

```bash
cargo test -p starter-contracts
cargo check -p starter-app-worker --target wasm32-unknown-unknown
cargo xtask e2e
```

Use the tests appropriate to the changed boundary, then finish with `cargo xtask verify`.

## Next step

Read [Extending](EXTENDING.md) for the complete feature sequence or [Architecture](ARCHITECTURE.md) for dependency direction.
