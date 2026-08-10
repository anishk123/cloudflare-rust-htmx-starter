# Progressive Documentation and Onboarding Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Give first-time humans and coding agents a fast successful start, progressive learning paths, and one consistent contribution workflow across every active documentation surface.

**Architecture:** `README.md` becomes the welcoming orientation layer, `AGENTS.md` remains the normative agent contract, `CONTRIBUTING.md` defines the shared human/agent contribution loop, and focused `docs/` guides provide progressively deeper reference material. Rust `xtask` structural checks enforce only durable terminology, path, and navigation contracts; historical decision records remain unchanged except for explicit context when needed.

**Tech Stack:** GitHub-flavored Markdown, Rust 1.97.1 `xtask` structural checks, Git/GitHub CLI, GitHub Actions.

## Global Constraints

- Optimize the primary path for developers who may be new to Rust, Cloudflare Workers, HTMX, or one of the three.
- Serve coding agents through one authoritative `AGENTS.md` contract and deterministic `cargo xtask` commands.
- Use progressive disclosure: first run and first edit before deployment, security, and operations depth.
- Keep claims measurable and honest; do not promise universal Core Web Vitals or latency.
- Preserve historical specs/plans as decision records rather than rewriting their chronology.
- Do not add a documentation framework, package manager, screenshot pipeline, or external runtime dependency.
- Do not deploy Cloudflare resources.
- Push the final branch, open a draft pull request, and wait for required GitHub Actions checks to pass.

---

### Task 1: Add durable documentation consistency guardrails

**Files:**
- Modify: `crates/xtask/src/main.rs`
- Test: `cargo xtask check`

**Interfaces:**
- Consumes: active Markdown paths tracked by the starter.
- Produces: structural failures for obsolete framework/asset references and missing onboarding/contribution navigation.
- Excludes: `docs/superpowers/` historical decision records from current-terminology enforcement.

- [ ] **Step 1: Expand the active documentation check and verify RED**

Replace the partial documentation-path loop in `template_check()` with the complete active set:

```rust
let active_docs = [
    "README.md",
    "AGENTS.md",
    "CONTRIBUTING.md",
    "CHANGELOG.md",
    "CLAUDE.md",
    "CODEX.md",
    "OMNIAGENT.md",
    "SECURITY.md",
    "THIRD_PARTY_NOTICES.md",
    ".github/pull_request_template.md",
    "docs/ARCHITECTURE.md",
    "docs/AUTH.md",
    "docs/COMMANDS.md",
    "docs/CONTRACTS.md",
    "docs/DEPLOYMENT.md",
    "docs/EXTENDING.md",
    "docs/LOCAL_DEVELOPMENT.md",
    "docs/SECRETS.md",
    "docs/SECURITY.md",
];
```

Keep the existing banned terms and add the old stylesheet path:

```rust
for banned in [
    "Maud",
    "maud",
    "Pico CSS",
    "pico.min.css",
    "public/app.css",
    "public/app.js",
    "public/vendor/",
] {
    if source.contains(banned) {
        return Err(format!(
            "obsolete starter reference {banned:?} remains in {path}"
        ));
    }
}
```

Run: `cargo xtask check`

Expected: FAIL because `CHANGELOG.md` still describes Pico CSS as the current UI layer.

- [ ] **Step 2: Add high-signal navigation requirements**

Add exact, non-prose-sensitive requirements:

```rust
for (path, needles) in [
    (
        "README.md",
        &["Five-minute start", "Choose your path", "CONTRIBUTING.md"][..],
    ),
    (
        "AGENTS.md",
        &["## Documentation rules", "## Completion"][..],
    ),
    (
        "CONTRIBUTING.md",
        &["## First contribution", "## Pull request checklist"][..],
    ),
] {
    let source = fs::read_to_string(path).map_err(|error| error.to_string())?;
    for needle in needles {
        if !source.contains(needle) {
            return Err(format!("{path} missing documentation contract: {needle}"));
        }
    }
}
```

Run: `cargo xtask check`

Expected: FAIL on the first missing navigation contract.

- [ ] **Step 3: Format and inspect the guardrail diff**

Run: `cargo fmt --all`

Run: `git diff --check`

Expected: both commands exit zero; only `crates/xtask/src/main.rs` changes.

- [ ] **Step 4: Leave the check RED for the documentation implementation**

Do not weaken the expected terms. Record the exact first failure so Task 2 proves the user-facing implementation satisfies the guardrail.

### Task 2: Rebuild the repository front door and contribution journey

