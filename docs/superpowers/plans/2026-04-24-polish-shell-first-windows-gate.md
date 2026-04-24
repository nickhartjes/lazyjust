# Shell-First Polish — Windows Gate Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Stop pretending lazyjust works on Windows: gate `SessionManager::spawn_recipe` with `#[cfg(unix)]`, return a clear runtime error on Windows, and delete the dead `WINDOWS_WRAPPER` constant.

**Architecture:** Two small, mechanical commits. Task 1 wraps the existing `spawn_recipe` body in `#[cfg(unix)]` and adds a `#[cfg(windows)]` early-return. Task 2 removes the dead `WINDOWS_WRAPPER` constant and rewrites the `wrapper.rs` module doc to match reality. Zero unix behavior change.

**Tech Stack:** Rust, existing `lazyjust` crate. No new dependencies.

**Spec:** `docs/superpowers/specs/2026-04-24-polish-shell-first-windows-gate-design.md`

---

## File Structure

| File | Action | Responsibility after this branch |
|---|---|---|
| `src/session/manager.rs` | Modify | `spawn_recipe` has a `#[cfg(windows)]` early-return with a descriptive error; the rest of the body runs under `#[cfg(unix)]`. |
| `src/session/wrapper.rs` | Modify | Holds only `build_unix_command`. `WINDOWS_WRAPPER` deleted. Module `//!` doc rewritten to drop the stale "Windows still uses a wrapper batch script" sentence. |

---

## Task 1: Gate `spawn_recipe` with `#[cfg(unix)]` / `#[cfg(windows)]`

**Files:**
- Modify: `src/session/manager.rs`

- [ ] **Step 1: Read the current `spawn_recipe` body**

Run: `sed -n '31,125p' src/session/manager.rs`
Expected: see the existing `impl SessionManager { #[allow(clippy::too_many_arguments)] pub fn spawn_recipe(...) -> Result<SessionMeta> { ... } }` block.

Note: the existing body starts at the line that begins with `let (argv, _) = build_unix_command(...)` and ends with the `Ok(SessionMeta { ... })` return. That entire block moves inside a `#[cfg(unix)] { ... }` wrapper without any other edits. The `#[cfg(windows)]` block is new and sits before the unix block.

- [ ] **Step 2: Replace the `spawn_recipe` definition**

In `src/session/manager.rs`, locate the function header:

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
```

Immediately after the opening `{` of `spawn_recipe`, insert the `#[cfg(windows)]` early-return block:

```rust
        #[cfg(windows)]
        {
            let _ = (id, justfile, recipe, args, cwd, rows, cols, log_path, tx, log_cap);
            return Err(crate::error::Error::PtySpawn(
                "lazyjust: Windows support not yet implemented (tracked as a separate sub-project)".into(),
            ));
        }
```

Then wrap the entire existing body (from `let (argv, _) = build_unix_command(...)` through the final `Ok(SessionMeta { ... })`) in a `#[cfg(unix)] { ... }` block. The final function should look like:

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
            // ---- existing body moves here unchanged ----
            let (argv, _) = build_unix_command(justfile, recipe, args);

            let SpawnedPty {
                master,
                child,
                writer,
                reader,
            } = spawn(&argv, cwd, rows, cols)?;

            // Informational string only; see SessionMeta::command_line doc.
            let command_line = format!(
                "just --justfile {} {} {}",
                justfile.display(),
                recipe,
                args.join(" ")
            );

            let last_output: super::reader::LastOutput = Arc::new(Mutex::new(None));
            spawn_reader(reader, id, tx, Arc::clone(&last_output));

            let writer: SharedWriter = Arc::new(Mutex::new(writer));
            let prime_writer = Arc::clone(&writer);
            let line = super::shell::prime_line(justfile, recipe, args);
            std::thread::spawn(move || {
                // Wait for the shell's rc files to finish and the line editor to
                // enter raw mode. Heuristic: poll the reader's last-output timestamp
                // until the shell has been quiet for `idle_ms` after producing at
                // least one chunk. Fall back to a hard cap so we prime even on a
                // perfectly silent shell.
                let idle = std::time::Duration::from_millis(400);
                let cap = std::time::Duration::from_millis(5000);
                let start = std::time::Instant::now();
                loop {
                    std::thread::sleep(std::time::Duration::from_millis(50));
                    let last = last_output.lock().ok().and_then(|g| *g);
                    if start.elapsed() >= cap {
                        break;
                    }
                    if let Some(t) = last {
                        if t.elapsed() >= idle {
                            break;
                        }
                    }
                }
                if let Ok(mut w) = prime_writer.lock() {
                    let _ = w.write_all(line.as_bytes());
                    let _ = w.write_all(b"\r");
                    let _ = w.flush();
                }
            });

            let log_writer = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&log_path)
                .ok();

            self.handles.insert(
                id,
                SessionHandle {
                    master,
                    child,
                    writer,
                    log_writer,
                    log_written: 0,
                    log_cap,
                },
            );

            Ok(SessionMeta {
                id,
                recipe_name: recipe.to_string(),
                command_line,
                status: Status::Running,
                unread: true,
                started_at: Instant::now(),
                log_path,
            })
        }
    }
