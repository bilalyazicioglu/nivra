# Development

Read [CONTRIBUTING.md](https://github.com/bilalyazicioglu/nivra/blob/main/CONTRIBUTING.md) and [the milestone plan](https://github.com/bilalyazicioglu/nivra/blob/main/docs/PLAN.md).

Before opening a PR:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
cargo build --locked
sh scripts/demo.sh
```

CI repeats these checks on macOS and Linux. zsh is required for full shell integration coverage. Tests should cover behavior, including failure modes, rather than internal implementation details.

## Release gates

- All required CI jobs pass; breaking behavior documented.
- Clean-machine installation and real interactive-shell smoke test.
- Review capture overhead, redaction fixtures, storage permissions and partial-event handling.
- Reproducible checksummed artifacts and exact source tag before advertising downloads.
- Changelog, supported platforms and known limits reflect the released binary.

GitHub Wiki pages are sourced in `docs/wiki`. Review docs changes in a normal PR; publish the reviewed pages to the wiki repository afterwards. Do not silently diverge the wiki from the code revision it documents.
