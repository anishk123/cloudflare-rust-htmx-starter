# Local development

Use `cargo xtask bootstrap` once and `cargo xtask dev` day-to-day. Local D1, R2 and Queues persist under `.wrangler/state`. No Cloudflare account is required. Delete `.wrangler/state` to reset. Wrangler/Node is tooling only; application code stays Rust + HTML/CSS/JS assets.
