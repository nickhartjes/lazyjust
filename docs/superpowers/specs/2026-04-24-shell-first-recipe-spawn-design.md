# Shell-First Recipe Spawn — Design Specification

**Date:** 2026-04-24
**Status:** Draft, pending implementation plan
**Owner:** Nick Hartjes
**Depends on:** `feat(ui): visible focus indicator + non-stealing spawn + Ctrl+g exit` (commit `3eba98e`)

## Overview

Today the PTY wrapper runs `just` first and then `exec`s an interactive shell. When the user's shell init prints a banner (fastfetch, starship, etc.) the banner repaints the vt100 grid after the recipe has finished, so recipe output is buried below a full screen of shell-startup noise. Scrollback still contains the bytes, but the "session pane" appears to show only the post-exit shell.

Reverse the order: spawn the interactive shell first, then feed the recipe command into its stdin. Banner prints, recipe runs, prompt returns — the pane reads top-to-bottom like a real terminal session, with recipe output at the bottom rather than above several screens of rc-file output.

## Goals

1. After a recipe exits, the recipe's output is the most-recently-rendered content in the session pane (no shell banner overwriting it).
2. Post-exit interactive behavior (user keeps typing in the same pane) is preserved — the shell stays alive.
3. Existing `LazyjustDone=<code>` OSC-based exit capture continues to work.

## Non-goals

- Windows parity (deferred — see section 4).
- Suppressing or customizing the user's shell rc files.
- Prompt detection / OSC-133 integration.
- Handling shells whose rc files actively consume stdin during init (e.g. exotic plugins that `read` at load time).

---

## Behavioral contract

- Spawn command: user's `$SHELL -i` (fallback `/bin/sh`), with the recipe invocation delivered via the PTY writer immediately after `spawn()`.
- Shell runs its rc files normally. Any banner or prompt drawing prints first.
- Then the shell reads its stdin, sees the pre-typed `just …` line, and executes it as if the user had typed it.
- When `just` exits, the trailing `printf` emits the `LazyjustDone=<code>` OSC sequence. `session/osc.rs` parses it unchanged.
- The shell stays running; `Status::ShellAfterExit { code }` semantics are unchanged.
- All justfile path / recipe name / argument values are POSIX single-quoted before being written to the shell so whitespace, `$`, `;`, `'`, etc. cannot cause command injection from untrusted arg values.

---

## Implementation

### 1. `src/session/wrapper.rs`

Replace the unix wrapper. The spawned argv becomes the shell itself:

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

Signature kept for API compatibility; parameters are ignored. Callers pass them to `prime_line` instead.

Add two helpers:

```rust
pub fn shell_quote(s: &str) -> String {
    // POSIX single-quote escape: 'foo' → 'foo', it's → 'it'\''s', empty → ''
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
```

`UNIX_WRAPPER` constant is removed. `WINDOWS_WRAPPER` and `build_windows_command` are untouched.

### 2. `src/session/manager.rs::spawn_recipe`

Between the `spawn()` call and `spawn_reader`, write the primed line to `writer` with a trailing `\n` and flush. If the write fails, propagate as `Error::PtySpawn` (same error variant used by `spawn`):

```rust
let SpawnedPty { master, child, mut writer, reader } = spawn(&argv, cwd, rows, cols)?;

let line = crate::session::wrapper::prime_line(justfile, recipe, args);
writer
    .write_all(line.as_bytes())
    .and_then(|_| writer.write_all(b"\n"))
    .and_then(|_| writer.flush())
    .map_err(|e| crate::error::Error::PtySpawn(format!("prime shell stdin: {e}")))?;

spawn_reader(reader, id, tx);
```

`command_line` (used for log headers / debug) keeps its current `format!("just --justfile {} {} {}", …)` rendering — it's informational, not executed.

### 3. Reader / OSC path — no change

`spawn_reader`, `osc.rs`, and the `LazyjustDone` handling are unchanged. They already treat arbitrary bytes from the PTY as a stream; banner output flows through them exactly the same as recipe output.

### 4. Windows — deferred

`build_windows_command` + `WINDOWS_WRAPPER` unchanged. Shell-first priming needs distinct argv + quoting rules (cmd/pwsh differ from POSIX). Tracked as a follow-up; not blocking for unix users.

### 5. Surface area check

No changes required in `event_loop.rs`, `app/reducer.rs`, `ui/session_pane.rs`, or any key-handling code — they all operate on the PTY byte stream + `SessionMeta`, both unchanged.

---

## Testing

### Unit tests — `src/session/wrapper.rs`

Add `#[cfg(test)] mod tests` with:

- `shell_quote` cases: `foo` → `'foo'`; `foo bar` → `'foo bar'`; `it's` → `'it'\''s'`; `$(x)` → `'$(x)'`; empty → `''`; value containing newline → `'a\nb'` (literal newline inside single-quotes is fine for POSIX).
- `prime_line` case: `(/p/Justfile, "build", ["a b", "x"])` → `just --justfile '/p/Justfile' 'build' 'a b' 'x' ; printf '\033]1337;LazyjustDone=%d\007' $?`.

### Integration — `tests/session_integration.rs`

Extend the existing `spawn_echo_recipe_and_capture_done_marker` test (or add a sibling) to:

1. Before invoking `spawn_recipe`, set `std::env::set_var("SHELL", "/bin/sh")` for the test process so the spawned child inherits a known minimal shell with no rc files. Restore afterwards if other tests depend on the value.
2. Spawn a recipe that prints a known string.
3. Assert the captured byte stream contains both the printed string and the `LazyjustDone=0` marker.
4. Assert the session is still `Running` (from `try_wait` perspective) shortly after the marker — shell alive.

### Manual QA (local, 2026-04-24)

Run `lazyjust` in this repo:

| Step | Expected |
|---|---|
| `Enter` on `fmt` | Pane shows zsh banner (fastfetch/starship), then `just --justfile '…' 'fmt'` echo, then `cargo fmt` output, then prompt. |
| Recipe completes | Bottom of pane = recipe output + prompt; no banner after recipe. |
| `PgUp` | Scrollback reveals banner above recipe. |
| Type `ls` in the pane | Works. Shell is live. |
| `x` | Pane closes; log file on disk contains full stream. |

---

## Risks / known limitations

1. Shells whose rc files `read` from stdin during init could swallow the primed line. Not observed with stock zsh / bash / fish. Documented as a known limitation; workaround is to export `SHELL=/bin/sh` before launching `lazyjust`.
2. Timing race: if the PTY's stdin buffer overflows before the shell drains it, writes could block. Prime line is a single line (~200 bytes worst case); well under typical PTY buffer (4 KB+).
3. Interactive shell banner may include cursor-movement / alt-screen sequences. vt100 parser handles these already for recipe output; no new code path.

## Rollback

Single-commit revert restores `UNIX_WRAPPER` and removes `prime_line` + the stdin-write block in `spawn_recipe`.
