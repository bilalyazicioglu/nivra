# Architecture

The current application is a modular Rust binary. Keep the event/snapshot model small until real usage requires a workspace or daemon.

| Module | Responsibility |
| --- | --- |
| main | CLI, explicit execution, comparison and terminal-safe rendering |
| model | Session references, events, snapshots and timestamps |
| store | SQLite schema v1, short lock timeout, marks and recording controls |
| git | Bounded Git subprocesses, status and changed-file fingerprints |
| privacy | Conservative redaction and default ignored commands |
| shell | Opt-in zsh script generation |

## Capture

Privacy check → before snapshot → event start → user command → after snapshot → event completion. Raw command text arrives on stdin for shell hooks; persistence receives only the redacted string. An unfinished event remains visibly incomplete instead of pretending success.

SQLite stores sessions, events, marks and settings. Snapshots are JSON inside event/mark rows, preserving a small versioned schema. `PRAGMA user_version` gates migrations; a newer schema is rejected. Foreign keys link events to sessions. Capture uses a 50 ms database busy timeout. This alpha uses rollback journaling, not WAL.

`nivra run` gives each invocation a separate session. Automatic hooks group a shell's events by a session identifier. Timestamps are Unix wall-clock milliseconds; duration is approximate and clock adjustments may affect it.

## Git observations

Snapshots record repository root, branch, HEAD and porcelain status. NUL-delimited paths support whitespace and newlines. Changed regular files up to 8 MiB receive SHA-256 fingerprints, so two equally-sized edits with the same Git status still compare differently. Source contents and full patches are not stored.

Ignored files and submodule contents are outside capture. Non-UTF-8 filenames, oversized files and unreadable files can make a snapshot incomplete. Marks reject incomplete snapshots. Comparison reports warnings, changed worktree states and HEAD/branch transitions. It does not reconstruct committed diffs.

Git queries have individual 750 ms deadlines and a 4 MiB output cap. Hooks are synchronous and file hashing has no aggregate latency budget yet. Benchmarking and a total-budget design are required before claiming low overhead.

## Semantics

Marks use `(name, repository root)` identity. Saving the same name replaces its baseline. Comparisons list events started within the wall-clock interval whose before snapshot belongs to the same repository. Overlapping commands may span either boundary; the list is context, not attribution.

Commands are never rerun by the viewer. Terminal output escapes control characters from stored metadata. JSON output keeps structured strings for consumers.
