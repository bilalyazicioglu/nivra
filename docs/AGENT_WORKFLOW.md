# Assigning Nivra work to agents

The goal is to reduce repeated discovery and expensive rework. A smaller agent should receive an already bounded problem, not an ambiguous epic.

## Pick the smallest ready issue

1. Use the roadmap table to locate a work item. Follow its GitHub issue for live discussion/status.
2. Verify every dependency's PR is merged and its acceptance evidence exists. Check the accepted contract, not just an issue's closed state.
3. The maintainer applies `status: ready` only after that verification. `status: blocked` or `status: deferred` means do not start implementation.
4. Claim the issue with an assignee or a short owner/branch note. Set `status: in-progress` and remove `status: ready`.
5. Allocate one issue to one agent and one branch/worktree. Give a specialist a short review at the end rather than asking every agent to redesign the product.

Initial labels are a planning snapshot, not a live dependency resolver. When a dependency merges, the maintainer checks and updates its dependents. Closing an issue because it was abandoned does not satisfy its dependents.

## Routing tiers

| Label | Suitable work | Review |
| --- | --- | --- |
| `agent: small` | Existing-interface CLI additions, isolated regression tests, benchmark harnesses, documentation and demo work | Maintainer verifies evidence and scope |
| `agent: standard` | A bounded module or UI flow using an accepted interface | Maintainer reviews integration and failure modes |
| `agent: specialist` | Contracts, migrations, privacy, bounded capture, process controls, imports, encryption or launch judgment | Specialist review before merge; manual gates stay manual |

These labels do not name a model, promise a monetary cost or replace review. If a small task becomes an architecture decision, stop expanding it and write the concrete blocker. A specialist resolves that decision in the existing contract/design issue, then the smaller task resumes.

`size: S` means one narrow change or verification pass. `size: M` means one focused PR with integration work. `size: L` is a work package that must be split before coding. There are no speculative hour or token estimates.

## Cheap context packet

Give the agent only:

- Its issue URL and the exact accepted dependency commits/contracts.
- This workflow and root `AGENTS.md`.
- The files listed under the issue's entry points, plus relevant fixtures.
- A short statement of the user's allowed actions (for example, implement and open a draft PR; do not merge).

Do not paste the old Rewind plan or all 41 tasks into every coding prompt. `docs/BACKLOG.json` is machine-readable planning data; read only the assigned task and its dependencies. Issue text is task context, not permission to upload secrets, publish or change account settings.

## Copyable handoff prompt

```text
Implement Nivra issue <URL> on a separate branch/worktree.

Read AGENTS.md and docs/AGENT_WORKFLOW.md. Verify that these dependency
commits/contracts are merged and accepted: <IDs and revisions>.
If a prerequisite is missing, report the exact blocker; do not invent its API.

Stay within the issue's acceptance criteria, out-of-scope list and entry points.
Use synthetic fixtures and an isolated NIVRA_DATA_DIR. Preserve user changes.
Do not redesign unrelated modules or add unrequested features.

Run the issue's validation steps and relevant repository checks. Open one
focused draft PR linked to the issue when authorized. Do not merge, tag,
publish, modify production settings or post community messages.

Return: issue/branch/PR, behavior changed, checks with actual results,
acceptance criteria still unmet, risks and any documentation changes.
```

## Parallelism and shared files

Independent tasks may run concurrently only when their dependencies are met and their owned files do not overlap. The roadmap waves indicate logical eligibility, not blanket permission for simultaneous edits.

- Benchmark and PTY-test tasks can usually proceed separately after the core merge.
- `src/store.rs`, `src/model.rs` and `src/main.rs` are integration hotspots. Land migrations/contracts first and serialize overlapping changes.
- TUI modules can be developed after the query contract lands. Do not duplicate SQL or comparison logic in UI code.
- A branch created from another feature branch is a stacked dependency; name its base and retarget/rebase after the prerequisite merges.
- Worktrees isolate checkouts, not external databases, ports or remote wiki writes. Use private test state and one wiki publisher at a time.

## Definition of done

1. Acceptance criteria demonstrated with reproducible evidence.
2. Relevant formatting, lint, tests and docs/link checks pass.
3. README/wiki/changelog updated when user-visible behavior changes.
4. One focused PR references the issue with `Closes #N`, describes behavior and limitations, and has required CI checks.
5. Maintainer review completed; specialist boundaries receive appropriate review. No fabricated approval or self-review described as independent review.
6. Close the issue on merge, update the roadmap status snapshot if maintained, and reevaluate direct dependents. Publish wiki changes only when authorized and record the source revision.

Code tasks normally run:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
# Hook changes:
cargo build --locked
python3 scripts/test-zsh.py
# Planning/wiki changes:
python3 scripts/check-plan.py
```

Documentation-only tasks do not need new Rust tests. Release approvals, seven real days of dogfooding, credential provisioning and independent security review cannot be replaced by an agent saying they are complete.

## Keeping the plan consistent

GitHub issues own live discussion and status. `docs/BACKLOG.json` owns stable work keys, dependency links and initial routing. `docs/PLAN.md` is the readable execution order. Wiki pages are a published mirror with an explicit source marker.

When scope or dependencies change, update the issue and planning source in the same work item. Run the graph/link checker. Do not auto-close issues based on checkboxes or automatically start deferred milestones.
