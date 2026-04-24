# Shell-First Recipe Spawn Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Reverse PTY spawn order so the user's interactive shell starts first and the recipe is delivered through its stdin — leaving recipe output at the bottom of the session pane instead of buried behind rc-file banners.

**Architecture:** Replace the `sh -c <wrapper>` indirection in `session/wrapper.rs` with a direct spawn of `$SHELL -i`. Add two pure helpers — `shell_quote` (POSIX single-quote escape) and `prime_line` (builds the `just … ; printf '\033]1337;LazyjustDone=%d\007' $?` string). `session/manager.rs::spawn_recipe` writes the primed line to the PTY writer immediately after `spawn()`, before starting the reader task. The `LazyjustDone` OSC marker stays unchanged, so `session/osc.rs` and downstream consumers need no edits.

**Tech Stack:** Rust 1.x, `portable-pty`, `tokio`, existing `lazyjust` crate layout.

**Spec:** `docs/superpowers/specs/2026-04-24-shell-first-recipe-spawn-design.md`

---

## File Structure

| File | Action | Responsibility |
|---|---|---|
| `src/session/wrapper.rs` | Modify | Drop `UNIX_WRAPPER`. Rewrite `build_unix_command` to return `[$SHELL, "-i"]`. Add pure helpers `shell_quote` and `prime_line`. Inline `#[cfg(test)] mod tests`. |
| `src/session/manager.rs` | Modify | After `spawn()`, write `prime_line(...)` + `\n` to `writer` and flush before `spawn_reader`. |
| `tests/session_integration.rs` | Modify | Update the existing low-level test for the new `build_unix_command`: prime the shell via `prime_line` before reading. Add a sibling end-to-end test that drives `SessionManager::spawn_recipe`. |

`WINDOWS_WRAPPER` and `build_windows_command` are untouched (Windows deferred per spec §4).

---

## Task 1: `shell_quote` helper

**Files:**
- Modify: `src/session/wrapper.rs` (add helper + `#[cfg(test)] mod tests`)

- [ ] **Step 1: Write the failing tests**

Append to `src/session/wrapper.rs`:

```rust
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
        // POSIX single-quote: literal newline is fine inside.
        assert_eq!(shell_quote("a\nb"), "'a\nb'");
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p lazyjust --lib session::wrapper::tests -- --nocapture`
Expected: FAIL — `cannot find function shell_quote in this scope`.

- [ ] **Step 3: Write minimal implementation**

Add above the `#[cfg(test)]` block in `src/session/wrapper.rs`:

```rust
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
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p lazyjust --lib session::wrapper::tests`
Expected: `test result: ok. 6 passed`.

- [ ] **Step 5: Commit**

```bash
git add src/session/wrapper.rs
git commit -m "feat(session): add shell_quote helper"
```

---

## Task 2: `prime_line` helper

**Files:**
- Modify: `src/session/wrapper.rs`

- [ ] **Step 1: Write the failing tests**

Add inside the existing `mod tests` block:

```rust
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
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p lazyjust --lib session::wrapper::tests::prime_line`
Expected: FAIL — `cannot find function prime_line in this scope`.

- [ ] **Step 3: Write minimal implementation**

Add to `src/session/wrapper.rs` (below `shell_quote`, above `#[cfg(test)]`):

```rust
pub fn prime_line(
    justfile: &std::path::Path,
    recipe: &str,
    args: &[String],
) -> String {
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
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p lazyjust --lib session::wrapper::tests`
Expected: `test result: ok. 9 passed`.

- [ ] **Step 5: Commit**

```bash
git add src/session/wrapper.rs
git commit -m "feat(session): add prime_line helper"
```

---

## Task 3: Replace `build_unix_command`; remove `UNIX_WRAPPER`

**Files:**
- Modify: `src/session/wrapper.rs`
- Modify: `tests/session_integration.rs`

The existing integration test `spawn_echo_recipe_and_capture_done_marker` in `tests/session_integration.rs` currently assumes `build_unix_command` embeds the recipe in argv. After this task it must prime the shell via the new helper.

- [ ] **Step 1: Write the failing (updated) integration test**

Replace the body of `tests/session_integration.rs` with:

