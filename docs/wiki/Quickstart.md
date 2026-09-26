# Quickstart

Build from source with a current stable Rust toolchain and Git:

```sh
git clone https://github.com/bilalyazicioglu/nivra.git
cd nivra
cargo build --locked
sh scripts/demo.sh
cargo install --path . --locked
rehash  # zsh: refresh command lookup
nivra --version
```

`cargo build` alone does not install the `nivra` command. The demo uses temporary history; its marks do not carry into your project. See [Troubleshooting](Troubleshooting.md) if installation or capture is unclear.

In your Git project, run a passing test through `nivra run -- <command>`, save `nivra mark working`, edit a file, run the test again and inspect `nivra diff working now`.

`nivra run` accepts a program and its arguments, not a shell expression. For redirections/pipelines, use `nivra run -- sh -c '...'`. The command's stdin/stdout/stderr remain attached to your terminal.

For automatic capture in a zsh session:

```zsh
eval "$(nivra init zsh)"
echo hello
false
nivra events
```

The integration uses preexec/precmd hooks. Re-evaluating the init script does not duplicate them. Capture failures are suppressed in the hooks. Explicit runs print capture warnings but still execute the requested program.

To unload the hooks from the current shell:

```zsh
add-zsh-hook -d preexec _nivra_preexec
add-zsh-hook -d precmd _nivra_precmd
```

Remove the init line from `.zshrc` if you added it. `nivra pause` leaves the hooks installed but skips new capture. Pause and ignore-next apply globally across shells.
