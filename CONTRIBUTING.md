# Contributing to Nivra

Nivra helps developers compare development state over time. Read the [product plan](docs/PLAN.md) and [architecture](docs/wiki/Architecture.md) before proposing broad changes.

## Local development

Install current stable Rust, Git, and zsh. Clone the repository and run:

```sh
cargo build --locked
cargo test --locked
sh scripts/demo.sh
python3 scripts/test-zsh.py
```

Set `NIVRA_DATA_DIR` to a dedicated temporary directory while developing so experiments never touch your daily history. Tests create their own repositories and databases. zsh integration tests skip if zsh is unavailable locally; CI installs it.

## A useful contribution

1. Search existing issues. For substantial behavior changes, describe the debugging scenario and acceptance criteria in an issue first.
2. Create a focused branch, such as `fix/mark-repository-scope` or `feat/timeline-search`.
3. Change behavior, meaningful tests, and relevant documentation together. Avoid unrelated cleanup.
4. Run formatting, Clippy with warnings denied, and the full test suite (commands in README).
5. Open a PR explaining the problem, resulting behavior, validation, and known limits. Link an issue when there is one.

Use an imperative, descriptive commit subject. Conventional prefixes are welcome but not required. A bug fix should reproduce the original bug. Tests should verify behavior, especially exit-code preservation, partial capture, path handling, and privacy boundaries.

## Review and merge

Maintainers review correctness, scope, user-facing claims and privacy behavior. CI must pass before merging. External contributions require maintainer review; solo-maintained changes use an explicit self-review note. Squash merge focused PRs, preserving attribution. Do not manufacture reviews or activity to make the repository look larger.

Update CHANGELOG.md for user-visible changes. Documentation-only corrections do not need an entry. Never add real shell histories, credentials, personal paths, or proprietary source to fixtures.

## First contributions

Good starting areas: a minimal reproduction of a shell edge case, a failing redaction fixture using synthetic credentials, installation documentation verified on a clean machine, or a small accessibility/output improvement. The maintainer applies `good first issue` only when scope and acceptance criteria are ready.

## Community

Follow the [code of conduct](CODE_OF_CONDUCT.md). Disagree with ideas respectfully, provide concrete evidence, and help make review understandable. For security issues, follow [SECURITY.md](SECURITY.md) instead of opening a public issue.
