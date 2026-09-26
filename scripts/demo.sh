#!/bin/sh
# Real, repeatable demo. Uses a temporary repo and a separate temporary database.
set -eu
NIVRA_BIN=${NIVRA_BIN:-"$(pwd)/target/debug/nivra"}
case "$NIVRA_BIN" in /*) ;; *) echo 'NIVRA_BIN must be absolute' >&2; exit 1;; esac
DEMO_DIR=$(mktemp -d)
trap 'rm -rf "$DEMO_DIR"' EXIT HUP INT TERM
export NIVRA_DATA_DIR="$DEMO_DIR/data"
mkdir "$DEMO_DIR/repo"
cd "$DEMO_DIR/repo"
git init -q -b main
git config user.email demo@example.invalid
git config user.name 'Nivra demo'
printf 'working\n' > behavior.txt
git add behavior.txt
git commit -qm 'Working baseline'
printf '\n$ nivra run -- grep -q working behavior.txt\n'
"$NIVRA_BIN" run -- grep -q working behavior.txt
printf '✓ Test passed\n\n$ nivra mark working\n'
"$NIVRA_BIN" mark working
printf '\n$ printf "broken\\n" > behavior.txt\n'
"$NIVRA_BIN" run -- sh -c 'printf "broken\n" > behavior.txt'
printf '\n$ nivra run -- grep -q working behavior.txt\n'
if "$NIVRA_BIN" run -- grep -q working behavior.txt; then
    echo 'Demo expected a failure' >&2; exit 1
else
    code=$?
    [ "$code" -eq 1 ] || exit "$code"
    printf '✕ Test failed (exit 1)\n'
fi
printf '\n$ nivra diff working now\n'
"$NIVRA_BIN" diff working now
