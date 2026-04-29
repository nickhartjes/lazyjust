# Release recipe + docs

**Date:** 2026-04-29
**Status:** Approved (design)
**Topic:** Capture lazyjust's release process as a `just release VERSION` recipe and a short `RELEASE.md`, with a one-line pointer in a new `AGENTS.md`.

## Problem

The lazyjust release process is currently tribal knowledge: bump
`Cargo.toml` (and let `cargo build` refresh `Cargo.lock`), commit
`release: vX.Y.Z` with a body summarizing user-facing changes, push a
matching `vX.Y.Z` tag, and let `.github/workflows/release.yml` build the
artifacts. Nothing in the repo describes this. Nothing enforces that
`fmt` / `lint` / `test` are clean before the bump commit. Nothing tells
a coding agent how to drive a release.

## Goal

A repeatable, recorded release process:

- One human-runnable command `just release X.Y.Z` does every local step
  (preflight checks, version bump, lock refresh, changelog from git log,
  release commit, local tag) and stops short of the network. The push
  remains the explicit, manual safety gate.
- `RELEASE.md` documents the process for humans in three short sections:
  when to release, how to release, what CI does.
- `AGENTS.md` is the standing repo-wide rules file for coding agents
  (Claude Code, Cursor, Codex, etc.) and contains a single-line pointer
  to `RELEASE.md` plus a few other repo-wide conventions.

## Non-goals

- Auto-publish to crates.io.
- Homebrew formula bump automation.
- A separate `CHANGELOG.md` file (the release commit body is the
  authoritative changelog for this project).
- Pre-release tags (`v0.2.0-rc.1`) or any non-`vMAJOR.MINOR.PATCH`
  tag shape.
- Conventional-commit-driven automatic version inference.

## Design

### `just release VERSION` recipe

Add to the project `justfile`. Steps run sequentially; recipe aborts on
the first non-zero exit.

1. **Validate VERSION** — must match `^[0-9]+\.[0-9]+\.[0-9]+$`. If not,
   exit 1 with a clear message.
2. **Verify clean main** — current branch is `main`, working tree clean
   (`git status --porcelain` empty), HEAD is up-to-date with
   `origin/main` (no commits ahead, no commits behind).
3. **Preflight** — declare the recipe with the existing `ci` recipe as
   a dependency: `release VERSION: ci`. That implicitly runs `fmt`,
   `lint`, `test`, and `color-gate` before any release-specific step.
4. **Bump `Cargo.toml`** — replace the `version = "..."` line in the
   `[package]` block. Use a guarded `sed` (or a tiny awk fallback) so
   only the package version is touched.
5. **Refresh `Cargo.lock`** — `cargo build --quiet` (the lock updates
   when the package version changes).
6. **Compute changelog** — `git log $(git describe --tags --abbrev=0)..HEAD --oneline`,
   each line bulletized. If `git describe` fails (no previous tag), the
   body is just "Initial release."
7. **Commit** — `git add Cargo.toml Cargo.lock && git commit -m
   "release: vVERSION" -m "<bullet list>"` via HEREDOC.
8. **Tag** — `git tag vVERSION` against the new commit.
9. **Print push instruction** — `echo "Done. Push with: git push origin main --follow-tags"`.

Network actions (push) are deliberately out of the recipe.

### `RELEASE.md`

Three short sections:

```markdown
# Releasing lazyjust

## When to release

After a meaningful merge to `main`. No fixed cadence.

## How to release

1. `just release X.Y.Z`
2. Inspect the commit and tag with `git show HEAD` and `git tag -l vX.Y.Z`.
3. `git push origin main --follow-tags`

The recipe runs `just ci` (fmt + lint + test + color-gate) first, bumps
`Cargo.toml`, refreshes `Cargo.lock`, builds the changelog body from
`git log <previous-tag>..HEAD --oneline`, commits as `release: vX.Y.Z`,
and tags `vX.Y.Z`. It never pushes.

## What CI does on the tag push

`.github/workflows/release.yml` triggers on `v*` tag pushes. It runs the
test suite once more, then builds and uploads release artifacts (binaries
+ `.sha256` per platform tarball) to the GitHub Release.

Currently NOT automated:
- crates.io publish (`cargo publish`).
- Homebrew tap formula bump.

These remain manual until the corresponding follow-ups land.
```

### `AGENTS.md`

```markdown
# AGENTS.md

Repo-wide guidance for coding agents working on lazyjust.

- Run `just ci` before pushing (fmt + lint + test + color-gate).
- Releases: `just release X.Y.Z` then `git push origin main --follow-tags`. See [RELEASE.md](RELEASE.md).
- Specs and plans live under `docs/superpowers/specs/YYYY-MM-DD-<topic>-design.md` and `docs/superpowers/plans/YYYY-MM-DD-<topic>.md`.
- Use `.worktrees/<branch-suffix>/` (gitignored) for branch isolation.
- UI chrome must use `theme.*` slots, not hardcoded `Color::X`. The `just color-gate` recipe enforces this for everything except `src/ui/session_pane.rs`.
```

## Testing

The recipe itself is shell scripting; testing it is best done by:

- Running `just release 0.1.4` against a clean local main, verifying the
  commit + tag look right, and dropping them with `git reset --hard HEAD~1
  && git tag -d v0.1.4` if anything is off.
- A dry-run option is **not** in scope. The "verify clean main" guard +
  recipe-only-no-push design make a true dry run unnecessary; if anything
  goes wrong, the local commit + tag can be undone without remote impact.

No automated test is added. Justfile recipes are not unit-tested in this
repo.

## Risks

- **`sed -i` portability.** macOS uses BSD `sed`, which requires `sed -i ''`;
  GNU `sed` requires `sed -i`. Use the BSD form (this repo's primary
  development is macOS) and add a comment noting the risk for Linux
  contributors. If the project later picks up regular Linux development on
  the recipe, switch to a small `awk -i inplace` or python-based
  in-place edit.
- **First-ever release.** `git describe --tags --abbrev=0` exits non-zero
  if no tags exist. The recipe handles this with a `||` fallback that
  yields the literal `Initial release.` body.
- **Pushing the wrong commit** is the gap the manual push step is meant
  to cover. The recipe's printed instruction names the exact command;
  human runs it deliberately.
- **Concurrent edits.** If a contributor pushes to `origin/main` between
  the recipe's "verify up-to-date" check and the human's `git push`, the
  push fails. They re-run the recipe after `git pull --ff-only`. No data
  loss.

## Out of scope follow-ups

- crates.io publish step (a `just publish` recipe that gates on a fresh
  release commit being on HEAD).
- Homebrew tap formula auto-bump on tag push.
- A separate `CHANGELOG.md` if the project later needs one.
- Pre-release / RC tag handling.