```

The exact existing body text may differ by whitespace; copy the current body as-is into the `#[cfg(unix)]` block rather than using the snippet above verbatim. The snippet is illustrative — it shows the wrapping shape, not the precise byte-for-byte content.

- [ ] **Step 3: Verify unix compile + tests + lint**

Run: `cargo build`
Expected: `Finished` with no errors.

Run: `cargo test`
Expected: every target `test result: ok`.

Run: `cargo clippy --all-targets -- -D warnings`
Expected: `Finished` with zero warnings.

Run: `cargo fmt --check`
Expected: no output, exit 0.

- [ ] **Step 4: Verify Windows target compiles**

Install the Windows target if missing:

```
rustup target add x86_64-pc-windows-gnu
```

Expected: `target x86_64-pc-windows-gnu is up to date` (or newly installed).

Run: `cargo check --target x86_64-pc-windows-gnu`

Expected: `Finished` with no errors. If the `portable_pty` crate (or a transitive dep) fails to build for `x86_64-pc-windows-gnu`, try `x86_64-pc-windows-msvc` (`rustup target add x86_64-pc-windows-msvc && cargo check --target x86_64-pc-windows-msvc`). If neither target compiles for reasons *outside this commit* (e.g. a known upstream dep issue), note that in the commit message and continue — the cfg gating is auditable from the unix build.

- [ ] **Step 5: Commit**

```bash
git add src/session/manager.rs
git commit -m "feat(session): gate spawn_recipe with cfg(unix); Windows returns explicit error"
```

---

## Task 2: Delete `WINDOWS_WRAPPER` and update `wrapper.rs` module doc

**Files:**
- Modify: `src/session/wrapper.rs`

- [ ] **Step 1: Overwrite `src/session/wrapper.rs`**

Replace the entire contents of `src/session/wrapper.rs` with:

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

The `WINDOWS_WRAPPER` constant is gone. The module `//!` doc no longer claims Windows uses a wrapper batch script.

- [ ] **Step 2: Dead-reference audit**

Run: `grep -rn 'WINDOWS_WRAPPER' src/ tests/`
Expected: zero hits.

Run: `grep -rn 'WINDOWS_WRAPPER' docs/`
Expected: matches only in the historical spec/plan markdown files under `docs/superpowers/specs/` and `docs/superpowers/plans/`. Those are frozen records, do NOT edit them.

- [ ] **Step 3: Verify unix compile + tests + lint**

Run: `cargo build`
Expected: `Finished` with no errors.

Run: `cargo test`
Expected: every target `test result: ok`.

Run: `cargo clippy --all-targets -- -D warnings`
Expected: `Finished` with zero warnings.

Run: `cargo fmt --check`
Expected: no output, exit 0.

- [ ] **Step 4: Verify Windows target still compiles**

Run: `cargo check --target x86_64-pc-windows-gnu`
Expected: `Finished` (or `x86_64-pc-windows-msvc` if gnu is unavailable — same fallback as Task 1).

- [ ] **Step 5: Commit**

```bash
git add src/session/wrapper.rs
git commit -m "refactor(session): delete dead WINDOWS_WRAPPER; drop stale module doc"
```

---

## Task 3: Manual smoke (no commit)

**Files:** none

- [ ] **Step 1: Launch on unix**

Run: `cargo run --release -- .`
Expected: UI opens, pressing `Enter` on a recipe runs it (shell-first priming works exactly as before).

- [ ] **Step 2: Quit**

Press `q` (confirm with `y` if sessions running).

- [ ] **Step 3: No commit**

Verification only. If anything regressed from pre-Task-1 behavior, bisect between Task 1 and Task 2 commits.

---

## Self-review notes

- **Spec coverage:** §Behavioral contract (Unix unchanged, Windows returns `PtySpawn` error) → Task 1. §Implementation 1 (gate `spawn_recipe`) → Task 1. §Implementation 2 (delete `WINDOWS_WRAPPER`, update doc) → Task 2. §Implementation 3 (dead-reference audit) → Task 2 Step 2. §Testing manual verification (`cargo check --target x86_64-pc-windows-*`) → Task 1 Step 4 + Task 2 Step 4.
- **Placeholder scan:** none.
- **Type consistency:** `spawn_recipe` signature and return type (`Result<SessionMeta>`) unchanged; `build_unix_command` signature unchanged; `Error::PtySpawn(String)` matches the existing variant used elsewhere in `session/manager.rs`.
