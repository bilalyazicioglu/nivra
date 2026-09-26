# Security policy

Nivra is an early alpha. There is no security-audited or stable release yet. Fixes target the current main branch; please include the exact version/commit in reports.

## Report privately

Use [GitHub private vulnerability reporting](https://github.com/bilalyazicioglu/nivra/security/advisories/new) when enabled. If it is unavailable, open a public issue containing only a request for a private contact channel, without exploit details or sensitive data. Maintainers will arrange a private channel. Do not attach your live database or real terminal history.

Include synthetic reproduction steps, affected versions/platforms, impact, and a proposed fix if available. Maintainers will acknowledge and investigate as availability permits; no response-time SLA is claimed.

## Current boundaries

- Redaction is heuristic. Arbitrary positional secrets and encoded credentials may not be recognized.
- Paths, branch names and fingerprints are metadata, not anonymized data.
- SQLite is not encrypted. OS account access and disk encryption remain relevant.
- Use a dedicated data directory. Unix directory/database permissions are tightened to 0700/0600.
- No automatic deletion, sync, telemetry, output capture, or network service is implemented.
- Shell hooks are opt-in and must fail open. Capture failure must never prevent a user's command.

See the [privacy guide](docs/wiki/Privacy.md) for operational controls.
