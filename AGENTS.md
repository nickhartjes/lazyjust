# AGENTS.md

Repo-wide guidance for coding agents working on lazyjust.

- Run `just ci` before pushing (fmt + lint + test + color-gate).
- Releases: commit **and push** the `CHANGELOG.md` entry first (the recipe
  requires `HEAD == origin/main`), then `just release X.Y.Z`, then
  `git push origin main --follow-tags`. See [RELEASE.md](RELEASE.md).
- Specs and plans live under `docs/superpowers/specs/YYYY-MM-DD-<topic>-design.md` and `docs/superpowers/plans/YYYY-MM-DD-<topic>.md`.
- Use `.worktrees/<branch-suffix>/` (gitignored) for branch isolation.
- `nix develop` writes a `flake.lock` (this repo does not track one) and
  **stages it**. `git commit` commits the index, not just the paths you named
  to `git add`, so it can ride along in an unrelated commit. Clear it with
  `git reset -- flake.lock` before committing.
- Tests that mutate process-global state (env vars) must serialize themselves
  behind a module-level mutex — see `tests/config_loader.rs` and
  `src/ui/path_display.rs`. `just test` and CI both run at default
  parallelism; only `nix/checks.nix` passes `--test-threads=1`, so never rely
  on single-threaded execution. Getting this wrong produces a flake that
  passes locally and fails intermittently on Windows CI.
- UI chrome must use `theme.*` slots, not hardcoded `Color::X`. The `just color-gate` recipe enforces this for everything except `src/ui/session_pane.rs`.
