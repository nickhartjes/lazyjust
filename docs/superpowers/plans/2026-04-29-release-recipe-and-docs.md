# Release Recipe + Docs Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a `just release VERSION` recipe that performs every local release step (preflight, version bump, lock refresh, changelog from git log, commit, tag) and stops short of the network push, plus a short `RELEASE.md` and a new `AGENTS.md`.

**Architecture:** Three independent additions. (1) A new `release VERSION` recipe in the project `justfile` invoked manually by humans or agents. (2) A short `RELEASE.md` at the repo root explaining when/how to release. (3) A new `AGENTS.md` at the repo root with repo-wide agent rules including a release pointer.

**Tech Stack:** `just` (justfile recipe), bash/`sed`/`git`/`cargo` from the recipe, plain markdown.

**Spec:** `docs/superpowers/specs/2026-04-29-release-recipe-and-docs-design.md`

---

## File Structure

- Modify: `justfile` — add `release VERSION` recipe at the bottom (keep existing recipes untouched).
- Create: `RELEASE.md` (root) — release process for humans.
- Create: `AGENTS.md` (root) — repo-wide agent rules.

No source code is touched. No tests added (justfile recipes are not unit-tested in this repo; the spec's "Testing" section accepts manual verification).

---

### Task 1: Add `release VERSION` recipe to the justfile

**Files:**
- Modify: `justfile` (project root) — append the new recipe.

- [ ] **Step 1: Append the `release` recipe**

Open `justfile` and append, at the end of the file, after the existing `ci: fmt lint test color-gate` line:

```just

# Cut a release: bump Cargo.toml, refresh Cargo.lock, build a changelog
# from `git log <last-tag>..HEAD --oneline`, commit as `release: vVERSION`,
# and tag `vVERSION`. Does NOT push — push manually with
# `git push origin main --follow-tags`.
release VERSION:
    #!/usr/bin/env bash
    set -euo pipefail

    # 1. Validate VERSION shape.
    if ! [[ "{{VERSION}}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
        echo "release: VERSION must be MAJOR.MINOR.PATCH (got: {{VERSION}})" >&2
        exit 1
    fi

    # 2. Verify clean main, up-to-date with origin.
    branch="$(git rev-parse --abbrev-ref HEAD)"
    if [[ "$branch" != "main" ]]; then
        echo "release: must run on main (currently on: $branch)" >&2
        exit 1
    fi
    if [[ -n "$(git status --porcelain)" ]]; then
        echo "release: working tree is dirty; commit or stash first" >&2
        git status --short
        exit 1
    fi
    git fetch --quiet origin main
    if [[ "$(git rev-parse HEAD)" != "$(git rev-parse origin/main)" ]]; then
        echo "release: HEAD is not equal to origin/main; pull/push first" >&2
        echo "  local:  $(git rev-parse HEAD)" >&2
        echo "  origin: $(git rev-parse origin/main)" >&2
        exit 1
    fi

    # 3. Preflight: read-only checks (do not auto-rewrite the tree).
    cargo fmt --all -- --check
    cargo clippy --all-targets -- -D warnings
    cargo test --all-targets
    just color-gate

    # 4. Bump Cargo.toml [package] version. BSD/macOS sed form. Linux
    #    contributors with GNU sed must drop the empty backup arg.
    sed -i '' -E "s/^version = \"[0-9]+\.[0-9]+\.[0-9]+\"$/version = \"{{VERSION}}\"/" Cargo.toml
    if ! grep -q "^version = \"{{VERSION}}\"$" Cargo.toml; then
        echo "release: failed to bump Cargo.toml version" >&2
        exit 1
    fi

    # 5. Refresh Cargo.lock by running a build.
    cargo build --quiet

    # 6. Build the changelog body from `git log <last-tag>..HEAD`.
    last_tag="$(git describe --tags --abbrev=0 2>/dev/null || true)"
    if [[ -n "$last_tag" ]]; then
        body="$(git log "$last_tag..HEAD" --oneline | sed 's/^/- /')"
        if [[ -z "$body" ]]; then
            body="No commits since $last_tag (re-tag of an existing commit?)."
        fi
    else
        body="Initial release."
    fi

    # 7. Commit.
    git add Cargo.toml Cargo.lock
    git commit -m "release: v{{VERSION}}" -m "$body"

    # 8. Tag.
    git tag "v{{VERSION}}"

    # 9. Tell the human how to push.
    echo
    echo "Done. Commit and tag created locally."
    echo "Push with: git push origin main --follow-tags"
```

- [ ] **Step 2: Sanity-check the recipe parses**

Run: `just --list`
Expected: the existing recipes plus a new `release VERSION` line.

If `just --list` errors out, the recipe syntax is broken — fix and rerun.

- [ ] **Step 3: Smoke-test the validation guards (no real release)**

These checks should each abort early with a clear message and leave no
mutations behind. Run them from a clean main:

```bash
just release foo            # rejects bad VERSION shape
just release 1.2.3.4        # rejects bad VERSION shape
git checkout -b temp-branch # simulate wrong-branch guard
just release 0.1.4          # aborts: not on main
git checkout main
git branch -D temp-branch
echo " " >> README.md       # dirty tree
just release 0.1.4          # aborts: dirty tree
git checkout -- README.md
```

Expected after the sequence: each invocation prints the matching guard
message and exits non-zero, the working tree is clean, no commits added,
no tags added (`git tag -l v0.1.4` empty).

- [ ] **Step 4: Commit**

```bash
git add justfile
git commit -m "$(cat <<'EOF'
chore(justfile): add release VERSION recipe

Local-only release flow: validate version shape, verify clean main +
up-to-date with origin, run fmt --check / clippy / test / color-gate,
bump Cargo.toml [package] version, refresh Cargo.lock, build the
changelog body from `git log <last-tag>..HEAD --oneline`, commit
`release: vVERSION`, and tag `vVERSION`. Does not push — that remains
the explicit human step.
EOF
)"
```

---

### Task 2: Create `RELEASE.md`

**Files:**
- Create: `RELEASE.md` (project root).

- [ ] **Step 1: Write the file**

Create `RELEASE.md` at the repo root with exactly the following content:

```markdown
# Releasing lazyjust

## When to release

After a meaningful merge to `main`. No fixed cadence.

## How to release

1. `just release X.Y.Z`
2. Inspect the new commit and tag with `git show HEAD` and `git tag -l vX.Y.Z`.
3. `git push origin main --follow-tags`

The recipe runs `cargo fmt --check`, `cargo clippy -- -D warnings`,
`cargo test --all-targets`, and `just color-gate` first; bumps the
`[package]` version in `Cargo.toml`; refreshes `Cargo.lock`; builds the
changelog body from `git log <previous-tag>..HEAD --oneline`; commits as
`release: vX.Y.Z`; and tags `vX.Y.Z`. It never pushes.

If you want to abandon a local release:

```bash
git tag -d vX.Y.Z
git reset --hard HEAD~1
```

## What CI does on the tag push

`.github/workflows/release.yml` triggers on `v*` tag pushes. It re-runs
`cargo test --all-targets`, then builds and uploads release artifacts
(per-platform binaries plus `.sha256` checksums) to the GitHub Release.

Currently NOT automated:

- `cargo publish` to crates.io.
- Homebrew tap formula bump.

These are manual until the corresponding follow-ups land.
```

- [ ] **Step 2: Verify the file lints clean as markdown**

The repo has no markdown linter wired in, so the only check is visual.
Run: `head -1 RELEASE.md`
Expected: `# Releasing lazyjust`.

- [ ] **Step 3: Commit**

```bash
git add RELEASE.md
git commit -m "$(cat <<'EOF'
docs: add RELEASE.md

Document when to release, the two-command flow (`just release X.Y.Z`
then `git push origin main --follow-tags`), and what CI does on the
tag push. Calls out the still-manual crates.io and Homebrew steps.
EOF
)"
```

---

### Task 3: Create `AGENTS.md`

**Files:**
- Create: `AGENTS.md` (project root).

- [ ] **Step 1: Write the file**

Create `AGENTS.md` at the repo root with exactly the following content:

```markdown
# AGENTS.md

Repo-wide guidance for coding agents working on lazyjust.

- Run `just ci` before pushing (fmt + lint + test + color-gate).
- Releases: `just release X.Y.Z` then `git push origin main --follow-tags`. See [RELEASE.md](RELEASE.md).
- Specs and plans live under `docs/superpowers/specs/YYYY-MM-DD-<topic>-design.md` and `docs/superpowers/plans/YYYY-MM-DD-<topic>.md`.
- Use `.worktrees/<branch-suffix>/` (gitignored) for branch isolation.
- UI chrome must use `theme.*` slots, not hardcoded `Color::X`. The `just color-gate` recipe enforces this for everything except `src/ui/session_pane.rs`.
```

- [ ] **Step 2: Verify the file**

Run: `head -1 AGENTS.md`
Expected: `# AGENTS.md`.

- [ ] **Step 3: Commit**

```bash
git add AGENTS.md
git commit -m "$(cat <<'EOF'
docs: add AGENTS.md with repo-wide agent rules

Capture the non-obvious conventions (just ci before push, release
flow pointer, spec/plan layout, .worktrees, color-gate rule) that
coding agents need on every session.
EOF
)"
```

---

## Verification

After Tasks 1–3 land:

- `just --list` shows the new `release` recipe alongside the existing recipes.
- `RELEASE.md` and `AGENTS.md` exist at the repo root and render cleanly on GitHub.
- The validation guards from Task 1 step 3 still abort cleanly. (Repeat the smoke test if you have time.)

A real release end-to-end is **not** part of this plan. Cut the next release with `just release 0.1.4` (or whatever the next version is) when you're ready.

## Out of scope follow-ups

- `just publish` recipe for `cargo publish`.
- Homebrew tap formula auto-bump.
- A separate `CHANGELOG.md` file.
- Pre-release / RC tag handling.
- A GNU/BSD-portable `sed` shim (only matters once Linux contributors regularly cut releases).
