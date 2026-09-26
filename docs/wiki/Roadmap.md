# Roadmap

This is a source-alpha roadmap, not a list of shipped features. The core merged through [PR #4](https://github.com/bilalyazicioglu/nivra/pull/4); later items remain gated by their linked dependencies.

[Full execution plan and dependency graph](https://github.com/bilalyazicioglu/nivra/blob/main/docs/PLAN.md) · [Machine-readable backlog](https://github.com/bilalyazicioglu/nivra/blob/main/docs/BACKLOG.json) · [Agent workflow](Agent-Workflow.md)

After the planning source is accepted, assign [#7](https://github.com/bilalyazicioglu/nivra/issues/7) and [#8](https://github.com/bilalyazicioglu/nivra/issues/8) for bounded benchmark/test work while a specialist owns [#6](https://github.com/bilalyazicioglu/nivra/issues/6). The TUI follows the query and comparison contracts.

## M0 — Core proof and planning

- [#5](https://github.com/bilalyazicioglu/nivra/issues/5) **P00** — Publish an agent-ready roadmap and the initialized wiki (small, S)

## M1 — Daily-use alpha

- [#6](https://github.com/bilalyazicioglu/nivra/issues/6) **A01** — Define snapshot, query and comparison contracts before parallel implementation (specialist, M)
- [#7](https://github.com/bilalyazicioglu/nivra/issues/7) **A02** — Add reproducible release-binary capture benchmarks (small, S)
- [#8](https://github.com/bilalyazicioglu/nivra/issues/8) **A03** — Expand interactive zsh regression coverage (small, S)
- [#9](https://github.com/bilalyazicioglu/nivra/issues/9) **A04** — Implement validated local configuration loading (standard, M)
- [#10](https://github.com/bilalyazicioglu/nivra/issues/10) **A05** — Harden SQLite migrations, concurrent initialization and private storage (specialist, M)
- [#11](https://github.com/bilalyazicioglu/nivra/issues/11) **A06** — Enforce a total capture budget with explicit partial snapshots (specialist, M)
- [#12](https://github.com/bilalyazicioglu/nivra/issues/12) **A07** — Add configurable exclusions and strengthen pre-storage redaction (specialist, M)
- [#13](https://github.com/bilalyazicioglu/nivra/issues/13) **A08** — Separate command elapsed time from wall-clock event timestamps (standard, M)
- [#14](https://github.com/bilalyazicioglu/nivra/issues/14) **A09** — Extract a paginated read API for events, sessions and marks (standard, M)
- [#15](https://github.com/bilalyazicioglu/nivra/issues/15) **A10** — Add event inspection and named-mark listing commands (small, S)
- [#16](https://github.com/bilalyazicioglu/nivra/issues/16) **A11** — Add session browsing and composable timeline search (standard, M)
- [#17](https://github.com/bilalyazicioglu/nivra/issues/17) **A12** — Implement structured state comparison shared by CLI and TUI (specialist, M)
- [#18](https://github.com/bilalyazicioglu/nivra/issues/18) **A13** — Expand comparisons to staged and committed Git changes (specialist, M)
- [#19](https://github.com/bilalyazicioglu/nivra/issues/19) **A14** — Implement explicit retention and safe garbage collection (specialist, M)
- [#20](https://github.com/bilalyazicioglu/nivra/issues/20) **A15** — Improve doctor and installation troubleshooting (small, S)
- [#21](https://github.com/bilalyazicioglu/nivra/issues/21) **A16** — Polish terminal output for narrow widths and accessibility (small, S)
- [#22](https://github.com/bilalyazicioglu/nivra/issues/22) **A17** — Implement read-only current port inspection on macOS (standard, M)
- [#23](https://github.com/bilalyazicioglu/nivra/issues/23) **A18** — Add safe, explicit port-owner termination (specialist, M)
- [#24](https://github.com/bilalyazicioglu/nivra/issues/24) **A19** — Build the TUI terminal lifecycle and application shell (standard, M)
- [#25](https://github.com/bilalyazicioglu/nivra/issues/25) **A20** — Implement keyboard-driven timeline browsing in the TUI (standard, M)
- [#26](https://github.com/bilalyazicioglu/nivra/issues/26) **A21** — Add event details and baseline comparison panels to the TUI (standard, M)
- [#27](https://github.com/bilalyazicioglu/nivra/issues/27) **A22** — Add a deterministic why command for repeated success/failure checks (standard, M)

## M2 — Public preview

- [#28](https://github.com/bilalyazicioglu/nivra/issues/28) **B01** — Complete a documented daily-use alpha evaluation (specialist, M)
- [#29](https://github.com/bilalyazicioglu/nivra/issues/29) **B02** — Build reproducible preview release artifacts and checksums (standard, M)
- [#30](https://github.com/bilalyazicioglu/nivra/issues/30) **B03** — Verify clean-machine install, upgrade and uninstall flows (small, S)
- [#31](https://github.com/bilalyazicioglu/nivra/issues/31) **B04** — Produce the real demo recording and public-preview materials (small, S)
- [#33](https://github.com/bilalyazicioglu/nivra/issues/33) **B05** — Run the public-preview release gate (specialist, S)
- [#32](https://github.com/bilalyazicioglu/nivra/issues/32) **DOCSYNC** — Add a checked, manual wiki publication workflow (small, S)

## M3 — Portability and bundles

- [#34](https://github.com/bilalyazicioglu/nivra/issues/34) **C01** — Add Bash integration using the accepted capture contract (standard, M)
- [#35](https://github.com/bilalyazicioglu/nivra/issues/35) **C02** — Add Fish integration using the accepted capture contract (standard, M)
- [#36](https://github.com/bilalyazicioglu/nivra/issues/36) **C03** — Add a Linux port provider and publish a tested support matrix (standard, M)
- [#37](https://github.com/bilalyazicioglu/nivra/issues/37) **C04** — Specify a versioned portable session bundle and import threat model (specialist, M)
- [#38](https://github.com/bilalyazicioglu/nivra/issues/38) **C05** — Implement local bundle export with explicit privacy preview (standard, M)
- [#39](https://github.com/bilalyazicioglu/nivra/issues/39) **C06** — Implement safe, read-only inspection of imported bundles (specialist, M)

## M4 — Runtime timeline

- [#40](https://github.com/bilalyazicioglu/nivra/issues/40) **D01** — Design runtime observation, ownership and resource budgets (specialist, M)
- [#41](https://github.com/bilalyazicioglu/nivra/issues/41) **D02** — Implement a bounded runtime timeline pilot (specialist, L)
- [#42](https://github.com/bilalyazicioglu/nivra/issues/42) **D03** — Design and implement explicit, race-safe process restart (specialist, M)

## M5 — Optional encrypted sync

- [#43](https://github.com/bilalyazicioglu/nivra/issues/43) **E01** — Decide optional encrypted sync architecture and threat model (specialist, M)
- [#44](https://github.com/bilalyazicioglu/nivra/issues/44) **E02** — Implement the approved optional sync client and self-hosted server (specialist, L)

## M6 — Reviewed sharing

- [#45](https://github.com/bilalyazicioglu/nivra/issues/45) **F01** — Design sanitized share previews and a read-only session viewer (specialist, M)

## Readiness

M3–M6 are deferred. Verify dependency PRs and accepted contracts before assigning code. Size-L items are planning containers to split first. GitHub status labels are authoritative only after checking their prerequisite evidence; the source plan is a planning snapshot.
