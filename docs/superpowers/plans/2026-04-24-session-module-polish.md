# Session-Module Polish Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close the reviewer-flagged follow-ups from the shell-first-spawn branch: split `session/wrapper.rs` into `shell.rs` + `wrapper.rs`, add rustdoc on the trust-boundary helpers, clarify `SessionMeta.command_line`, and serialize the test `SHELL` override via `std::sync::Once`.

**Architecture:** Pure refactor / documentation pass. Zero behavior change. Four independent commits, each keeps the crate compiling and every test green.

**Tech Stack:** Rust (existing `lazyjust` crate). No new dependencies.

**Spec:** `docs/superpowers/specs/2026-04-24-session-module-polish-design.md`

---

## File Structure

| File | Action | Responsibility after this branch |
|---|---|---|
| `src/session/shell.rs` | Create | Pure POSIX single-quote escape + primed command-line composer. `shell_quote`, `prime_line`, their inline unit tests. |
| `src/session/wrapper.rs` | Modify | Platform-specific argv builders. `build_unix_command`, `WINDOWS_WRAPPER`. `shell_quote` and `prime_line` moved out. |
| `src/session/mod.rs` | Modify | Add `pub mod shell;`. |
| `src/session/manager.rs` | Modify | Call site path update (`super::wrapper::prime_line` → `super::shell::prime_line`). Leading comment at `command_line` `format!` site. |
| `src/app/types.rs` | Modify | Rustdoc on `SessionMeta::command_line`. |
| `tests/session_integration.rs` | Modify | Import split (pull `prime_line` from `session::shell`). Replace two `set_var("SHELL", …)` call sites with a `force_posix_shell()` helper backed by `Once`. |

---

## Task 1: File split — extract `session/shell.rs`

**Files:**
- Create: `src/session/shell.rs`
- Modify: `src/session/wrapper.rs`
- Modify: `src/session/mod.rs`
- Modify: `src/session/manager.rs`
- Modify: `tests/session_integration.rs`

Rationale: the move is atomic — the crate will not compile until all five files are consistent (`mod.rs` must declare the new module, `wrapper.rs` must drop the moved fns, and every caller must import from the new path). Land it as one commit.

- [ ] **Step 1: Create `src/session/shell.rs` with moved functions and tests**

Write the new file with content below. It is a verbatim move of `shell_quote` + `prime_line` + the inline `#[cfg(test)] mod tests` from `wrapper.rs`, wrapped in a module doc comment.

```rust
//! Shell-string construction primitives used by the session layer.
//!
//! `shell_quote` returns POSIX single-quote-escaped input; `prime_line`
//! composes the full command line (plus OSC exit-code marker) fed into
//! an interactive shell's stdin. Both are pure and platform-agnostic.

pub fn shell_quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('\'');
    for c in s.chars() {
        if c == '\'' {
            out.push_str("'\\''");
        } else {
            out.push(c);
        }
    }
    out.push('\'');
    out
}

pub fn prime_line(justfile: &std::path::Path, recipe: &str, args: &[String]) -> String {
    let mut line = format!(
        "just --justfile {} {}",
        shell_quote(&justfile.display().to_string()),
        shell_quote(recipe),
    );
    for a in args {
        line.push(' ');
        line.push_str(&shell_quote(a));
    }
    line.push_str(" ; printf '\\033]1337;LazyjustDone=%d\\007' $?");
    line
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_quote_plain() {
        assert_eq!(shell_quote("foo"), "'foo'");
    }

    #[test]
    fn shell_quote_with_space() {
        assert_eq!(shell_quote("foo bar"), "'foo bar'");
    }

    #[test]
    fn shell_quote_with_single_quote() {
        assert_eq!(shell_quote("it's"), "'it'\\''s'");
    }

    #[test]
    fn shell_quote_with_dollar_and_paren() {
        assert_eq!(shell_quote("$(evil)"), "'$(evil)'");
    }

    #[test]
    fn shell_quote_empty() {
        assert_eq!(shell_quote(""), "''");
    }

    #[test]
    fn shell_quote_newline_preserved_literal() {
        assert_eq!(shell_quote("a\nb"), "'a\nb'");
    }

    #[test]
    fn prime_line_no_args() {
        let line = prime_line(std::path::Path::new("/p/Justfile"), "build", &[]);
        assert_eq!(
            line,
            "just --justfile '/p/Justfile' 'build' ; printf '\\033]1337;LazyjustDone=%d\\007' $?"
        );
    }

    #[test]
    fn prime_line_with_args_and_spaces() {
        let args = vec!["a b".to_string(), "x".to_string()];
        let line = prime_line(std::path::Path::new("/p/Justfile"), "build", &args);
        assert_eq!(
            line,
            "just --justfile '/p/Justfile' 'build' 'a b' 'x' ; printf '\\033]1337;LazyjustDone=%d\\007' $?"
        );
    }

    #[test]
    fn prime_line_escapes_dangerous_recipe_name() {
        let line = prime_line(std::path::Path::new("/p/Justfile"), "it's; rm -rf /", &[]);
        assert!(line.contains("'it'\\''s; rm -rf /'"));
        assert!(line.ends_with("$?"));
    }
}
```

