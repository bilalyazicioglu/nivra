# Privacy

Nivra is local-first, but local does not mean secret-free.

## Stored

Command text after heuristic filtering; timestamps; exit status; working directory; repository root; branch; HEAD; changed filenames/status; content fingerprints. Marks save the same snapshot metadata. Paths are not anonymized.

## Not recorded

Terminal stdout/stderr, environment dumps, source file bodies, full patches, network traffic, or telemetry. Git status still observes names of untracked files unless Git ignores them.

## Controls

- `nivra pause` / `nivra resume`: machine-wide capture switch for this data directory.
- `nivra ignore-next`: consume the next eligible capture across shells, not necessarily the shell where it was requested.
- Leading-space commands and commands beginning with `pass`, `security`, or `ssh-add` are skipped by default.
- Common credential terms cause the entire command to become `[private command redacted]` before SQLite insertion.

Filtering is conservative and incomplete. Arbitrary positional secrets, unusual credential names and encoded values can evade it. A command skipped for privacy does not consume ignore-next. Pause sensitive workflows proactively. Existing records are unaffected by pause.

The database is unencrypted. Use OS account controls and disk encryption. Unix directory and database permissions are 0700 and 0600. `NIVRA_DATA_DIR` must name a dedicated directory because Nivra tightens its permissions. Do not point it at your home or a shared directory.

There is no automatic retention or export yet. To remove history, unload the hooks in all shells, stop active Nivra commands, and remove the dedicated Nivra data directory after checking its path. Uninstalling the binary alone does not remove history.

Never post a live database or real shell history to GitHub. Reproduce problems in a temporary repository with synthetic values.
