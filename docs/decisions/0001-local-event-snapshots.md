# ADR 0001: local event snapshots first

Status: accepted for the source alpha · 2026-09-26

## Context

Nivra needs to demonstrate a developer-state comparison before taking on a TUI, daemon or runtime monitor. Shell history alone does not distinguish a passing baseline from later working-tree edits.

## Decision

Use one Rust binary, SQLite and a before/event/after model. Start with explicit execution and opt-in zsh hooks. Store Git metadata and changed-file fingerprints, not patches or terminal output. Keep raw command strings out of persistence by redacting first.

## Consequences

This delivers a repeatable local workflow and simple distribution path. Synchronous snapshots add measurable latency and do not capture every editor change. Fingerprints reveal that content changed, but cannot reconstruct it. Committed-file comparisons, total capture budgets, retention and runtime observations require further work.

These tradeoffs are documented rather than hidden behind a polished demo.