**Files:**
- Modify: `README.md`
- Modify: `AGENTS.md`
- Modify: `CONTRIBUTING.md`
- Modify: `CHANGELOG.md`
- Modify: `CLAUDE.md`
- Modify: `CODEX.md`
- Modify: `OMNIAGENT.md`
- Modify: `.github/pull_request_template.md`
- Modify: `SECURITY.md`
- Modify: `THIRD_PARTY_NOTICES.md`
- Modify: `crates/xtask/src/main.rs` only if a guardrail is factually incorrect
- Test: `cargo xtask check`

**Interfaces:**
- Produces: a five-minute first run, a “choose your path” navigation layer, a demo-to-product map, and one shared contribution loop.
- Consumes: canonical commands and architecture boundaries from `AGENTS.md` and the focused guides.

- [ ] **Step 1: Rewrite the README opening for immediate orientation**

Keep the truthful status/version badges. Make the opening sequence:

```markdown
# Rust + HTMX Cloudflare Starter

Build polished, progressively enhanced Cloudflare applications in Rust—without a frontend build pipeline.

## What you get

| Need | Included foundation |
|---|---|
| Fast first render | Server-rendered Askama HTML, no hydration, tiny enforced CSS/JS budgets |
| Product-ready UI | Starter-owned responsive design system, dark mode, accessible native controls |
| Durable data | D1 repositories with prepared SQL and versioned contracts |
| Background work | Idempotent Cloudflare Queue consumer |
| Public discovery | Canonical HTML, Markdown, JSON, JSON-LD, sitemap, robots and `llms.txt` |
| Offline resilience | Conservative shell cache and IndexedDB mutation outbox |
```

Follow with explicit “Good fit” and “Choose something else when” guidance so the starter’s boundaries are understandable before installation.

- [ ] **Step 2: Add the five-minute path**

Use these exact stages and commands:

```markdown
## Five-minute start

### 1. Clone and run

```bash
git clone https://github.com/anishk123/cloudflare-rust-htmx-starter.git my-app
cd my-app
cargo xtask dev
```

Open `http://localhost:8787`. No Cloudflare account is required for local development.

### 2. Make it yours

```bash
cargo xtask configure my-app --title "My App" --github my-org/my-app
```

Edit HTML in `crates/templates/templates/`, tokens/components in
`public/assets/app.css`, and route coordination in `workers/app/src/routes/`.

### 3. Verify

```bash
cargo xtask verify
```
```

Explain Rust/rustup and Node 22+ prerequisites immediately before the commands.

- [ ] **Step 3: Add progressive navigation and demo-to-product mapping**

Add `## Choose your path` linking directly to:

- `docs/LOCAL_DEVELOPMENT.md`
- `docs/EXTENDING.md`
- `/design-system` plus the template/CSS paths
- `docs/ARCHITECTURE.md`
- `docs/AUTH.md`
- `docs/SECURITY.md`
- `docs/DEPLOYMENT.md`
- `CONTRIBUTING.md`

Add `## From Evidence Notes to your product` mapping note creation, D1 persistence, Queue summarization, R2 upload boundaries, public representations, and offline replay to reusable product patterns.

- [ ] **Step 4: Tighten AGENTS.md into the normative companion**

Preserve the canonical command block and architecture rules. Add:

```markdown
## Documentation rules

1. Keep `README.md` as orientation and route deeper detail to focused docs.
2. Keep `AGENTS.md` normative; tool-specific agent files point here rather than copying rules.
3. Update `CONTRIBUTING.md` when the human/agent contribution loop changes.
4. Use current Askama and `public/assets/` paths in active documentation.
5. Preserve historical decision records under `docs/superpowers/`; add context instead of rewriting history.
6. Prefer copyable commands, explicit prerequisites, honest claims and a clear next step.
```

Expand `## Completion` to require documentation updates when behavior/contracts/commands change and exact command reporting.

- [ ] **Step 5: Turn CONTRIBUTING.md into the shared first-contribution guide**

Use these sections:

```markdown
## First contribution
## Find the right layer
## Feature workflow
## Pull request checklist
## Review-friendly changes
```

The checklist must require focused tests, `cargo xtask verify`, relevant e2e/local flow checks, contract versioning, cache/offline/publication review, documentation updates, and no secrets/generated state.

- [ ] **Step 6: Align remaining root documentation**

- Update `CHANGELOG.md` with a `0.3.0` entry for Askama templates, the starter-owned design system, route/cache modularization, SEO/AEO, and onboarding; keep `0.2.0` as historical chronology but phrase Pico as what that release introduced without treating it as current guidance.
- Make `CLAUDE.md`, `CODEX.md`, and `OMNIAGENT.md` point to `CONTRIBUTING.md` after the authoritative `AGENTS.md`/`README.md` read.
- Expand the pull request template with a short purpose/impact prompt and exact verification-command reporting.
- Link root `SECURITY.md` to `docs/SECURITY.md`, `docs/AUTH.md`, and private vulnerability reporting.
- Keep third-party notices factual and link the vendoring command/reference guide.

