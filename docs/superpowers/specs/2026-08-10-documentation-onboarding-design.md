# Progressive Documentation and Onboarding Design

**Date:** 2026-08-10  
**Status:** Approved for planning

## Goal

Make the repository immediately useful to a first-time human or coding agent, then reveal deeper architecture, security, operations, and contribution guidance as they gain confidence. A newcomer should reach a successful local run quickly, understand where product changes belong, and know how to contribute without reading the entire repository first.

## Audience

The primary path serves developers who may be new to Rust, Cloudflare Workers, HTMX, or one of the three. It must also serve coding agents that need authoritative commands and architectural boundaries without relying on inference.

Experienced users should be able to skip directly to focused reference material. Security and production guidance remains explicit, but does not interrupt the local-first path.

## Documentation Architecture

### README: orientation and first success

The README is the front door. It will:

1. State the product promise and intended use cases in plain language.
2. Show the important capabilities and deliberate non-goals.
3. Provide a five-minute `clone → run → customize → verify` path.
4. Explain what the Evidence Notes example demonstrates and how its pieces map to a real product.
5. Provide a “choose your path” index for common next steps: build a feature, edit the UI, understand the architecture, add authentication, work offline, deploy, and contribute.
6. Describe performance and caching defaults with measurable facts rather than universal speed claims.
7. Link to focused reference documents instead of duplicating their full content.

### AGENTS.md: authoritative execution contract

`AGENTS.md` remains concise and normative. It will expose:

- canonical commands;
- the external Askama-template and starter-owned design-system workflow;
- contract-first and test-first feature sequencing;
- security, caching, offline, and deployment boundaries;
- a completion checklist that requires exact verification reporting;
- documentation expectations so agent-authored changes preserve the progressive onboarding model.

Tool-specific agent entry files will continue to point to this single contract so instructions do not drift.

### Focused docs: progressive depth

Each document under `docs/` will have one clear job, state relevant prerequisites, link to the next likely document, and avoid repeating source-of-truth material unnecessarily.

The documentation set will cover:

- architecture and request/data flow;
- local development and the command surface;
- extending the starter and editing external templates;
- durable contracts;
- authentication and authorization boundaries;
- application and control-plane secrets;
- security defaults and production review points;
- provisioning, migrations, and deployment.

Root governance and security documents will be checked for consistency with the focused guides.

## Voice and Presentation

The voice is confident, direct, and welcoming. “Pizzazz” comes from a clear narrative, useful signposts, compact capability summaries, and visible momentum—not hype, excessive emoji, or decorative complexity.

Claims must remain verifiable. The documentation may describe the enforced CSS and JavaScript budgets, server-rendered fast path, edge/static caching policies, and absence of hydration. It must also state that production Core Web Vitals depend on the resulting application and require field measurement.

Code samples will be copyable and ordered by when a newcomer needs them. Paths and commands will match the repository exactly.

## Consistency Audit

The implementation will inspect all tracked Markdown documentation, including root guides, `docs/`, agent entry files, contribution/security policies, and templates used by the project generator. Historical specs and plans under `docs/superpowers/` will retain their decision record; they will be checked for status/context but will not be rewritten to pretend they were authored after the implementation.

The audit will remove or correct:

- stale Pico CSS, Maud, and old asset-path references;
- commands that differ from the canonical `cargo xtask` surface;
- broken or ambiguous internal links;
- duplicated instructions that can drift;
- missing prerequisites or unclear environment scope;
- claims that are stronger than the implemented behavior;
- sections that lack a clear next step.

## Contribution Journey

A contributor should be able to follow this sequence without private context:

1. Run the project locally.
2. Locate templates, styles, route modules, domain rules, and durable contracts.
3. Choose the appropriate feature workflow.
4. Add a failing test before implementation.
5. Preserve progressive enhancement, caching, security, and offline boundaries.
6. Run the canonical checks.
7. Open a well-scoped pull request with the exact validation commands reported.

The README will introduce this journey; `CONTRIBUTING.md`, `AGENTS.md`, and focused guides will provide the details.

## Automated Guardrails

Automation will enforce only durable, high-signal documentation rules. Existing structural checks may be extended to detect known stale framework/path terminology and ensure required onboarding/contribution links remain present. Checks will avoid subjective prose tests or brittle assertions tied to paragraph wording.

The final local gate is:

```bash
cargo fmt --all -- --check
cargo xtask verify
cargo xtask e2e
cargo xtask smoke
```

After local success, the branch will be pushed and a draft pull request opened. GitHub Actions must complete successfully before the work is reported as done. No Cloudflare deployment is part of this documentation publication task.

## Acceptance Criteria

- A first-time user can identify prerequisites, start the app, locate the main customization points, and run verification from the README alone.
- A coding agent can find all normative commands and architecture rules in `AGENTS.md` without conflicting instructions elsewhere.
- Every active documentation file has an explicit purpose and consistent current terminology; historical decision records are clearly contextualized.
- Human and agent contribution paths converge on the same feature workflow and verification commands.
- Performance, caching, offline, authentication, and deployment claims match the implementation.
- High-signal documentation consistency checks pass locally and in CI.
- The complete local verification suite passes on the final commit.
- The published pull request’s required GitHub Actions checks are green.
