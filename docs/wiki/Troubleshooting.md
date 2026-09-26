# Troubleshooting

This page describes the source alpha merged through [PR #4](https://github.com/bilalyazicioglu/nivra/pull/4). It is not a stable-release promise.

## `zsh: command not found: nivra`

`cargo build --locked` compiles `target/debug/nivra` inside the checkout. It does not put a `nivra` command on your PATH. The demo uses that local binary, which explains why the demo can work while `nivra --help` fails.

From a checkout containing the core implementation:

```sh
cargo install --path . --locked
```

Then in zsh:

```zsh
rehash
command -v nivra
nivra --version
```

Cargo normally installs under `~/.cargo/bin`. If that directory is not on your PATH, test it in the current shell with `export PATH="$HOME/.cargo/bin:$PATH"`. Custom `CARGO_HOME` or `CARGO_INSTALL_ROOT` setups may use a different directory. Only add the appropriate PATH line to your startup file after verifying it. Nivra does not edit that file for you.

Without installation, run `./target/debug/nivra --help` from the checkout.

## `mark not found in this repository`

The demo uses a temporary Git repository and its own temporary data directory, then deletes them. Its `working` mark does not become a mark in your project. Marks are also repository-scoped.

In your own Git repository:

```sh
nivra mark working
# edit files or run commands
nivra diff working now
```

## No commands recorded

Explicit capture requires `nivra run -- <program> <args>`. Automatic capture requires opting in to hooks in the current zsh session:

```zsh
eval "$(nivra init zsh)"
echo hello
nivra events
```

Check `nivra status`, paused state, leading-space commands and whether `NIVRA_DATA_DIR` points to the expected dedicated directory. The current doctor does not reliably detect every shell's installed hooks; improved diagnostics are planned, not shipped.

## Incomplete snapshots or events

An interrupted command may have no after snapshot. Large/unreadable files, unsupported paths and bounded Git queries can make observations incomplete. The alpha rejects incomplete marks and shows comparison warnings. Do not interpret an incomplete snapshot as proof that nothing changed.

## Capture is slow

The source alpha uses synchronous hooks and per-Git-query timeouts. There is not yet a total hashing/capture budget. Pause or unload the integration if it disrupts your shell, and report a reproduction using synthetic files. Performance benchmarks and a shared capture budget are tracked in the roadmap.