- [ ] **Step 7: Verify the front door turns the guardrail GREEN**

Run: `cargo xtask check`

Expected: PASS with `template checks passed`.

Run: `git diff --check`

Expected: exit zero.

- [ ] **Step 8: Commit the front door and guardrails**

```bash
git add README.md AGENTS.md CONTRIBUTING.md CHANGELOG.md CLAUDE.md CODEX.md OMNIAGENT.md SECURITY.md THIRD_PARTY_NOTICES.md .github/pull_request_template.md crates/xtask/src/main.rs
git commit -m "docs: create a progressive onboarding journey"
```

### Task 3: Turn focused guides into a progressive learning path

**Files:**
- Modify: `docs/LOCAL_DEVELOPMENT.md`
- Modify: `docs/COMMANDS.md`
- Modify: `docs/EXTENDING.md`
- Modify: `docs/ARCHITECTURE.md`
- Modify: `docs/CONTRACTS.md`
- Modify: `docs/AUTH.md`
- Modify: `docs/SECRETS.md`
- Modify: `docs/SECURITY.md`
- Modify: `docs/DEPLOYMENT.md`
- Test: `cargo xtask check`

**Interfaces:**
- Produces: one-purpose guides with prerequisites, copyable examples, concrete repository paths, and an explicit next step.
- Consumes: README orientation without duplicating the entire quick start.

- [ ] **Step 1: Expand local development and command reference**

`docs/LOCAL_DEVELOPMENT.md` must cover prerequisites, first run, what `dev` bootstraps, local state locations, safe reset, dev identity, common failures (port 8787, missing Node/Rust target), and the next path to `docs/EXTENDING.md`.

`docs/COMMANDS.md` must group every command under `Start`, `Build and verify`, `Create/customize`, and `Cloudflare operations`. For each command, state whether it is local-only, networked, or remote/mutating. Preserve exact command names from `cargo xtask help`.

- [ ] **Step 2: Make extending the starter a file-by-file tutorial**

`docs/EXTENDING.md` must show:

```text
contract → failing test → domain rule → prepared SQL → Askama page/fragment
→ route policy → queue/offline only when justified → verify
```

Include a “Where does this change go?” table for contracts, domain, database, templates, routes, jobs, CSS, JS, migrations, and automation. Add an external-template example that names `crates/templates/templates/`, `crates/templates/src/`, and `/design-system` without embedding HTML in routes.

- [ ] **Step 3: Add navigable architecture depth**

`docs/ARCHITECTURE.md` must cover request flow, ownership of each boundary, public/private caching, offline data flow, and the reason for the separate jobs Worker. Add a repository map and decision rules for when to create a new crate/module versus extend an existing focused unit.

- [ ] **Step 4: Expand contracts into a safe evolution guide**

`docs/CONTRACTS.md` must explain which shapes are durable, versioning expectations, `/contracts`, producer/consumer coordination, and a concrete checklist for changing a Queue or offline contract safely.

- [ ] **Step 5: Layer security guidance without blocking local onboarding**

- Keep `docs/AUTH.md` as the detailed identity/ownership/CSRF boundary and add “Before real user data” plus “Next step” summaries.
- Expand `docs/SECRETS.md` with local/deployed/CI placement and rotation guidance, while preserving the control-plane/runtime distinction.
- Add a production-readiness checklist and next links to `docs/SECURITY.md` without duplicating the auth or secrets guides.

- [ ] **Step 6: Make deployment deliberate and reversible**

`docs/DEPLOYMENT.md` must start with prerequisites and a “local verification first” gate, then describe auth, provisioning, migrations, deployment order, post-deploy checks, rollback limitations, and preview/staging resource isolation. State clearly that ordinary pushes and PRs never deploy.

- [ ] **Step 7: Audit headings, internal links, terminology and next steps**

Run:

```bash
rg -n "Maud|maud|Pico CSS|pico.min.css|public/app.css|public/app.js|public/vendor/" \
  README.md AGENTS.md CONTRIBUTING.md CHANGELOG.md CLAUDE.md CODEX.md OMNIAGENT.md \
  SECURITY.md THIRD_PARTY_NOTICES.md .github/pull_request_template.md docs/*.md
```

Expected: no matches.

Run: `cargo xtask check`

Expected: PASS with `template checks passed`.

- [ ] **Step 8: Commit the focused guides**

```bash
git add docs/ARCHITECTURE.md docs/AUTH.md docs/COMMANDS.md docs/CONTRACTS.md docs/DEPLOYMENT.md docs/EXTENDING.md docs/LOCAL_DEVELOPMENT.md docs/SECRETS.md docs/SECURITY.md
git commit -m "docs: add progressive contributor learning paths"
```

