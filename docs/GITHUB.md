# GitHub operating model

## Sources and current state

Core code merged through PR #4. The documentation plan remains a separate PR so it can be reviewed without expanding the implementation diff. The initialized Wiki is published from `docs/wiki`; Home/footer identify the source revision and source-preview status.

- [Execution order](PLAN.md)
- [Machine-readable backlog](BACKLOG.json)
- [Agent handoff and definition of done](AGENT_WORKFLOW.md)
- [Live issues](https://github.com/bilalyazicioglu/nivra/issues)
- [Milestones](https://github.com/bilalyazicioglu/nivra/milestones)

## Labels

| Dimension | Values | Meaning |
| --- | --- | --- |
| Type | task, design, epic | Bounded implementation, contract decision, or tracking/work package |
| Agent | small, standard, specialist | Suggested routing, not model names or cost guarantees |
| Size | S, M, L | Narrow task, focused integration PR, or split-before-code work package |
| Priority | P0, P1, P2 | Foundation/safety gate, milestone requirement, secondary/deferred work |
| Status | needs-triage, blocked, ready, in-progress, deferred | Maintainer-managed readiness after dependency verification |
| Area | capture, privacy, ux, docs | Additional navigation; not required on every issue |

Maintain only one status label at a time. Apply `good first issue` only after scope/interfaces are clear and dependencies have merged; `agent: small` does not automatically mean beginner-ready. Marking a task ready requires acceptance evidence, not just a closed dependency issue.

## Tracking and milestones

#1 is the core acceptance tracker. #2 lists the bounded M1 tasks. #3 lists the M2 evidence/release tasks. Do not assign these broad trackers as a single coding task. Each detailed issue has a stable planning key and dependency links in `docs/BACKLOG.json`.

M0 establishes the core and contributor plan. M1 is the daily-use alpha. M2 is the public-preview gate. M3–M6 are deliberately deferred portability, runtime, optional sync and sharing directions. Large future packages must be split after design acceptance. There are no invented calendar deadlines.

## Branches, checks and review

The current main protection requires PRs, resolved conversations, `Rust / macos-latest` and `Rust / ubuntu-latest`, including for administrators. Force pushes and branch deletion are blocked. Approval count is zero for solo maintenance; that is not a claim of independent review.

One issue → one branch/worktree → focused PR → validation → maintainer review → squash merge. Privacy, migrations, process control, import and encryption boundaries require specialist scrutiny. Do not bypass protection for docs. If a docs-only PR predates the core workflow on main, keep it pending until #4 is merged and rebase; confirm required checks run on the resulting branch. The planning workflow checks the manifest graph and local documentation targets separately.

## Wiki publication

Canonical Markdown pages live in `docs/wiki`. Wiki links omit `.md`; repo source links retain it. The initial publication is user-authorized during planning and explicitly identifies unmerged preview code/docs instead of implying a released main branch.

1. Clone or fast-forward `https://github.com/bilalyazicioglu/nivra.wiki.git` in a separate temporary directory.
2. Review current pages; preserve unrelated user-created content.
3. Copy the selected source pages, rewrite relative page links to wiki URLs, and verify every target exists.
4. Include a source revision marker in Home/footer. Preview documentation must identify its PR/source revision.
5. Inspect the diff, commit and push only when authorized; verify the remote commit matches.

Keep later publication manual until the checked publisher task is implemented. Do not store tokens in the repo or introduce a write-enabled PR workflow. Documentation changes go through PR review; a wiki edit must be reconciled back to sources rather than silently overwritten.
