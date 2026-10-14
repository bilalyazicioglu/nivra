# ADR 0002: snapshot, query and comparison contracts

Status: proposed · 2026-09-28 · closes A01 (#6) on maintainer acceptance

## Context

M1 work (#9–#19) will be implemented in parallel by different contributors. Without shared contracts each one would invent its own snapshot completeness flag, pagination cursor and comparison shape. This ADR fixes the small interfaces they share and records the invariant that outranks every feature.

The current implementation (schema v1, `src/model.rs`, `src/store.rs`, `src/git.rs`, `compare` in `src/main.rs`) is the starting point. Where it falls short of this contract, the gap and its owner are named below.

## Invariant: Nivra must never make the shell less reliable

If Nivra crashes, hangs up to its budget, finds its database locked, read-only, corrupt or missing, cannot run Git, or is not installed at all, the user's command still runs, keeps its exit status, and the prompt returns.

- **Fail open.** Every capture error is swallowed at the hook boundary (`scripts/nivra.zsh`: `|| _NIVRA_EVENT=''`, `|| true`, `2>/dev/null`). `nivra run` prints one line to stderr and runs the command anyway. No capture path may `exit`, `return` a different status, `set -e`, trap signals or write to the terminal.
- **Fast and bounded.** Capture work in `preexec`/`precmd` is synchronous today. Each Git query has a 750 ms deadline, but there is no total budget. Until #11 lands, overhead must be measured (#7) rather than claimed. After #11, a hook exceeding its budget records a partial snapshot instead of waiting.
- **Non-blocking storage.** SQLite busy timeout stays at 50 ms. A lock means the event is dropped or partial, never a stalled prompt.
- **Tested, not asserted.** `scripts/test-zsh.py` (#8) and the fail-open torture set enforce this in a real PTY: exit status, Ctrl-C (130), missing binary, locked/read-only/corrupt DB, missing Git, broken repository, nested shells.

A feature that cannot meet this invariant belongs outside the hook path (for example, computed at `diff` time).

## Snapshot

A snapshot is an observation of one directory at one moment.

| Field | Meaning |
| --- | --- |
| `cwd` | Directory the snapshot was taken from |
| `repo` | Worktree root from `git rev-parse --show-toplevel`; `null` outside Git or on Git failure |
| `branch` | Short symbolic ref; `null` when detached or on Git failure |
| `head` | Full commit id; `null` for an unborn branch or on Git failure |
| `files` | Path → `{status, fingerprint}` for every non-ignored changed path |
| `warning` | Human-readable note (v1: only the last warning survives) |

**Identity.** Two snapshots describe the same state when `repo`, `head` and `files` (status *and* fingerprint) are equal. `branch` is a label, not part of identity. The porcelain `XY` status distinguishes staged (`X`) from unstaged (`Y`) changes, but the fingerprint hashes worktree content only. **The index content is not captured.** Staging a file without editing it is visible only through the status letter. #18 owns adding an index identity.

**Unborn and detached HEAD.** Unborn: `head = null`, `branch` = the unborn name if `symbolic-ref` resolves it. Detached: `branch = null`, `head` set. Renderers show `(unborn)` and `(detached)` respectively; they must not confuse either with a Git failure, which also sets `warning`.

**Completeness.** A snapshot is *complete* when Git answered every query and every changed non-deleted regular file up to 8 MiB was fingerprinted. Non-UTF-8 paths, oversized, unreadable or non-regular files, Git timeouts and output caps make it *incomplete*. Marks already reject incomplete snapshots; comparisons show their warnings.

Contract change (owner #11, `src/model.rs`, `src/git.rs`): add `complete: bool` and replace the single `warning` with `warnings: [string]` so one failure no longer hides another. Deserialization must treat a v1 row without these fields as `complete = (warning == null)`, `warnings = [warning]`.

## Events and ordering

An event is a command bracketed by a *before* snapshot and, when the command finishes and capture succeeds, an *after* snapshot.

- **Stable order** is `(started_ms, rowid)`. Wall-clock milliseconds can collide and can go backwards. `rowid` is the insertion tie-breaker and is not exposed as an identifier. Event `id` (UUID v4) is the only public identifier.
- **Incomplete events** (`finished_ms = null`) stay visible as incomplete. They are never silently completed or deleted outside explicit retention (#19).
- **Wall-clock vs elapsed.** `started_ms`/`finished_ms` are Unix wall-clock times for display and interval queries. Durations derived from them are approximate. #13 adds `elapsed_ms` measured from a monotonic clock inside one process. Until then no output may claim precise durations.
- **Overlap.** Concurrent shells interleave. An event belongs to a comparison interval if `started_ms` falls inside it and its before snapshot is in the same `repo`. An event can therefore span a boundary. Listing it is context, never attribution.

## Read API (owner #14, `src/store.rs`)

One paginated query replaces ad-hoc SQL in `main.rs` so that the CLI, `inspect` (#15), sessions/search (#16) and the TUI (#24–#26) agree.

```rust
pub struct EventQuery {
    pub repo: Option<String>,        // before snapshot repo
    pub session: Option<String>,
    pub since_ms: Option<i64>,       // inclusive
    pub until_ms: Option<i64>,       // inclusive
    pub status: Option<ExitFilter>,  // Success | Failure | Incomplete
    pub text: Option<String>,        // substring of the redacted command
    pub order: Order,                // NewestFirst (default) | OldestFirst
    pub limit: u32,                  // 1..=500
    pub cursor: Option<Cursor>,      // opaque; encodes (started_ms, rowid)
}
pub struct Page<T> { pub items: Vec<T>, pub next: Option<Cursor> }
```

Keyset pagination only, no `OFFSET`: inserts during browsing must not duplicate or skip rows. Cursors are opaque strings in JSON output and valid only for the same query. Marks and sessions use the same `Page` type. Comparisons must read their full interval by paging, never by a silent limit.

## Comparison result (owner #17, new `src/compare.rs`)

A structured value produced once and rendered by both CLI and TUI.

```json
{
  "from": {"label": "working", "at_ms": 1790000000000, "complete": true},
  "to":   {"label": "now",     "at_ms": 1790000600000, "complete": true},
  "repo": "/home/dev/app",
  "head":   {"before": "a1b2c3d…", "after": "a1b2c3d…", "changed": false},
  "branch": {"before": "main",     "after": "main",     "changed": false},
  "files": [
    {"path": "src/main.rs", "before": "clean", "after": " M",
     "lines": {"added": 24, "removed": 7, "basis": "since_mark"}},
    {"path": "Cargo.toml", "before": " M", "after": " M",
     "lines": {"added": 3, "removed": 1, "basis": "vs_head"}}
  ],
  "commands": [
    {"id": "…", "command": "cargo add serde", "exit_code": 0, "started_ms": 1790000100000},
    {"id": "…", "command": "cargo test",      "exit_code": 101, "started_ms": 1790000500000}
  ],
  "warnings": [],
  "causality": "observations only"
}
```

**Line statistics.** Nivra does not store file contents, so it cannot diff against a mark. `lines` comes from `git diff --numstat HEAD -- <paths>`, run only at comparison time with the bounded Git helper and `--no-ext-diff`, never in a hook.

- `basis: "since_mark"` only when the file was clean at the mark and `head` is unchanged. In that case "vs HEAD" equals "since mark" exactly.
- `basis: "vs_head"` otherwise, and renderers must say so.
- `lines: null` for binary files, deleted-then-untracked paths, Git failure or timeout.
- Untracked files count all lines as added.

**Worked examples.**

1. *Unborn HEAD.* Both `head` values are `null`, `changed: false`. There is no HEAD to diff against, so `lines` is `null` and every file shows its status only.
2. *Incomplete capture.* `to.complete: false`, `warnings` carries the reason. Files that could not be fingerprinted appear with `after.fingerprint = null`. They are reported as "unknown", not "unchanged".
3. *Same-millisecond events.* Two events with equal `started_ms` both appear, ordered by insertion.
4. *Concurrent shells.* A command from shell B starting inside the interval is listed even though shell A made the edit. The output keeps the line "Observed changes, not proof of causation."
5. *HEAD moved.* `head.changed: true`. Every `lines.basis` is `vs_head`, and committed changes are not expanded until #18.

## JSON compatibility and migrations

- **Additive by default.** CLI JSON may gain fields in minor releases. Consumers must ignore unknown fields. Removing or retyping a field requires a major version, or a release note plus a `schema` field on the top-level object.
- **Stored JSON** (`before_json`, `after_json`, `snapshot_json`): new fields use serde defaults so v1 rows stay readable. Stored JSON is never rewritten in place just to add a field.
- **SQLite schema.** `PRAGMA user_version` gates migrations. Each migration runs in `BEGIN IMMEDIATE` and is idempotent under concurrent first-open (owner #10). A database from a newer Nivra is refused read/write. Hooks treat that refusal like any other capture failure (fail open).

## Follow-up owners

| Issue | Contract piece | Files |
| --- | --- | --- |
| #10 A05 | concurrent-safe migrations | `src/store.rs` |
| #11 A06 | total budget, `complete`, `warnings[]` | `src/git.rs`, `src/model.rs`, `scripts/nivra.zsh` |
| #12 A07 | exclusions before capture | `src/privacy.rs` |
| #13 A08 | `elapsed_ms` | `src/model.rs`, `src/main.rs`, `src/store.rs` |
| #14 A09 | `EventQuery`, `Page`, cursor | `src/store.rs` |
| #15 A10, #16 A11 | consumers of #14 | `src/main.rs` |
| #17 A12 | comparison struct + renderer split | new `src/compare.rs`, `src/main.rs` |
| #18 A13 | index identity, committed diffs | `src/git.rs`, `src/compare.rs` |
| #19 A14 | retention of incomplete events | `src/store.rs` |

Consumers start implementation only after the maintainer accepts this ADR.

## Consequences

Contributors can build against fixed shapes, and the reliability invariant is testable rather than aspirational. The cost: line counts are honest but sometimes "vs HEAD" rather than "since mark", staged-only changes remain partly invisible until #18, and the v1 single `warning` stays lossy until #11.
