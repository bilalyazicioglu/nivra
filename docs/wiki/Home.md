# Nivra

**It worked twenty minutes ago. What changed?**

Nivra connects shell commands with observed Git state in a local development timeline. Save a working baseline, make a change, and inspect what changed around a failure. Previously codenamed Rewind.

## Status: source alpha

The core implementation merged through [PR #4](https://github.com/bilalyazicioglu/nivra/pull/4) as commit `419397b0422354f27bb0eab0a1d1e25086fc5511`. It passed macOS/Linux CI and is available on `main`, but no release binary has been published. These pages describe the implemented source alpha and planned work separately.

Implemented on `main`: explicit command recording, opt-in zsh hooks, local SQLite, Git before/after snapshots, repository-scoped marks/comparison, pause/resume, ignore-next and heuristic redaction.

Planned, not shipped: interactive TUI, current port inspection/controls, expanded comparison, configuration/retention, release installers, other shells, runtime history, portable bundles, optional sync and sharing.

## Start here

- [Quickstart](Quickstart.md) — build from source, install and try a real demo.
- [Troubleshooting](Troubleshooting.md) — command not found, missing marks and capture issues.
- [Roadmap](Roadmap.md) — all milestones and linked work items.
- [Agent workflow](Agent-Workflow.md) — smaller assignments, dependencies and review requirements.
- [Architecture](Architecture.md) — current event/snapshot model and boundaries.
- [Privacy](Privacy.md) — stored metadata and controls.
- [Development](Development.md) — checks, contribution process and release gates.

## Work plan

There are 41 new scoped/design-gated work items, plus the original three tracking issues. Each detailed issue names dependencies, entry files, acceptance criteria, validation and exclusions. The plan distinguishes small-agent tasks from specialist work and explicitly defers future designs.

[Repository plan](https://github.com/bilalyazicioglu/nivra/blob/main/docs/PLAN.md) · [Issues](https://github.com/bilalyazicioglu/nivra/issues) · [Milestones](https://github.com/bilalyazicioglu/nivra/milestones)

No accounts, cloud service or AI are required. Nivra reports observations, not proof that a command caused a bug. Local storage still contains sensitive metadata; read [Privacy](Privacy.md) before enabling daily capture.
