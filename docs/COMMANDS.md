# Command reference

Rust `xtask` is the canonical interface for humans, coding agents and CI. Run `cargo xtask help` for the source-of-truth summary and `cargo xtask versions` for pinned versions.

## Start

| Command | Scope | Purpose |
|---|---|---|
| `cargo xtask bootstrap` | local + network when tools/assets are missing | Check prerequisites, install the Wasm target and Worker builder, then vendor browser assets |
| `cargo xtask vendor` | local + network | Refresh only pinned HTMX and response-targets files under `public/assets/vendor/` |
| `cargo xtask dev` | local | Apply local migrations and run app/jobs Workers with persistent D1, R2 and Queue state |
| `cargo xtask migrate-local` | local mutation | Apply committed D1 migrations to persistent local state without starting Workers |

Starter-owned CSS and JavaScript are source code; vendoring never overwrites them.

## Build and verify

| Command | Scope | Purpose |
|---|---|---|
| `cargo xtask test` | local | Run native Rust tests plus static browser/template checks |
| `cargo xtask check` | local | Enforce structural, security, asset-budget and active-documentation contracts |
| `cargo xtask e2e` | local | Start temporary Workers/state and exercise CSRF, D1, Queue, publication, caching and discovery |
| `cargo xtask smoke` | local | Generate a branded sample application and verify its repository/onboarding contract |
| `cargo xtask verify` | local + network if tooling is absent | Run formatting, Clippy, tests, Wasm checks, release builds, structural checks, smoke and e2e |

Use focused tests during development. Use `verify` before requesting review and report any command you genuinely could not run.

## Create or customize

Create a sibling application:

```bash
cargo xtask new my-product \
  --title "My Product" \
  --dir ../my-product \
  --github my-org/my-product
```

Rename/configure the current clone:

```bash
cargo xtask configure my-product \
  --title "My Product" \
  --github my-org/my-product
```

`new` excludes Git history, credentials, build/local state and Cloudflare resource IDs. Both commands rewrite visible branding, application/resource names, the PWA manifest and CI badge repository.

## Cloudflare operations

These commands can read or mutate remote Cloudflare state. They require `CLOUDFLARE_API_TOKEN` and `CLOUDFLARE_ACCOUNT_ID` except `auth`, which only prints guidance.

| Command | Remote effect |
|---|---|
| `cargo xtask auth` | None; show exact scoped-token guidance |
| `cargo xtask whoami` | Read account identity to verify credentials |
| `cargo xtask provision` | Create or reuse D1, R2, Queue and DLQ resources; write D1 IDs into Worker configs |
| `cargo xtask migrate` | Apply committed migrations to remote D1 |
| `cargo xtask deploy` | Verify, provision/reuse, migrate, then deploy jobs Worker followed by app Worker |

Never run a remote mutation merely to test local work. Read [Deployment](DEPLOYMENT.md) and [Secrets](SECRETS.md) first.

## Discover versions

```bash
cargo xtask versions
```

This prints the pinned workers-rs/Worker builder, Askama, HTMX, response-targets, Wrangler and minimum Rust versions without querying a remote service.

## Next step

Use [Local development](LOCAL_DEVELOPMENT.md) for the daily loop, [Extending](EXTENDING.md) for feature sequencing, or [Deployment](DEPLOYMENT.md) before remote operations.
