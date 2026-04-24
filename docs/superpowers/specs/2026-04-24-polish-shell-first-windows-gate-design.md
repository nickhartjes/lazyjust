# Shell-First Polish — Windows Gate Design

**Date:** 2026-04-24
**Status:** Draft, pending implementation plan
**Owner:** Nick Hartjes
**Depends on:** Session-module polish merge ending at `56e9ffb`

## Overview

Investigating sub-project A (polish the shell-first recipe spawn) surfaced a real bug rather than polish: `SessionManager::spawn_recipe` calls `build_unix_command` unconditionally. On Windows the resulting argv (`[$SHELL, "-i"]`, fallback `/bin/sh -i`) fails at PTY spawn — `/bin/sh` doesn't exist and most Windows boxes don't set `$SHELL`. CI can't catch it because the integration tests are gated `#![cfg(not(windows))]` and no Windows job exists in the workflow. Separately, `src/session/wrapper.rs` still carries a `WINDOWS_WRAPPER` constant that no caller references — dead code left over from an earlier aspirational Windows path that was never wired.

This spec does two small, honest things: gate the shell-first priming path with `#[cfg(unix)]`, and return a clear runtime error on Windows. It also deletes the dead `WINDOWS_WRAPPER` constant and rewrites the `wrapper.rs` module doc so readers aren't misled about Windows support. Full Windows parity — PowerShell selection, PSReadLine handling, PowerShell-flavored quoting, ConPTY specifics — is out of scope and tracked as a separate sub-project.

Idle-constant tuning (400ms quiet / 5s cap) and silent-shell hardening were considered and deferred: no observed misbehavior justifies tuning without data.

## Goals

1. Windows `cargo build` continues to compile after this change.
2. On Windows, calling `lazyjust` produces a clear runtime error explaining the platform is unsupported, rather than silently attempting a unix spawn.
3. `src/session/wrapper.rs` carries no dead code and its module doc matches reality.
4. Unix runtime behavior is byte-identical before and after.

## Non-goals

- Actually making lazyjust work on Windows (tracked as follow-up sub-project).
- Reintroducing a Windows-specific wrapper script or PowerShell helper.
- Tuning the idle-detection constants.
- Adding a Windows job to CI.
- Expanding test coverage.

---

## Behavioral contract

### Unix

- No behavior change. `cargo test` on unix passes before and after every commit in this branch.
- `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` remain clean.

### Windows

- `SessionManager::spawn_recipe` returns `Err(crate::error::Error::PtySpawn("lazyjust: Windows support not yet implemented (tracked as a separate sub-project)"))` immediately — before any PTY spawn, thread, or file is created.
- Higher layers (`do_spawn`, event loop) receive the error like any other `PtySpawn` failure and surface it via the existing error channel.
- `cargo check --target x86_64-pc-windows-gnu` succeeds; this is a manual verification step, not CI-enforced.

---

## Implementation

### 1. Gate `spawn_recipe` in `src/session/manager.rs`

Wrap the existing body in `#[cfg(unix)]` and add a `#[cfg(windows)]` early-return that is the only statement the function executes on Windows.

```rust
impl SessionManager {
    #[allow(clippy::too_many_arguments)]
    pub fn spawn_recipe(
        &mut self,
        id: SessionId,
        justfile: &Path,
        recipe: &str,
        args: &[String],
        cwd: &Path,
        rows: u16,
        cols: u16,
        log_path: PathBuf,
        tx: Sender<AppEvent>,
        log_cap: u64,
    ) -> Result<SessionMeta> {
        #[cfg(windows)]
        {
            let _ = (id, justfile, recipe, args, cwd, rows, cols, log_path, tx, log_cap);
            return Err(crate::error::Error::PtySpawn(
                "lazyjust: Windows support not yet implemented (tracked as a separate sub-project)".into(),
            ));
        }
        #[cfg(unix)]
        {
            // existing body moves here unchanged — argv build, spawn, reader, prime
            // thread, handle insert, meta return.
        }
    }
}
```

