# Public-preview preparation

The first demo must be reproducible with `cargo build --locked && sh scripts/demo.sh`. `assets/demo.svg` is rendered from actual demo output with a shortened temporary path; it is not a mock TUI.

Before posting to Reddit:

- Capture a 15–20 second terminal video of the real workflow, without private paths/history.
- Validate installation on a clean macOS machine and document measured capture overhead.
- Link the exact source/release revision; disclose early-alpha limits.
- Read the target subreddit's current self-promotion rules before posting.
- Ask for concrete feedback on snapshot boundaries and useful state dimensions.

Draft (not posted):

> I built Nivra because I kept asking “this worked twenty minutes ago—what changed?” It records command boundaries and Git state locally, lets me mark a working baseline, and compares that baseline with now. The short demo shows a real passing check, an edit, a failing check, and the intervening changes. It's an early Rust CLI alpha: no account, no telemetry, no output recording, and no cloud dependency. It reports observations, not the cause of a bug. I'd love feedback on which state changes would help in your actual debugging sessions.

Do not claim TUI, process lineage, port history or performance numbers before they exist. Do not manufacture stars, users, testimonials or community activity.
