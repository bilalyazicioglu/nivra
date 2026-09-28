# Changelog

## Unreleased

- `nivra diff` groups output into Commands, Git and State, shows per-file and total line counts when comparing with `now`, and says explicitly when HEAD and branch are unchanged. Counts are exact since a mark only for files that were clean at the mark with HEAD unchanged; otherwise they are labelled "vs HEAD". JSON adds `lines`, `head_changed` and `branch_changed`.

## 0.1.0 — 2026-09-26

- Initial Rust CLI with explicit command recording and opt-in zsh hooks.
- Local SQLite sessions, events, before/after Git snapshots, repository-scoped marks.
- State comparison with fingerprints for already-dirty files and recorded commands.
- Global pause/resume, ignore-next, heuristic full-command redaction and private Unix storage permissions.
- Timeline text/JSON output, status, doctor and an isolated reproducible demo.
- Contributor documentation, GitHub issue/PR templates, CI and wiki sources.
- Source package installation through crates.io with `cargo install nivra --locked`.

This is an early source alpha, not a stable release or compatibility promise.
