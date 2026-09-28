# Development

Read [CONTRIBUTING.md](https://github.com/bilalyazicioglu/nivra/blob/main/CONTRIBUTING.md) and [the milestone plan](https://github.com/bilalyazicioglu/nivra/blob/main/docs/PLAN.md).

Before opening a PR:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
cargo build --locked
sh scripts/demo.sh
python3 scripts/test-zsh.py
```

CI repeats these checks on macOS and Linux. zsh is required for full shell integration coverage. Tests should cover behavior, including failure modes, rather than internal implementation details.

## Capture overhead

Measure before optimizing. `cargo build --release --locked && python3 scripts/bench.py` times startup, the Git queries alone, and hook start/end in a non-Git directory, a small clean repository and a large dirty repository. It uses temporary fixtures and an isolated data directory and writes results to `docs/benchmarks/`. See the [latest baseline](https://github.com/bilalyazicioglu/nivra/blob/main/docs/benchmarks/2026-09-28-darwin.md). CI enforces no thresholds.

## Release gates

- All required CI jobs pass; breaking behavior documented.
- Clean-machine installation and real interactive-shell smoke test.
- Review capture overhead, redaction fixtures, storage permissions and partial-event handling.
- Reproducible checksummed artifacts and exact source tag before advertising downloads.
- Changelog, supported platforms and known limits reflect the released binary.

GitHub Wiki pages are sourced in `docs/wiki`. Review docs changes in a normal PR; publish the reviewed pages to the wiki repository afterwards. Do not silently diverge the wiki from the code revision it documents.

For agent assignments read [Agent workflow](Agent-Workflow.md). Planning edits run `python3 scripts/check-plan.py`. Verify each issue's dependencies against merged commits before implementation.
