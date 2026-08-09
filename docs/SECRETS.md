# Secrets

There are two separate secret classes:

1. **Cloudflare control-plane credentials** — `CLOUDFLARE_API_TOKEN` and `CLOUDFLARE_ACCOUNT_ID`. These are used by Wrangler/CI only and never belong in Worker runtime variables.
2. **Application runtime secrets** — third-party API keys, OAuth client secrets, signing keys. Store deployed values with Cloudflare secrets, e.g. `wrangler secret put NAME -c workers/app/wrangler.jsonc`. Put local-only runtime values in ignored `.dev.vars` only when a feature needs them.

Never commit either class. Never log secret values. Prefer narrowly scoped, independently rotatable credentials.