- [ ] **Step 2: Declare the new module in `src/session/mod.rs`**

Replace the current contents of `src/session/mod.rs` with (alphabetical order, new `shell` module inserted):

```rust
pub mod manager;
pub mod osc;
pub mod pty;
pub mod reader;
pub mod retention;
pub mod shell;
pub mod wrapper;
```

- [ ] **Step 3: Trim `src/session/wrapper.rs`**

Overwrite `src/session/wrapper.rs` with the trimmed form. Everything except `WINDOWS_WRAPPER` and `build_unix_command` is removed, and a module doc is added:

```rust
//! Platform-specific argv builders. On Unix the PTY spawns `$SHELL -i`
//! directly; the recipe itself is primed via `session::shell::prime_line`.
//! Windows still uses a wrapper batch script.

pub const WINDOWS_WRAPPER: &str = r#"
@echo off
set JUSTFILE=%~1
shift
just --justfile "%JUSTFILE%" %*
echo 1337;LazyjustDone=%ERRORLEVEL%
%ComSpec%
"#;

pub fn build_unix_command(
    _justfile: &std::path::Path,
    _recipe: &str,
    _args: &[String],
) -> (Vec<String>, Vec<(String, String)>) {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
    (vec![shell, "-i".to_string()], Vec::new())
}
```

- [ ] **Step 4: Update the call site in `src/session/manager.rs`**

In `src/session/manager.rs`, locate the line that reads:

```rust
        let line = super::wrapper::prime_line(justfile, recipe, args);
```

Replace with:

```rust
        let line = super::shell::prime_line(justfile, recipe, args);
```

No other edits in this file.

- [ ] **Step 5: Update the import in `tests/session_integration.rs`**

Near the top of `tests/session_integration.rs`, locate:

```rust
use lazyjust::session::wrapper::{build_unix_command, prime_line};
```

Replace with two lines:

```rust
use lazyjust::session::shell::prime_line;
use lazyjust::session::wrapper::build_unix_command;
```

No other edits in this file in this task.

- [ ] **Step 6: Verify the crate compiles**

Run: `cargo build`
Expected: `Finished` with no errors.

- [ ] **Step 7: Verify every unit test still passes**

Run: `cargo test -p lazyjust --lib session::shell::tests`
Expected: `test result: ok. 9 passed`.

