# Nivra: first milestones

Status: M0 implemented locally; remote CI/review pending · 2026-09-26

Nivra was previously codenamed Rewind. The old engineering document is product input, not a promise that all features ship in the first release.

## Product contract

Answer “what changed between working and broken?” with a local, inspectable timeline. Compare observations; never imply that a command caused a change. No account, telemetry, remote service, output recording, environment capture, or AI dependency.

## M0 — a credible foundation (today)

- Modular Rust CLI, local SQLite store and versioned schema.
- Explicit `nivra run -- <program> <args>` recording and opt-in zsh hooks.
- Session/event/snapshot model; exit status, timing and repository identity.
- Named marks and comparisons that notice edits to already-dirty files.
- Pause/resume, ignore-next, redaction before persistence, private storage permissions.
- Repeatable demo in a temporary repository; integration tests for the real binary.
- README, contribution/security/conduct policies, issue forms, PR template, CI and wiki source pages.

Acceptance: a passing test, a mark, a real source edit, a failing test, and a comparison that names the changed file and intervening commands. Shell failure must not be caused by a capture failure.

## M1 — daily-use alpha

- Measure hook overhead on small and large repositories; publish median/p95.
- Interactive timeline and event inspector, search and session filters.
- Retention, exclusions and configuration; expanded secret-handling fixtures.
- Current port inspection with honest platform capability reporting.
- Validate interactive zsh behavior, nested shells, interrupted commands and locked storage across macOS/Linux CI.

Gate: use it for a week, record shortcomings as reproducible issues, fix data-loss and shell-interference bugs before a public release.

## M2 — public preview

- Signed/checksummed release artifacts and clean-machine installation verification.
- Real 15–20 second screencast, supported-platform matrix, reproducible demo.
- GitHub milestones, triage labels, verified CI checks and protected default branch.
- Published wiki from reviewed `docs/wiki` sources.
- Reddit post showing an actual failure investigation, known limitations and a specific feedback question.

Gate: no mocked success claims, performance claims without measurements, or advertised unimplemented features. Stars are an outcome, not an engineering requirement.

## Later

Bash/Fish, runtime history, portable export, optional encrypted sync. Each needs a separate design and acceptance tests.

## Working practices

One focused issue → one branch → one reviewable PR → CI → squash merge. Bugs need reproductions; feature requests need a developer scenario. Keep docs and behavior in the same PR. For solo maintenance, document self-review instead of inventing additional reviewers.

Reference patterns: [bat](https://github.com/sharkdp/bat) for a demonstration-led README and contributor entry points; [Atuin](https://github.com/atuinsh/atuin) for shell-tool documentation. Nivra focuses on state comparisons rather than history search.

## First implementation validation

Local macOS: 9 Rust tests, formatting and Clippy pass. A real PTY-driven interactive zsh test verifies failure exit codes, Ctrl-C (130), and missing-binary fail-open behavior. The temporary-repository demo passes. See the PR for remote CI results.
