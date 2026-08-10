# Deployment

Deployment is deliberate and remote-mutating. Complete local development and verification first; ordinary pushes and pull requests never deploy Cloudflare resources.

## Prerequisites

- a Cloudflare account and intended account ID;
- a scoped API token or interactive Wrangler login;
- local `cargo xtask verify` success;
- reviewed D1 migrations and environment-specific resource plan;
- real authentication before storing user/tenant data.

Run `cargo xtask auth` for the current token instructions. A custom token needs account-level **Edit** permissions for **Workers Scripts**, **D1**, **Workers R2 Storage**, and **Queues**. Scope it to only the intended account.

## 1. Verify credentials

```bash
export CLOUDFLARE_API_TOKEN="..."
export CLOUDFLARE_ACCOUNT_ID="..."

cargo xtask whoami
```

These are control-plane credentials. Never put them in `.dev.vars`, Worker vars or runtime bindings. See [Secrets](SECRETS.md).

## 2. Verify locally

```bash
cargo fmt --all -- --check
cargo xtask verify
cargo xtask e2e
cargo xtask smoke
```

Review the exact migration files, authentication boundary, public routes, cache policies and service-worker scope for the release.

## 3. Provision or inspect resources

```bash
cargo xtask provision
```

Provision creates or reuses:

```text
<app>-db
<app>-attachments
<app>-jobs
<app>-jobs-dlq
```

It writes the real D1 UUID into both Worker configs. Review the resulting diff before committing configuration changes.

## 4. Apply migrations and deploy

The high-level command is:

```bash
cargo xtask deploy
```

It reruns verification, provisions/reuses resources, applies remote migrations, deploys the jobs Worker first, then the app Worker. Consumer-first order keeps newly produced Queue messages compatible.

For deliberate lower-level operation:

```bash
cargo xtask migrate
```

Do not apply remote migrations merely to test a local idea.

## 5. Post-deploy checks

- Load `/healthz`, the public app shell and authenticated workspace flow.
- Create/process/publish a non-sensitive test record in the intended environment.
- Confirm published canonical, Markdown, JSON, sitemap, robots and `llms.txt` behavior.
- Confirm private/mutation/error responses remain `no-store`.
- Review Worker logs for request/job correlation without sensitive payloads.
- Measure actual latency and Core Web Vitals from the deployed region/device mix.

## Rollback and migration limits

Worker code can be rolled back independently, but D1 schema/data changes may not be reversible. Prefer additive compatible migrations, deploy compatible consumers before producers, and define a forward-fix or explicit rollback migration before destructive changes.

R2 objects and queued messages outlive a code rollback. Keep contract versions and old consumer compatibility long enough for in-flight work to drain.

## Preview and staging safety

The starter does not auto-deploy pull requests. If you add preview/staging later, create separate D1, R2, Queue and DLQ resources with separate bindings and secrets. Never point an untrusted branch or preview Worker at production data.

GitHub’s manual Deploy workflow uses a protected `production` environment. Store `CLOUDFLARE_API_TOKEN` and `CLOUDFLARE_ACCOUNT_ID` there; require environment reviewers when appropriate.

## Observability

Workers Logs are enabled in both configs. Tune sampling for production volume/cost rather than removing structured request/job events. Never log secret values, authorization headers, session cookies or private content.

## Next step

Review [Security](SECURITY.md) and [Authentication](AUTH.md) before the first real-data deployment. Use the [command reference](COMMANDS.md) when you need lower-level control.