Run: `cargo test -p lazyjust --lib session::wrapper::tests`
Expected: `test result: ok. 0 passed` (the module no longer has a `tests` submodule — this command is expected to report "no tests found" or `0 passed`; that's the point of the move, and is NOT a regression).

Run: `cargo test`
Expected: every test target `test result: ok`. No failures.

- [ ] **Step 8: Verify lint and format are clean**

Run: `cargo clippy --all-targets -- -D warnings`
Expected: `Finished` with zero warnings.

Run: `cargo fmt --check`
Expected: no output, exit 0.

- [ ] **Step 9: Commit**

```bash
git add src/session/shell.rs src/session/wrapper.rs src/session/mod.rs src/session/manager.rs tests/session_integration.rs
git commit -m "refactor(session): split wrapper.rs; shell-string helpers move to shell.rs"
```

---

## Task 2: Rustdoc on trust-boundary helpers

**Files:**
- Modify: `src/session/shell.rs`
- Modify: `src/session/wrapper.rs`

- [ ] **Step 1: Add rustdoc on `shell::shell_quote`**

In `src/session/shell.rs`, insert the following doc comment immediately above `pub fn shell_quote(s: &str) -> String {`:

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
```

- [ ] **Step 2: Add rustdoc on `shell::prime_line`**

In `src/session/shell.rs`, insert the following doc comment immediately above `pub fn prime_line(justfile: &std::path::Path, recipe: &str, args: &[String]) -> String {`:

```rust
/// Builds the line fed into an interactive shell's stdin to run a recipe.
/// Every user-controlled value (justfile path, recipe name, each arg) is
/// `shell_quote`'d. The trailing `printf` emits the `LazyjustDone=%d` OSC
/// marker parsed by `session::osc::scan_done_marker`.
```

- [ ] **Step 3: Add rustdoc on `wrapper::build_unix_command`**

In `src/session/wrapper.rs`, insert the following doc comment immediately above `pub fn build_unix_command(`:

```rust
/// Returns the argv for the PTY spawn on Unix: `[$SHELL, "-i"]` (fallback
/// `/bin/sh`). The recipe itself is not in argv — it is delivered via
/// `crate::session::shell::prime_line` written to the shell's stdin after
/// rc-file init. Parameters are kept for signature stability with the
/// (future) Windows builder.
```

- [ ] **Step 4: Verify build**

Run: `cargo build`
Expected: `Finished` with no errors.

- [ ] **Step 5: Verify tests**

Run: `cargo test`
Expected: every test target `test result: ok`.

- [ ] **Step 6: Verify lint and format are clean**

Run: `cargo clippy --all-targets -- -D warnings`
Expected: `Finished` with zero warnings.

Run: `cargo fmt --check`
Expected: no output, exit 0.

- [ ] **Step 7: Commit**

```bash
git add src/session/shell.rs src/session/wrapper.rs
git commit -m "docs(session): rustdoc shell_quote, prime_line, build_unix_command"
```

---

## Task 3: `SessionMeta.command_line` clarification

**Files:**
- Modify: `src/app/types.rs`
- Modify: `src/session/manager.rs`

- [ ] **Step 1: Add a doc comment to `SessionMeta::command_line`**

In `src/app/types.rs`, locate the field declaration:

```rust
    pub command_line: String,
```

inside `pub struct SessionMeta { ... }`. Replace the single line with the doc-commented form:

```rust
    /// Human-readable recipe invocation (`just --justfile <p> <recipe> <args>`).
    /// Informational only — the actual PTY argv is `$SHELL -i` and the recipe
    /// is delivered via `session::shell::prime_line` on stdin.
    pub command_line: String,
```

- [ ] **Step 2: Add a leading comment at the `format!` call site**

In `src/session/manager.rs`, locate:

```rust
        let command_line = format!(
            "just --justfile {} {} {}",
            justfile.display(),
            recipe,
            args.join(" ")
        );
```

Replace with:

```rust
        // Informational string only; see SessionMeta::command_line doc.
        let command_line = format!(
            "just --justfile {} {} {}",
            justfile.display(),
            recipe,
            args.join(" ")
        );
```

- [ ] **Step 3: Verify build**

Run: `cargo build`
Expected: `Finished` with no errors.

- [ ] **Step 4: Verify tests**

Run: `cargo test`
Expected: every test target `test result: ok`.

- [ ] **Step 5: Verify lint and format are clean**

Run: `cargo clippy --all-targets -- -D warnings`
Expected: `Finished` with zero warnings.

Run: `cargo fmt --check`
Expected: no output, exit 0.

- [ ] **Step 6: Commit**

```bash
git add src/app/types.rs src/session/manager.rs
git commit -m "docs(session): clarify SessionMeta.command_line is informational"
```

---

## Task 4: Test `SHELL` setup via `std::sync::Once`

**Files:**
- Modify: `tests/session_integration.rs`

- [ ] **Step 1: Add the `Once`-backed helper**

In `tests/session_integration.rs`, immediately below the existing `use` statements at the top of the file, add:

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

- [ ] **Step 2: Replace the first `set_var` call**

In `tests/session_integration.rs`, locate the body of `fn spawn_echo_recipe_and_capture_done_marker()`. Its first statement is:

```rust
    // Force a minimal shell so rc files cannot eat stdin or reorder output.
    std::env::set_var("SHELL", "/bin/sh");
```

Replace both lines with the single call:

```rust
    force_posix_shell();
```

- [ ] **Step 3: Replace the second `set_var` call**

In `tests/session_integration.rs`, locate the body of `async fn session_manager_spawn_recipe_primes_shell_and_emits_done()`. It contains:

```rust
    std::env::set_var("SHELL", "/bin/sh");
```

Replace with:

```rust
    force_posix_shell();
```

- [ ] **Step 4: Verify the integration tests still pass**

Run: `cargo test --test session_integration`
Expected: `test result: ok. 2 passed`.

- [ ] **Step 5: Verify the full suite**

Run: `cargo test`
Expected: every test target `test result: ok`.

- [ ] **Step 6: Verify lint and format are clean**

Run: `cargo clippy --all-targets -- -D warnings`
Expected: `Finished` with zero warnings.

Run: `cargo fmt --check`
Expected: no output, exit 0.

- [ ] **Step 7: Commit**

```bash
git add tests/session_integration.rs
git commit -m "test(session): serialize SHELL override via Once helper"
```

---

## Self-review notes

- **Spec coverage:** §1 → Task 1. §2 → Task 1 Step 3. §3 → Task 1 Step 2. §4 → Task 2 Step 1. §5 → Task 2 Step 2. §6 → Task 3. §7 → Task 4. §8 (caller updates) → Task 1 Steps 4-5. Every spec section has a task.
- **Placeholder scan:** none.
- **Type consistency:** `shell_quote(&str) -> String`, `prime_line(&Path, &str, &[String]) -> String`, `build_unix_command(&Path, &str, &[String]) -> (Vec<String>, Vec<(String, String)>)` — all match across the plan.
- **Name consistency:** `force_posix_shell()` is used consistently in Task 4 Steps 1-3.
