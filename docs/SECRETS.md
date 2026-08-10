# Secrets

Keep infrastructure credentials and application runtime secrets separate. They have different consumers, permissions and rotation paths.

## Secret classes

| Class | Examples | Used by | Never place in |
|---|---|---|---|
| Cloudflare control plane | `CLOUDFLARE_API_TOKEN`, `CLOUDFLARE_ACCOUNT_ID` | Wrangler, `xtask`, protected CI environment | Worker vars/bindings, `.dev.vars`, source code |
| Application runtime | API keys, OAuth client secrets, signing/encryption keys | app/jobs Worker at runtime | source code, logs, plain CI output |
| Local development runtime | local-only service key used by a feature | ignored Worker `.dev.vars` | committed files or shared examples with live values |

The local `DEV_USER_ID` is a development placeholder, not an authentication secret. It must not be enabled in production.

## Local placement

When a Worker feature needs a local runtime value, place it in an ignored file next to that Worker’s Wrangler config, for example `workers/app/.dev.vars`:

```dotenv
THIRD_PARTY_API_KEY=local-development-value
```

Use obvious non-secret placeholders in documentation. Before committing, check `git status` and never stage `.dev.vars`, `.cloudflare.env`, local state or credential exports.

## Deployed placement

Store Worker runtime secrets with Cloudflare Secrets:

```bash
wrangler secret put THIRD_PARTY_API_KEY -c workers/app/wrangler.jsonc
```

Set jobs Worker secrets separately when needed. A secret available to one Worker is not automatically available to the other.

Store `CLOUDFLARE_API_TOKEN` and `CLOUDFLARE_ACCOUNT_ID` in the protected GitHub `production` environment for the manual Deploy workflow. Scope the API token to the intended account and only the required Workers Scripts, D1, R2 and Queues permissions.

## Rotation and incident response

1. Revoke or rotate the credential at its provider immediately.
2. Update the correct local, Cloudflare Secret or protected CI location.
3. Redeploy only the Worker that consumes a changed runtime secret.
4. Review logs and provider audit history without printing the secret.
5. Remove leaked material from history if appropriate, but never treat history rewriting as rotation.

Prefer independently rotatable credentials with the narrowest useful scope. Never log secret values, full authorization headers or session cookies.

## Next step

Read [Deployment](DEPLOYMENT.md) before using control-plane credentials and [Authentication](AUTH.md) before adding session or OAuth secrets.
