# GitHub operating model

## Repository baseline

- Public code with MIT license, readable README and reproducible demo.
- Issue forms for bugs and features; clear PR review template.
- CI on macOS and Linux with read-only workflow permissions.
- Weekly Rust dependency updates and monthly action updates.
- Wiki source reviewed alongside code under `docs/wiki`.

## Triage labels

Use GitHub's `bug`, `enhancement`, `documentation`, `good first issue` and `help wanted`, plus `status: needs-triage`, `area: capture`, `area: privacy`, `area: ux`, `area: docs`. Apply beginner labels only to scoped tasks with a concrete acceptance test.

## Milestones

- **M0 · Core proof**: real capture → baseline → edit → failure → comparison; repository foundation.
- **M1 · Daily-use alpha**: capture budgets, interactive shell coverage, retention/config, timeline UX.
- **M2 · Public preview**: clean-machine install, release artifacts, measured performance, actual screencast.

## Review policy

Focused issue and branch, linked PR, meaningful validation, maintainer review, squash merge. Solo-maintained work includes explicit self-review. No fake reviewer approvals. Keep the default branch deployable/buildable.

After the first CI run, configure branch protection against the actual reported job names. Restrict force pushes and require passing checks. Requiring another maintainer's approval is appropriate only when another maintainer exists; do not create an unmergeable solo repository.

## Wiki publishing

`docs/wiki` is the canonical source. GitHub wikis use a separate Git repository. If the wiki has never been initialized, create its Home page through GitHub once, then clone `https://github.com/bilalyazicioglu/nivra.wiki.git`. Copy the reviewed Markdown pages, rewrite relative `.md` page links to wiki slugs, commit and push. Keep publication manual until the first main-branch merge; no write-token workflow is needed for this alpha.
