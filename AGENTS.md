# Nivra contributor-agent instructions

## Start from an issue

Read the assigned issue and `docs/AGENT_WORKFLOW.md`. Consult `docs/PLAN.md` for release gates and `docs/BACKLOG.json` for dependency IDs. Do not read the entire backlog into context unless planning work requires it.

Before code changes, verify that prerequisites are merged, accepted contracts exist and the task is claimed. The core implementation landed through PR #4; later dependencies still require their accepted merge revisions. Never assume an open issue, checked box or existing branch means a dependency is available.

Use one issue per branch/worktree. Do not spawn or assign other agents unless the user asks. Do not implement a size-L work package until a maintainer splits it into bounded issues. Shared edits to main/store/model require coordination; separate worktrees alone do not prevent merge conflicts.

## Product invariants

- Local-first; no required account, server or AI.
- Capture failure must not block the user's command or change its exit status.
- Redact before storage; no terminal output, environment dump or source-body persistence by default.
- Report observations and uncertainty, not causation.
- Never replay a captured command implicitly. Process stop/restart and import boundaries need specialist review.
- Keep changes within the issue's acceptance criteria. Preserve existing user changes.

## Validation and handoff

For Rust changes run formatting, Clippy with warnings denied and locked tests. For hook changes also run the real PTY test. For planning/wiki edits run `python3 scripts/check-plan.py` where available. Use synthetic fixtures and an isolated data directory.

The final handoff includes issue, branch, changed behavior, exact checks/results and remaining risks. Leave public release, wiki publication, merge, tagging and community posting to the user/maintainer unless explicitly authorized in the current task. Do not invent approvals, measurements or finished manual testing. Never bypass required CI or branch protection.
