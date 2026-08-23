# Releasing lazyjust

## When to release

After a meaningful merge to `main`. No fixed cadence.

## How to release

1. Add a `[X.Y.Z] - YYYY-MM-DD` section to `CHANGELOG.md` (with the
   matching `[X.Y.Z]: …compare…` link at the bottom). The recipe does
   not edit `CHANGELOG.md`.
2. Commit that entry **and push it**. The recipe refuses to run unless
   `HEAD` equals `origin/main`, so a local-only changelog commit aborts it
   before any of the real work happens. This is the easy one to get wrong:
   the final push at step 5 is *not* the first push of the release.
3. `just release X.Y.Z`
4. Inspect the new commit and tag with `git show HEAD` and `git tag -l vX.Y.Z`.
5. `git push origin main --follow-tags`

## What the recipe refuses to do

`just release` aborts, before touching anything, unless all of these hold:

- `X.Y.Z` is literally `MAJOR.MINOR.PATCH`.
- You are on `main`.
- The working tree is clean.
- `HEAD` equals `origin/main`. The recipe runs `git fetch --quiet origin
  main` itself, so a stale local ref will not fool it — see step 2 above.
- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
  `cargo test --all-targets`, and `just color-gate` all pass.

Those last checks need `rustfmt` and `clippy` on the *active* toolchain. A
Nix-profile or system `cargo` typically ships neither, and the failure is
`error: no such command: clippy` rather than anything about the release. Fix
it with `mise install` (the `mise.toml` toolchain declares both components),
or sidestep it with `nix develop -c just release X.Y.Z`.

The recipe runs `cargo fmt --check`, `cargo clippy -- -D warnings`,
`cargo test --all-targets`, and `just color-gate` first; bumps the
`[package]` version in `Cargo.toml`; refreshes `Cargo.lock`; builds the
changelog body from `git log <previous-tag>..HEAD --oneline`; commits as
`release: vX.Y.Z`; and creates an annotated tag `vX.Y.Z` carrying that
same changelog body. The annotated form is required so contributors with
`tag.gpgsign=true` or `tag.forceSignAnnotated=true` in their git config
get a signed tag instead of a recipe abort. It never pushes.

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
