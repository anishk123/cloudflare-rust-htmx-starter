# Contributing

Start locally with `cargo xtask dev`. Follow `AGENTS.md` whether you are a human or coding agent. Every behavioral change should include focused tests. Before opening a pull request run `cargo xtask verify`; explain any verification step you genuinely could not execute.

Do not deploy from a feature branch unless a human explicitly requests it. Keep dependencies and architecture minimal; new client frameworks, build pipelines, ORMs, or long-lived services require an explicit design reason.
