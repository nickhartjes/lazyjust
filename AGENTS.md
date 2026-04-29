# AGENTS.md

Repo-wide guidance for coding agents working on lazyjust.

- Run `just ci` before pushing (fmt + lint + test + color-gate).
- Releases: `just release X.Y.Z` then `git push origin main --follow-tags`. See [RELEASE.md](RELEASE.md).
- Specs and plans live under `docs/superpowers/specs/YYYY-MM-DD-<topic>-design.md` and `docs/superpowers/plans/YYYY-MM-DD-<topic>.md`.
- Use `.worktrees/<branch-suffix>/` (gitignored) for branch isolation.
- UI chrome must use `theme.*` slots, not hardcoded `Color::X`. The `just color-gate` recipe enforces this for everything except `src/ui/session_pane.rs`.
