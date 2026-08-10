# Local development

Use this guide to reach a reliable local run, understand what persists, and recover from common setup failures. You do not need a Cloudflare account for local product development.

## Prerequisites

- Rust 1.97.1 or newer through [rustup](https://rustup.rs/)
- Node.js 22 or newer for the pinned Wrangler tooling
- Git

There is no npm application dependency tree. Rust owns application and developer automation; Node only launches Cloudflare’s local runtime.

## First run

```bash
cargo xtask dev
```

The command checks or installs required tools, adds the `wasm32-unknown-unknown` Rust target, vendors pinned HTMX assets, applies local D1 migrations, builds both Workers, and starts the local app at `http://localhost:8787`.

Useful pages:

- `/` — Evidence Notes workspace
- `/design-system` — live tokens, components and interaction states
- `/offline.html` — static credential-free fallback
- `/healthz` — local health response
- `/contracts` — generated durable-contract catalog

## What persists locally

Wrangler stores local D1, R2 and Queue state under `.wrangler/state`. Stop and restart `cargo xtask dev` without losing data.

To apply migrations without starting the servers:

```bash
cargo xtask migrate-local
```

To reset all local Cloudflare data, stop the dev process and remove only the repository’s local state:

```bash
rm -rf .wrangler/state
cargo xtask dev
```

This reset is destructive to local-only data but does not touch remote Cloudflare resources.

## Development identity

The example is intentionally unauthenticated but database access is owner-scoped. Local development uses the `DEV_USER_ID` placeholder documented in [Authentication](AUTH.md). It is not a credential and must never be enabled in production.

When you need an explicit local identity, create the ignored file `workers/app/.dev.vars`:

```dotenv
DEV_USER_ID=11111111-1111-4111-8111-111111111111
```

Do not place `CLOUDFLARE_API_TOKEN` in this file. See [Secrets](SECRETS.md).

## Daily loop

```bash
cargo xtask check   # quick structural baseline
cargo xtask dev     # work against local D1/R2/Queues
cargo xtask test    # focused native checks
cargo xtask verify  # complete gate before review
```

Run `cargo xtask e2e` when HTTP, D1, Queue or publication behavior changes. Run `cargo xtask smoke` when generator, branding, paths or onboarding change.

## Common setup problems

### Port 8787 is already in use

Stop the existing local server or identify it with your operating system’s port tools. Do not kill an unfamiliar process automatically. `cargo xtask e2e` uses an isolated temporary port and state directory.

### Rust target or Worker build is missing

Run:

```bash
cargo xtask bootstrap
```

Then retry `cargo xtask dev`. Bootstrap reports the missing prerequisite instead of hiding it.

### Wrangler cannot be downloaded

Confirm Node 22+ and network access. The project invokes pinned Wrangler `4.120.0` through `npx` when the matching binary is not already installed. Application code still has no Node runtime dependency.

### Local migrations fail after a schema experiment

If the data is disposable, reset `.wrangler/state` as shown above. If it matters, fix the migration sequence; do not manually mutate state and assume production will behave the same way.

### The browser shows an older service worker

Use the in-app “Update available · reload” control. The starter deliberately does not replace an active editing session behind the user’s back.

## Next step

Continue with [Extending the starter](EXTENDING.md) for the file-by-file feature workflow, or use the [command reference](COMMANDS.md) to understand every automation command.
