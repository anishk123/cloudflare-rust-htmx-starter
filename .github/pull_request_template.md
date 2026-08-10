## Purpose and impact

What problem does this solve, why does it belong at this boundary, and what changes for users or contributors?

## What changed

- Describe the focused implementation changes.

## Verification

List the exact commands that actually ran and their results. Do not check an unexecuted command.

- [ ] `cargo xtask verify`
- [ ] `cargo xtask e2e` when HTTP, D1, Queue, publication or offline behavior changed
- [ ] `cargo xtask smoke` when generator, paths, branding or onboarding changed
- [ ] Local browser flow checked with `cargo xtask dev` when visible behavior changed

## Boundary review

- [ ] New or changed durable contracts are versioned and tested
- [ ] Public/private publication, cache, CSRF and PWA/offline boundaries were reviewed
- [ ] Prepared SQL and idempotent Queue/replay rules are preserved
- [ ] Documentation and `/design-system` were updated when relevant
- [ ] No secrets, generated `build/` output, local Wrangler state or unrelated files are committed

## Notes, migrations and risks

Call out migration order, compatibility, deferred work or operational follow-up. Write “None” when there is nothing to report.
