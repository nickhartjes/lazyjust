# Session-Module Polish — Design Specification

**Date:** 2026-04-24
**Status:** Draft, pending implementation plan
**Owner:** Nick Hartjes
**Depends on:** Merge commit `b9a8461` (shell-first recipe spawn)

## Overview

The shell-first-spawn branch merged with a pile of reviewer-flagged follow-ups: undocumented `pub` helpers on the shell-injection boundary, a single file (`src/session/wrapper.rs`) holding two unrelated concerns, a `SessionMeta.command_line` value that now contradicts what actually runs, and two test sites that each call `std::env::set_var("SHELL", …)` — a latent parallel-test footgun. This spec closes those items as a cohesive "polish the session module" pass: file split, documentation, accurate comments, race-free test setup. Zero behavior change; all tests stay green before and after.

## Goals

1. `src/session/` splits the shell-string concern from the platform-argv concern. Each file has one clear responsibility.
2. Every `pub` function on the trust boundary (`shell_quote`, `prime_line`, `build_unix_command`) carries a rustdoc comment that names its role, its security contract where applicable, and its caveats.
3. `SessionMeta.command_line` is unambiguous — no future reader assumes the string is what actually spawned.
4. `tests/session_integration.rs` sets up its `SHELL` override exactly once, regardless of parallel-test interleaving.

## Non-goals

- Any functional change to shell-first priming, argv construction, or the OSC exit-code path.
- Windows parity. `build_windows_command` is still deferred (separate spec).
- Test coverage expansion.
- Renaming `wrapper.rs` or collapsing files beyond the single split below.

---

## Behavioral contract

No observable behavior change. `cargo test` green before the branch lands, green after each intermediate commit, green at the end. `cargo clippy --all-targets -- -D warnings` clean. `cargo fmt --check` clean.

---

## Implementation

### 1. New file: `src/session/shell.rs`

Move `shell_quote` and `prime_line` (and their `#[cfg(test)] mod tests`) verbatim from `src/session/wrapper.rs`. Add a module-level doc:

```rust
//! Shell-string construction primitives used by the session layer.
//!
//! `shell_quote` returns POSIX single-quote-escaped input; `prime_line`
//! composes the full command line (plus OSC exit-code marker) fed into
//! an interactive shell's stdin. Both are pure and platform-agnostic.
```

### 2. Trim `src/session/wrapper.rs`

After the move, the file holds only:

```rust
//! Platform-specific argv builders. On Unix the PTY spawns `$SHELL -i`
//! directly; the recipe itself is primed via `session::shell::prime_line`.
//! Windows still uses a wrapper batch script.

pub const WINDOWS_WRAPPER: &str = r#" ... "#; // unchanged

/// Returns the argv for the PTY spawn on Unix: `[$SHELL, "-i"]` (fallback
/// `/bin/sh`). The recipe itself is not in argv — it is delivered via
/// `shell::prime_line` written to the shell's stdin after rc-file init.
/// Parameters are kept for signature stability with the (future) Windows
/// builder.
pub fn build_unix_command(
    _justfile: &std::path::Path,
    _recipe: &str,
    _args: &[String],
) -> (Vec<String>, Vec<(String, String)>) {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
    (vec![shell, "-i".to_string()], Vec::new())
}
```

### 3. `src/session/mod.rs`

Add `pub mod shell;` alongside the existing `pub mod wrapper;`. Alphabetical order.

### 4. Rustdoc on `shell::shell_quote`

```rust
/// POSIX single-quote escape. Returns a valid single-quoted shell word
/// whose expansion equals `s` byte-for-byte under any POSIX `sh`,
/// including `$`, backticks, `\`, newlines, and UTF-8.
///
/// Security: this is the only quoting layer between attacker-controllable
/// input (recipe name, justfile path, positional args) and a shell `eval`
/// context. Do not replace with `$'…'` (bash-only) or backslash-escaping
/// (content-dependent) — POSIX single-quote is the one form where no
/// interior character has meaning.
///
/// Caveat: a NUL byte in `s` is preserved in the returned `String` but
/// most shells truncate at NUL when the word crosses `execve` / PTY stdin.
pub fn shell_quote(s: &str) -> String { ... }
```

### 5. Rustdoc on `shell::prime_line`