```rust
#![cfg(not(windows))]

use lazyjust::session::osc::scan_done_marker;
use lazyjust::session::pty::spawn;
use lazyjust::session::wrapper::{build_unix_command, prime_line};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::time::{Duration, Instant};

fn make_justfile(tmp: &tempfile::TempDir) -> PathBuf {
    let path = tmp.path().join("justfile");
    std::fs::write(&path, "hi:\n\techo lazyjust-hello\n").unwrap();
    path
}

#[test]
fn spawn_echo_recipe_and_capture_done_marker() {
    // Force a minimal shell so rc files cannot eat stdin or reorder output.
    std::env::set_var("SHELL", "/bin/sh");

    let tmp = tempfile::tempdir().unwrap();
    let justfile = make_justfile(&tmp);

    let (argv, _) = build_unix_command(&justfile, "hi", &[]);
    let mut spawned = spawn(&argv, tmp.path(), 24, 80).unwrap();

    let line = prime_line(&justfile, "hi", &[]);
    spawned.writer.write_all(line.as_bytes()).unwrap();
    spawned.writer.write_all(b"\n").unwrap();
    spawned.writer.flush().unwrap();

    let mut buf = Vec::new();
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut chunk = [0u8; 4096];
    loop {
        if Instant::now() > deadline {
            panic!("timeout waiting for done marker");
        }
        match spawned.reader.read(&mut chunk) {
            Ok(0) => break,
            Ok(n) => {
                buf.extend_from_slice(&chunk[..n]);
                let (_, codes) = scan_done_marker(&buf);
                if !codes.is_empty() {
                    assert_eq!(codes[0], 0);
                    assert!(std::str::from_utf8(&buf)
                        .unwrap()
                        .contains("lazyjust-hello"));
                    let _ = spawned.child.kill();
                    return;
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) => panic!("read err: {e}"),
        }
    }
    panic!("EOF before done marker");
}
```

- [ ] **Step 2: Run test to verify it currently fails**

Run: `cargo test --test session_integration`
Expected: compile error or runtime panic — `build_unix_command` still returns the old wrapper-based argv, so the shell command embeds the recipe instead of reading it from stdin; the double-run + no-primed-stdin mismatch surfaces as a test failure or a compile error on the missing `prime_line` import path. This task proceeds to make it pass.

- [ ] **Step 3: Rewrite `build_unix_command` and delete `UNIX_WRAPPER`**

Edit `src/session/wrapper.rs`. Delete the `UNIX_WRAPPER` constant entirely. Replace `build_unix_command` with:

```rust
pub fn build_unix_command(
    _justfile: &std::path::Path,
    _recipe: &str,
    _args: &[String],
) -> (Vec<String>, Vec<(String, String)>) {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
    (vec![shell, "-i".to_string()], Vec::new())
}
```

Leave `WINDOWS_WRAPPER` and `build_windows_command` (if present) unchanged.

- [ ] **Step 4: Run integration test to verify it passes**

Run: `cargo test --test session_integration -- --nocapture`
Expected: `test result: ok. 1 passed`. Output contains `lazyjust-hello` and the `LazyjustDone=0` OSC byte sequence.

- [ ] **Step 5: Run full library unit tests to confirm wrapper tests still pass**

Run: `cargo test -p lazyjust --lib session::wrapper::tests`
Expected: `test result: ok. 9 passed`.

- [ ] **Step 6: Commit**

```bash
git add src/session/wrapper.rs tests/session_integration.rs
git commit -m "feat(session): spawn \$SHELL -i directly; drop recipe-embedded wrapper"
```

---

## Task 4: Wire `prime_line` into `SessionManager::spawn_recipe`

**Files:**
- Modify: `src/session/manager.rs`
- Modify: `tests/session_integration.rs` (add sibling end-to-end test)

- [ ] **Step 1: Write the failing end-to-end test**

Append to `tests/session_integration.rs`:

```rust
#[tokio::test]
async fn session_manager_spawn_recipe_primes_shell_and_emits_done() {
    use lazyjust::app::action::AppEvent;
    use lazyjust::session::manager::SessionManager;

    std::env::set_var("SHELL", "/bin/sh");

    let tmp = tempfile::tempdir().unwrap();
    let justfile = make_justfile(&tmp);
    let log_path = tmp.path().join("session.log");

    let (tx, mut rx) = tokio::sync::mpsc::channel::<AppEvent>(256);
    let mut mgr = SessionManager::default();

    let _meta = mgr
        .spawn_recipe(
            1,
            &justfile,
            "hi",
            &[],
            tmp.path(),
            24,
            80,
            log_path.clone(),
            tx,
            1024 * 1024,
        )
        .unwrap();

    let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
    let mut collected: Vec<u8> = Vec::new();
    loop {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            panic!("timeout waiting for done marker; got {:?}", String::from_utf8_lossy(&collected));
        }
        match tokio::time::timeout(remaining, rx.recv()).await {
            Ok(Some(AppEvent::SessionBytes { id, bytes })) => {
                assert_eq!(id, 1);
                collected.extend_from_slice(&bytes);
                let (_, codes) = scan_done_marker(&collected);
                if !codes.is_empty() {
                    assert_eq!(codes[0], 0);
                    assert!(std::str::from_utf8(&collected).unwrap().contains("lazyjust-hello"));
                    mgr.kill(1);
                    return;
                }
            }
            Ok(Some(_)) => continue,
            Ok(None) => panic!("channel closed before done marker"),
            Err(_) => panic!("timeout waiting for done marker"),
        }
    }
}
```