The `let _ = (...)` discard silences `unused_variables` on the Windows path without `#[allow]`. No logic change anywhere else in `manager.rs`.

### 2. Delete `WINDOWS_WRAPPER` and update `wrapper.rs` module doc

`src/session/wrapper.rs` shrinks to the unix builder only. The `//!` doc is rewritten to reflect that the file now has a single responsibility and Windows is explicitly unsupported.

```rust
//! Unix argv builder. The PTY spawns `$SHELL -i` directly; the recipe
//! itself is primed into the shell's stdin via `session::shell::prime_line`.
//! Windows support is not yet implemented (see
//! `session::manager::SessionManager::spawn_recipe` for the runtime stub).

/// Returns the argv for the PTY spawn on Unix: `[$SHELL, "-i"]` (fallback
/// `/bin/sh`). The recipe itself is not in argv — it is delivered via
/// `crate::session::shell::prime_line` written to the shell's stdin after
/// rc-file init. Parameters are retained (and underscored) for call-site
/// stability; a future Windows builder is expected to consume them.
pub fn build_unix_command(
    _justfile: &std::path::Path,
    _recipe: &str,
    _args: &[String],
) -> (Vec<String>, Vec<(String, String)>) {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
    (vec![shell, "-i".to_string()], Vec::new())
}
```

The `WINDOWS_WRAPPER` constant is removed entirely. No test or other module references it (grep confirms). When Windows support is actually implemented, that sub-project will introduce whatever it needs (likely a PowerShell-flavored shell module rather than a batch-script constant).

### 3. Dead-reference audit

After the two edits above, grep the tree for `WINDOWS_WRAPPER`. The only remaining matches should be in historical spec/plan documents under `docs/superpowers/` — those are frozen records, not live references, and are left alone.

---

## Testing

No new tests. Rationale:

- Unix behavior is byte-identical — existing test suite (2 integration tests, 9 unit tests in `session::shell`, full reducer/snapshot/other suites) continues to cover it.
- The Windows error-stub path has no runtime coverage in CI because CI runs only on unix (`ubuntu-latest`). Adding a Windows matrix to the CI workflow is explicitly out of scope.

### Manual verification during implementation

The implementer runs, in addition to the usual unix checks (`cargo build`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`):

```
rustup target add x86_64-pc-windows-gnu
cargo check --target x86_64-pc-windows-gnu
```

Expected: `Finished`. If `cargo check` fails because `portable_pty` or a transitive dep doesn't build against `x86_64-pc-windows-gnu`, fall back to `x86_64-pc-windows-msvc` (requires MSVC toolchain — skip if unavailable). The point is to confirm the cfg-gated code path compiles on a Windows target, not to produce a usable binary.

If neither Windows target compiles for reasons unrelated to this change (upstream dep issues), document that in the implementation report and continue — the cfg gating itself is auditable from the unix build by reading the diff.

---

## Risks

1. **`x86_64-pc-windows-gnu` target not installed.** Implementer must install it once via `rustup target add`; otherwise `cargo check` on that target errors with "toolchain does not contain target". No runtime impact.
2. **`portable_pty` Windows compile may fail on `gnu` target.** The crate has both ConPTY and winpty backends; gnu-target support varies. Spec accepts this as a known limitation of the manual verification step and does not require fixing it.
3. **Future Windows sub-project needs to reintroduce both pieces** — a `spawn_recipe` Windows branch and whatever platform-specific argv/quoting/priming it uses. This spec deliberately leaves no scaffolding behind; the follow-up spec starts clean.

## Rollback

Two commits (file-split Windows gate, dead-code deletion). Either reverts independently. Full revert restores `WINDOWS_WRAPPER` and removes the cfg gate; the codebase returns to the state at `56e9ffb` with no loss of data or test coverage.