```rust
/// Builds the line fed into an interactive shell's stdin to run a recipe.
/// Every user-controlled value (justfile path, recipe name, each arg) is
/// `shell_quote`'d. The trailing `printf` emits the `LazyjustDone=%d` OSC
/// marker parsed by `session::osc::scan_done_marker`.
pub fn prime_line(
    justfile: &std::path::Path,
    recipe: &str,
    args: &[String],
) -> String { ... }
```

### 6. `SessionMeta.command_line` clarification

In `src/app/types.rs`, add a doc comment on the field:

```rust
pub struct SessionMeta {
    pub id: SessionId,
    pub recipe_name: String,
    /// Human-readable recipe invocation (`just --justfile <p> <recipe> <args>`).
    /// Informational only — the actual PTY argv is `$SHELL -i` and the recipe
    /// is delivered via `session::shell::prime_line` on stdin.
    pub command_line: String,
    pub status: Status,
    pub unread: bool,
    pub started_at: Instant,
    pub log_path: PathBuf,
}
```

In `src/session/manager.rs`, add a leading comment at the `format!` site:

```rust
// Informational string only; see SessionMeta::command_line doc.
let command_line = format!(
    "just --justfile {} {} {}",
    justfile.display(),
    recipe,
    args.join(" ")
);
```

### 7. Test `SHELL` setup via `std::sync::Once`

In `tests/session_integration.rs`, replace the two scattered `std::env::set_var("SHELL", "/bin/sh")` calls with a shared helper:

```rust
use std::sync::Once;

// Force every PTY spawn in this test binary to run `/bin/sh` rather than
// the developer's login shell. `Once` guarantees a single mutation even
// under `cargo test`'s parallel runner. If a future test needs a non-POSIX
// shell, drop this helper and adopt a per-test RAII guard that saves and
// restores the prior value.
static INIT_SHELL: Once = Once::new();

fn force_posix_shell() {
    INIT_SHELL.call_once(|| {
        std::env::set_var("SHELL", "/bin/sh");
    });
}
```

Each `#[test]` / `#[tokio::test]` replaces its first `set_var` line with `force_posix_shell();`.

### 8. Update callers of the moved helpers

- `src/session/manager.rs`: `super::wrapper::prime_line(…)` → `super::shell::prime_line(…)`.
- `tests/session_integration.rs`: split the import:
  ```rust
  use lazyjust::session::shell::prime_line;
  use lazyjust::session::wrapper::build_unix_command;
  ```

No other call sites.

---

## Testing

No new tests. The existing `#[cfg(test)] mod tests` in `shell_quote` / `prime_line` moves with the functions into `shell.rs` — same 9 cases, same assertions.

### Verification sequence

TDD isn't applicable to a zero-behavior refactor; commit-per-concern is the discipline. The implementation plan should land the work across four atomic commits, and after each one:

1. `cargo build` — compiles.
2. `cargo test` — full suite, every target `test result: ok`.
3. `cargo clippy --all-targets -- -D warnings` — clean.
4. `cargo fmt --check` — clean.

Suggested commit grouping:

- **Commit A — file split.** Create `src/session/shell.rs` with moved fns + tests, trim `src/session/wrapper.rs`, update `mod.rs`, update call sites in `manager.rs` and `tests/session_integration.rs`. Must be atomic: the crate won't compile until all three are consistent.
- **Commit B — rustdoc.** Add doc comments on `shell_quote`, `prime_line`, `build_unix_command`.
- **Commit C — `command_line` clarification.** Doc comment on the field + leading comment at the `format!` call site.
- **Commit D — test `Once`.** Introduce `INIT_SHELL` / `force_posix_shell()`; replace the two `set_var` call sites.

### Manual QA

None needed. Refactor only.

---

## Risks

1. **Missed call site.** Anyone importing `lazyjust::session::wrapper::{shell_quote, prime_line}` directly (external crate, example, etc.) breaks at compile time. Audit today: only `session/manager.rs` and `tests/session_integration.rs` reference these — no external consumers.
2. **`Once` behavior in `cargo test` under `--test-threads=1`.** Orthogonal — `Once` is thread-safe and idempotent regardless of parallelism. No regression.
3. **Docstring rot.** Doc comments reference other modules (`session::osc::scan_done_marker`, `session::shell::prime_line`). Rustdoc `broken_intra_doc_links` lint catches path drift automatically if the crate enables it; otherwise treat as accepted low-risk.

## Rollback

Single-commit revert per step, or a branch-level revert of the merge. All changes are local to `src/session/`, `src/app/types.rs`, and `tests/session_integration.rs`.