Note: this test requires the `tokio` test macro. `tokio` is already a workspace dependency; if `tokio::test` is not directly importable in this integration test file, add `tokio = { version = "<existing>", features = ["macros", "rt", "time"] }` to `[dev-dependencies]` in `Cargo.toml` mirroring the main `tokio` version. Run `cargo test --test session_integration -- --list` first to check if the macro is already available — it likely is, since `tests/retention_tests.rs` or similar may use it. If missing, update `Cargo.toml` dev-dependencies before running the test.

- [ ] **Step 2: Run test to verify it currently fails**

Run: `cargo test --test session_integration session_manager_spawn_recipe_primes_shell_and_emits_done -- --nocapture`
Expected: timeout panic — `spawn_recipe` spawns `$SHELL -i` but never primes the stdin, so `just` never runs and no `LazyjustDone` marker appears.

- [ ] **Step 3: Wire prime write into `spawn_recipe`**

Edit `src/session/manager.rs`. In `spawn_recipe`, after the `let SpawnedPty { master, child, writer, reader } = spawn(&argv, cwd, rows, cols)?;` line, change the binding to `mut writer` and insert the prime before `spawn_reader`:

```rust
let SpawnedPty {
    master,
    child,
    mut writer,
    reader,
} = spawn(&argv, cwd, rows, cols)?;

let line = super::wrapper::prime_line(justfile, recipe, args);
writer
    .write_all(line.as_bytes())
    .and_then(|_| writer.write_all(b"\n"))
    .and_then(|_| writer.flush())
    .map_err(|e| crate::error::Error::PtySpawn(format!("prime shell stdin: {e}")))?;

spawn_reader(reader, id, tx);
```

Keep the rest of the function untouched. `command_line` still uses its existing `format!("just --justfile {} {} {}", …)` form — it is informational only.

- [ ] **Step 4: Run the new test to verify it passes**

Run: `cargo test --test session_integration session_manager_spawn_recipe_primes_shell_and_emits_done -- --nocapture`
Expected: `test result: ok. 1 passed`. `collected` contains `lazyjust-hello` and a `LazyjustDone=0` OSC byte sequence.

- [ ] **Step 5: Run the full integration test file to confirm both tests pass**

Run: `cargo test --test session_integration`
Expected: `test result: ok. 2 passed`.

- [ ] **Step 6: Commit**

```bash
git add src/session/manager.rs tests/session_integration.rs
git commit -m "feat(session): prime shell stdin with recipe in spawn_recipe"
```

---

## Task 5: Final CI triad + manual QA

**Files:** none (verification only)

- [ ] **Step 1: Run `cargo fmt --check`**

Run: `cargo fmt --check`
Expected: no output, exit 0. If formatting drift, run `cargo fmt` and stage the fixup into the most recent task's commit via a new amendment (or a new commit — preferred, per repo convention).

- [ ] **Step 2: Run `cargo clippy --all-targets -- -D warnings`**

Run: `cargo clippy --all-targets -- -D warnings`
Expected: `Finished` with no warnings. Common issue: `_justfile`/`_recipe`/`_args` params in `build_unix_command` may trigger `clippy::needless_pass_by_value` — suppress with `#[allow(clippy::needless_pass_by_value)]` on the fn only if clippy actually fires; otherwise leave untouched.

- [ ] **Step 3: Run the full test suite**

Run: `cargo test`
Expected: every test target reports `test result: ok`. New totals: wrapper unit tests +9; integration tests +1 (=2).

- [ ] **Step 4: Manual QA against live binary**

Run: `cargo run --release -- .`

Validate against spec §Testing → Manual QA:

| Step | Expected |
|---|---|
| `Enter` on `fmt` | Pane shows zsh banner (fastfetch/starship), then `just --justfile '…' 'fmt'` echo, then `cargo fmt` output, then prompt. |
| Recipe completes | Bottom of pane = recipe output + prompt; no banner overlay. |
| `PgUp` | Scrollback reveals banner above recipe. |
| Type `ls` in the pane | Shell is live; command executes. |
| `x` | Pane closes; log file at `~/.local/share/lazyjust/…` or configured path contains the full stream. |

If any row fails, file a follow-up and either revert the feature commits or adjust the spec.

- [ ] **Step 5: No commit**

Nothing to commit — this task is verification only. The branch state after Task 4 is the final deliverable.

---

## Self-review notes

- **Spec coverage:** §Behavioral contract → Tasks 3/4. §Implementation 1 (`wrapper.rs`) → Tasks 1/2/3. §Implementation 2 (`manager.rs`) → Task 4. §Implementation 3 (reader/OSC no-change) → verified in Task 4 test. §Implementation 4 (Windows deferred) → explicit no-op. §Testing (unit) → Tasks 1/2. §Testing (integration) → Tasks 3/4. §Testing (manual QA) → Task 5.
- **Placeholder scan:** none.
- **Type consistency:** `shell_quote(&str) -> String`, `prime_line(&Path, &str, &[String]) -> String`, `build_unix_command(&Path, &str, &[String]) -> (Vec<String>, Vec<(String,String)>)` consistent across tasks.
