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
