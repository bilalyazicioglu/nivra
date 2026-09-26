# Working with agents

Assign one ready issue, not a milestone or the old Rewind plan. Each issue includes outcome, dependencies, entry files, acceptance criteria, validation and exclusions.

| Tier | Use for |
| --- | --- |
| `agent: small` | Narrow tests, CLI additions using existing interfaces, benchmarks, docs/demo work |
| `agent: standard` | Bounded modules and UI integration using accepted contracts |
| `agent: specialist` | Contracts, migrations, capture budgets, privacy, process safety, imports, encryption and release judgment |

Size S/M means one focused task/PR. Size L must be split before coding. Future milestones are deliberately deferred. A lower tier is a routing suggestion, not a promise of cost or correctness.

## Workflow

1. Check dependencies are merged and contracts accepted; have the maintainer mark the task ready.
2. Claim one issue and create a separate branch/worktree.
3. Read only the task, workflow, accepted contracts and relevant files.
4. Implement within acceptance criteria; coordinate edits to shared main/store/model modules.
5. Run the task's checks and submit one PR with exact evidence and remaining risks.
6. Maintainer reviews/merges and unlocks direct dependents. Agents do not self-approve or infer launch permission.

The full [agent workflow and copyable handoff prompt](https://github.com/bilalyazicioglu/nivra/blob/docs/agent-ready-roadmap/docs/AGENT_WORKFLOW.md) are maintained in the repository. [Roadmap](Roadmap.md) contains all issue links.

For the first low-cost assignments after the planning source is accepted, use [#7](https://github.com/bilalyazicioglu/nivra/issues/7) and [#8](https://github.com/bilalyazicioglu/nivra/issues/8). Do not ask an agent to fabricate a seven-day usage report or an independent security review.
