# Changelog

## Unreleased

- `nivra ports [--json]` lists listening TCP ports with address, PID, process and working directory on macOS. Read-only; unavailable fields show as `—`.

## 0.1.0 — 2026-09-26

- Initial Rust CLI with explicit command recording and opt-in zsh hooks.
- Local SQLite sessions, events, before/after Git snapshots, repository-scoped marks.
- State comparison with fingerprints for already-dirty files and recorded commands.
- Global pause/resume, ignore-next, heuristic full-command redaction and private Unix storage permissions.
- Timeline text/JSON output, status, doctor and an isolated reproducible demo.
- Contributor documentation, GitHub issue/PR templates, CI and wiki sources.
- Source package installation through crates.io with `cargo install nivra --locked`.

This is an early source alpha, not a stable release or compatibility promise.
