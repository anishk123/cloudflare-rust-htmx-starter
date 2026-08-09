# Deployment

Run `cargo xtask auth` for exact token instructions. Minimum account-level **Edit** permissions are **Workers Scripts**, **D1**, **Workers R2 Storage**, and **Queues**. Scope the token to the intended account only.

Export `CLOUDFLARE_API_TOKEN` and `CLOUDFLARE_ACCOUNT_ID`, verify with `cargo xtask whoami`, then run:

```bash
cargo xtask deploy
```

`deploy` runs local verification, creates/reuses the D1 database, R2 bucket, Queue and DLQ, writes the real D1 UUID into both Worker configs, applies remote migrations, then deploys the jobs Worker followed by the app Worker. `cargo xtask provision` and `cargo xtask migrate` remain available as lower-level commands.

Deployment tokens never belong in `.dev.vars` or Worker runtime bindings. For GitHub Actions, store the same variables as protected `production` environment secrets.

## Preview and staging safety

This starter does not auto-deploy pull requests. If you add preview/staging deployments later, provision **separate D1, R2, Queue and DLQ resources**. Never reuse production binding IDs for untrusted branches or previews.

Workers Logs are enabled in both Worker configs. Adjust Cloudflare observability sampling for production volume/cost as appropriate.