### Task 4: Verify generated-starter onboarding and complete the documentation review

**Files:**
- Modify: `crates/xtask/src/main.rs`
- Modify only documentation implicated by review failures.
- Test: `cargo xtask smoke`

**Interfaces:**
- Produces: proof that a generated application retains the human/agent onboarding contract after branding substitutions.
- Consumes: `template_smoke()` generated directory and all documentation from Tasks 2–3.

- [ ] **Step 1: Add generated-document regression assertions**

In `template_smoke()`, after generation, require:

```rust
for (path, needles) in [
    ("README.md", &["Five-minute start", "Choose your path"][..]),
    ("AGENTS.md", &["## Documentation rules", "## Completion"][..]),
    (
        "CONTRIBUTING.md",
        &["## First contribution", "## Pull request checklist"][..],
    ),
] {
    let source = fs::read_to_string(dir.join(path)).map_err(|error| error.to_string())?;
    for needle in needles {
        if !source.contains(needle) {
            return Err(format!("generated {path} missing onboarding contract: {needle}"));
        }
    }
}
```

Run: `cargo xtask smoke`

Expected: PASS without special generator branches, proving the implemented source documentation is copied into a generated application. Temporarily change one expected heading locally and confirm the assertion fails, then restore it before continuing.

- [ ] **Step 2: Run the complete active-doc audit**

Run: `git ls-files '*.md'`

Inspect every listed active document and explicitly distinguish historical `docs/superpowers/` records from current guidance.

Run: `rg -n "T[B]D|T[O]DO|PLACEH[O]LDER|<starter-repository-url>" --glob '*.md' --glob '!docs/superpowers/**'`

Expected: no matches.

- [ ] **Step 3: Validate current command and repository-path claims**

Run: `cargo xtask help`

Run: `rg --files crates/templates/templates public/assets workers/app/src/routes docs | sort`

Compare every documented command/path against these outputs. Fix only factual mismatches.

- [ ] **Step 4: Run focused automation**

Run: `cargo fmt --all -- --check`

Run: `cargo xtask check`

Run: `cargo xtask smoke`

Run: `node --check public/assets/app.js`

Run: `node --check public/sw.js`

Expected: every command exits zero.

- [ ] **Step 5: Commit generated-onboarding verification and review fixes**

```bash
git add crates/xtask/src/main.rs README.md AGENTS.md CONTRIBUTING.md CHANGELOG.md CLAUDE.md CODEX.md OMNIAGENT.md SECURITY.md THIRD_PARTY_NOTICES.md .github/pull_request_template.md docs
git commit -m "test: enforce starter onboarding documentation"
```

### Task 5: Final verification, publication and green CI

**Files:**
- Modify only files implicated by verification or CI failures.

**Interfaces:**
- Consumes: the complete feature branch.
- Produces: a pushed branch, draft pull request, and green required GitHub Actions checks.

- [ ] **Step 1: Run the canonical local gate on the final commit**

Run:

```bash
cargo fmt --all -- --check && cargo xtask verify && cargo xtask e2e && cargo xtask smoke
```

Expected: every command exits zero. If a command fails, diagnose the root cause, add a focused regression test when behavior changed, fix it, and repeat the complete chain.

- [ ] **Step 2: Confirm publish scope and GitHub authentication**

Run: `git status -sb`

Expected: clean `codex/state-of-art-starter` branch.

Run: `gh auth status`

Expected: authenticated GitHub account with repository access. If invalid, stop and ask the user to run `gh auth login -h github.com`; do not expose or request a token in chat.

- [ ] **Step 3: Push the branch**

Run: `git push -u origin codex/state-of-art-starter`

Expected: branch is published to `origin` with upstream tracking.

- [ ] **Step 4: Open a draft pull request**

Create a draft PR targeting the remote default branch with:

- title: `Refactor starter for approachable, state-of-the-art product development`
- summary of external Askama templates, starter-owned UI, modular Worker/cache/security boundaries, SEO/AEO, generator hardening, and progressive documentation;
- exact local verification commands;
- explicit note that this PR does not deploy Cloudflare resources.

- [ ] **Step 5: Monitor GitHub Actions to completion**

Run: `gh pr checks --watch --fail-fast=false`

Expected: all required checks pass. If a check fails, invoke `github:gh-fix-ci`, inspect the failing Actions logs, implement only an approved in-scope fix, rerun local verification, commit, push, and watch again.

- [ ] **Step 6: Report the published result**

Report branch, commit, PR URL/target, exact local commands, GitHub check status, and any intentionally deferred operational step. Do not call the task done while a required check is pending or failing.
