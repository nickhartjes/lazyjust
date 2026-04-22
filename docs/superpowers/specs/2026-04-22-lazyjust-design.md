# lazyjust — Design Specification

**Date:** 2026-04-22
**Status:** Draft, pending implementation plan
**Owner:** Nick Hartjes

## Overview

`lazyjust` is a terminal UI front-end for [`just`](https://github.com/casey/just) inspired by `lazygit`. It discovers every justfile in a project, lets the user browse and filter recipes with fuzzy search, and runs them inside embedded pseudo-terminals so interactive prompts (passwords, nested editors, colored output) continue to work. Multiple recipes can run concurrently; completed sessions drop into an interactive shell in the same PTY so the user can keep investigating.

## Goals

1. Discover all justfiles in a project (root + recursive scan, respecting `.gitignore`).
2. Browse and fuzzy-filter recipes, grouped by `[group('...')]` attributes and by `mod` modules.
3. Run recipes in embedded PTYs with full interactivity (passwords, colors, TUI subprograms).
4. Allow concurrent sessions — many recipes running at once, switchable.
5. Shell-after-exit: once a recipe finishes, its PTY becomes an interactive shell in the same working directory so the user can continue typing commands.
6. Per-recipe session history inline on the recipe list, with unread tracking.

## Non-goals (v1)

- Editing justfiles.
- Remote execution / SSH.
- Recipe dependency graph visualisation.
- Session persistence across restarts (re-attaching to running PTYs after quit).
- Custom themes or a user config file.
- `.env` / dotenv management UI.
- Recipe favorites / custom ordering.
- Mouse interaction beyond terminal resize detection.

## Success criteria

- `lazyjust` invoked in a project with 50+ recipes renders the list in under 200 ms.
- The user can start `just deploy`, be prompted for a sudo password, enter it, then navigate away and run additional recipes while `deploy` continues in the background.
- Works on macOS, Linux, and Windows (ConPTY) with no extra setup beyond `just` being on PATH.
- No in-process justfile parsing — `just`'s own parser remains the source of truth.

## Target users

Developers already using `just` who run many recipes in monorepo-style projects and want fast navigation without remembering recipe names.

---

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                  lazyjust binary                        │
│                                                         │
│   ┌───────────────┐    ┌──────────────────────────┐     │
│   │ App (state)   │◀───│ Event loop (tokio)       │     │
│   │               │    │  - crossterm events      │     │
│   │ - justfiles[] │    │  - session output msgs   │     │
│   │ - recipes[]   │    │  - process exit signals  │     │
│   │ - sessions{}  │    └──────────────────────────┘     │
│   │ - focus       │                 ▲                   │
│   └───────┬───────┘                 │                   │
│           │                         │                   │
│           ▼                         │                   │
│   ┌───────────────┐    ┌───────────────────────────┐    │
│   │ Renderer      │    │ Session manager           │    │
│   │ (ratatui)     │    │  - spawn/kill PTYs        │    │
│   │               │    │  - per-session tokio task │    │
│   │ - list view   │    │  - vt100 screen per sess  │    │
│   │ - pty view    │    └──────┬────────────────────┘    │
│   │ - modals      │           │                         │
│   └───────────────┘           ▼                         │
│                        ┌───────────────────────────┐    │
│                        │ portable-pty (PTY pair)   │    │
│                        └──────┬────────────────────┘    │
│                               ▼                         │
│                  [child process: `just recipe` then sh] │
└─────────────────────────────────────────────────────────┘
           │
           ▼
   ┌─────────────────────┐
   │ Discovery module    │ (startup + on demand)
   │  - walk project fs  │
   │  - respect gitignore│
   │  - invoke `just --dump` per file
   └─────────────────────┘
```

Single binary, no IPC, no daemon. One process runs the TUI and owns all PTYs.

### Technology choices

| Concern | Choice | Rationale |
|---------|--------|-----------|
| TUI framework | `ratatui` + `crossterm` | De facto Rust TUI stack. Works on macOS/Linux/Windows. |
| PTY backend | `portable-pty` | Unix PTY + Windows ConPTY in one API. Battle-tested (wezterm). |
| VT / ANSI parsing | `vt100` | Produces a screen grid ratatui can render. Smaller surface than `alacritty_terminal`. |
| Async runtime | `tokio` | Compose crossterm events with N per-session reader channels via `select!`. |
| Justfile parsing | `just` binary via subprocess | `just --dump --dump-format=json` is authoritative; no parser drift. |
| FS discovery | `ignore` crate | `.gitignore`-aware walk used by `ripgrep`. |
| Fuzzy filter | `nucleo` (or `fuzzy-matcher` crate) | Fast fuzzy scoring, used by helix/zellij. Nucleo preferred. |
| Logging | `tracing` + `tracing-appender` | Structured logs, daily rotating file. |
| Snapshot tests | `insta` | Render output golden tests. |

### Modules

| Module | Responsibility |
|--------|----------------|
| `app` | Central state, event reducer, focus/mode logic. |
| `discovery` | FS walk, gitignore, invoke `just --dump`, build recipe tree. |
| `session` | PTY lifecycle, vt100 parser per session, bytes in/out. |
| `ui` | ratatui layout, panes, widgets, modal dialogs. |
| `input` | Keybindings map → app actions. |
| `config` | Hardcoded defaults (no config file v1). |

### State shape (sketch)

```rust
struct App {
    justfiles: Vec<Justfile>,
    active_justfile: usize,          // index into justfiles
    filter: String,                  // current fuzzy filter
    list_cursor: usize,              // highlighted recipe
    sessions: Vec<Session>,          // running + completed, indexed by SessionId
    active_session: Option<SessionId>, // focused session (None = show preview)
    focus: Focus,                    // List | Preview | Session | Modal(..)
    mode: Mode,                      // Normal | FilterInput | ParamInput | Dropdown | Help
    split_ratio: f32,                // left pane width fraction (default 0.30)
}

struct Justfile {
    path: PathBuf,
    recipes: Vec<Recipe>,
    groups: Vec<GroupName>,          // ordered as declared
}

struct Recipe {
    name: String,                    // e.g. "build" or "api::serve"
    module_path: Vec<String>,        // e.g. ["api"] for mods
    group: Option<String>,
    params: Vec<Param>,
    doc: Option<String>,
    command_preview: String,         // from `just --show`
    runs: Vec<SessionId>,            // session history for this recipe
}

struct Param {
    name: String,
    default: Option<String>,
    kind: ParamKind,                 // Positional | Variadic (`just` has no named params)
}

struct Session {
    id: SessionId,
    recipe_name: String,
    command_line: String,
    pty: Box<dyn MasterPty>,
    child: Box<dyn Child>,
    writer: Box<dyn Write + Send>,
    screen: vt100::Parser,
    status: Status,                  // Running | ShellAfterExit { code } | Exited { code }
    unread: bool,
    started_at: Instant,
    log_path: PathBuf,
}
```

---

## UI layout

```
┌── Top bar ────────────────────────────────────────────────┐
│  lazyjust   —   justfile: ./justfile ▾        ? help  q   │
├── Left pane (default 30%) ─┬── Right pane (default 70%) ──┤
│ GROUP: build               │ $ ./scripts/deploy.sh        │
│   build                    │ → Uploading artifacts...     │
│ > deploy        ●          │ Password for deploy@prod: █  │
│ GROUP: test                │                              │
│   test          ✓ 12s ago  │                              │
│ GROUP: api (mod)           │                              │
│   api::serve    ● ✓ ✗      │                              │
│                            │                              │
├────────────────────────────┴──────────────────────────────┤
│ Status: ↑↓ move  / filter  Enter run  ? help              │
└───────────────────────────────────────────────────────────┘
```

### Top bar

- Left: app name.
- Center: current justfile path with `▾` dropdown indicator. Press `d` to open dropdown.
- Right: `?` help, `q` quit hints.
- Session tabs are **not** in the top bar; session awareness lives inline on the recipe list.

### Left pane — recipe list

- Recipes grouped by `[group('name')]` attribute and by `mod` (imported modules render as their own groups with `mod::` names).
- Declaration order preserved within each group.
- Each row:
  ```
  > deploy          ●                    ./scripts/deploy.sh
    test            ✓ 12s ago            cargo test
    api::serve      ● ✓ ✗                (3 past runs)
  ```
- Inline **session indicators** show each recipe's session state:
  - `●` **blue** — currently running.
  - `✓` **green** — exited success, unread.
  - `✗` **red** — exited failure, unread.
  - `✓` / `✗` **gray** — viewed (user has opened that session's output since it exited).
  - Multiple runs → show up to three dots; if more, `✓ ×5` style summary.
- Groups collapsible (default expanded).

### Right pane

- If `active_session` is `None`: preview of highlighted recipe — command line, params, doc comment.
- If `active_session` is `Some(id)`: vt100-rendered screen of that session.

### Status bar

- Shows active keybindings relevant to the current mode.
- During filter input shows the filter buffer.
- During param input shows current param name and value buffer.

### Resizable panes

- Default split: 30% left / 70% right.
- `>` grow left pane by 5% (max 70%).
- `<` shrink left pane by 5% (min 15%).
- `=` reset to 30/70.
- Hard floors: left ≥ 20 cols, right ≥ 40 cols.
- Ratio stored in `App::split_ratio`, in-memory only (not persisted across restarts in v1).

### Navigation & input model

- **Normal mode** keybindings (lazygit-flavored, vim-style):
  - `j` / `k` — move cursor in list
  - `h` / `l` — cycle through sessions of highlighted recipe in chronological order (newest last); right pane updates
  - `Enter` — if recipe has running session, focus it; else run (prompts params if needed)
  - `Shift+Enter` or `r` — always spawn a new session (don't focus existing)
  - `K` — kill focused session (confirmation modal)
  - `x` — close focused session (confirmation if still running)
  - `/` — enter filter input mode
  - `d` — open justfile dropdown
  - `Tab` — cycle focus between list / right pane
  - `Ctrl+o` / `Ctrl+i` — jump to next/prev unread session
  - `L` — copy focused session's log path to clipboard + show in status bar
  - `>` / `<` / `=` — resize split
  - `?` — help modal
  - `q` — quit (confirmation if sessions running)
- **Session pane focus:** typed keys pass through to PTY writer. A reserved global escape (v1 default: `F12`) returns focus to the list without disturbing the running program. Exact key is an open question — must not collide with common nested programs (tmux `Ctrl+b`, vim `Esc`, shell job control `Ctrl+z`).
- **Filter input mode:** characters append to `app.filter`. `Esc` cancels. `Enter` returns to Normal with filter active.
- **Param input mode:** each param prompted with its default pre-filled. `Tab` / `Enter` advances. `Esc` cancels the run entirely.
- Both vim keys and arrow keys work; arrows were not listed in the answers but are the TUI norm and harmless.

---

## Data flow

### Startup

1. `lazyjust [path]` — path defaults to cwd.
2. `discovery::discover(path)`:
   1. Walk filesystem with the `ignore` crate, respecting `.gitignore` and hardcoded ignores (`.git/`, `node_modules/`, `target/`, `dist/`).
   2. Collect paths matching `justfile`, `Justfile`, `.justfile`, `*.just`.
   3. Root justfile = the one at depth 0 (or CLI `--justfile` override). If multiple at depth 0, use `just`'s native resolution order.
   4. For each file, spawn `just --justfile <p> --dump --dump-format=json`, parse into a `Justfile`.
3. Initialise `App` with discovered justfiles, root selected, list cursor at 0.
4. Enter ratatui alternate screen, raw mode, event loop.

### Select and run a recipe

1. User navigates list, presses `Enter`.
2. Input layer dispatches `Action::RunHighlighted`.
3. If the highlighted recipe has one or more running sessions and the user pressed plain `Enter` (not `Shift+Enter` / `r`) → focus the most recent running session and stop. No spawn, no param prompt.
4. Otherwise, if recipe has params → `Mode = ParamInput`, render modal, collect values. `Esc` aborts the whole run. `Enter` on last param → `Action::ConfirmParams(values)`.
5. `SessionManager::spawn`:
   1. Build argv: `["just", "--justfile", <path>, <recipe>, <param1>, <param2>, ...]`. Each element is a separate argv entry — no shell interpolation from lazyjust.
   2. Create PTY pair via `portable-pty` (initial size matches right pane; resized on layout changes).
   3. Actual child launched via a fixed wrapper script that takes argv via positional parameters so no quoting/interpolation is needed:
      ```sh
      #!/bin/sh
      justfile="$1"; shift
      just --justfile "$justfile" "$@"
      CODE=$?
      printf '\033]1337;LazyjustDone=%d\007' "$CODE"
      exec "${SHELL:-/bin/sh}" -i
      ```
      Spawned as: `sh -c "$WRAPPER" sh <justfile> <recipe> <param1> <param2> ...`. The embedded `\033` and `\007` are interpreted by `printf`, producing a real ESC-terminated OSC sequence.
      Windows: equivalent via `cmd.exe /c` or `pwsh` wrapper that emits the same OSC sequence before `exec`-ing the user shell.
   4. Spawn a blocking tokio task `reader(master_reader, id, tx)` that forwards bytes to the main loop via `mpsc::Sender<AppEvent>`.
   5. Insert `Session` into `app.sessions`; set `active_session`; append its `SessionId` to the recipe's `runs` list.

### Session output → render

```
reader task:
  loop {
    n = master_reader.read(&mut buf)?;
    if n == 0 { break; }
    tx.send(AppEvent::SessionBytes { id, bytes: buf[..n].to_vec() }).await;
  }

main loop (tokio::select!):
  event = select! {
    ct  = crossterm_events.next() => Crossterm(ct),
    msg = session_rx.recv()       => Session(msg),
    _   = tick_interval.tick()    => Tick,
  };
  match event {
    Session(SessionBytes { id, bytes }) => {
      sess = app.sessions.get_mut(id);
      sess.screen.process(&bytes);      // vt100 updates grid
      sess.log_writer.write_all(&bytes)?;
      if id != app.active_session { sess.unread = true; }
      mark_dirty();
    }
    Tick => {
      for sess in running { if let Some(code) = sess.child.try_wait()? { sess.status = Exited(code); } }
    }
    Crossterm(evt) => input::handle(evt) → action → app::reduce(action),
  }
  if dirty { render(); }
```

- Render throttled: batched at most every ~16 ms (≈60 fps), never on every byte.
- `vt100::Parser` is configured with a custom OSC callback that detects `OSC 1337 LazyjustDone=<code>` and emits `AppEvent::RecipeExited { id, code }` while stripping the sequence from the visible grid.

### Shell-after-exit

- The wrapper `sh -c "just X; <OSC marker>; exec $SHELL -i"` means once the recipe exits, the shell takes over the same PTY in the same cwd.
- `AppEvent::RecipeExited` flips `session.status` from `Running` → `ShellAfterExit { code }`. Inline indicator on list turns to `✓`/`✗` (unread).
- The PTY stays open; the user can type commands into it as in any shell.
- When the wrapper shell itself exits, `child.try_wait()` returns the shell's exit code; `status` transitions to `Exited { code }`. The session's output is preserved, scrollable, but input is disabled.

### Switch justfile

1. `d` opens dropdown. Dropdown gets its own fuzzy filter if there are more than ~10 justfiles.
2. `Enter` on a justfile → `Action::SelectJustfile(idx)`.
3. `app.active_justfile` updated, cursor reset, filter cleared.
4. Sessions are **not** closed; they remain in state and still show as indicators on the recipes that spawned them when those justfiles are re-selected.

### Filter recipes

1. `/` enters `FilterInput` mode.
2. Each typed char updates `app.filter`.
3. Renderer filters recipes with `nucleo` scoring; groups with zero matches hide; ordering: score desc, tie-break by declaration order.
4. `Esc` clears and exits. `Enter` keeps filter and returns to Normal.

### Resize

- `crossterm::Event::Resize(w, h)` recomputes layout.
- For each session, call `session.pty.resize(rows, cols)` — portable-pty sends SIGWINCH (or ConPTY equivalent).
- `vt100::Parser::set_size(rows, cols)` on each session.

---

## Error handling

### Startup failures

| Failure | Handling |
|---------|----------|
| `just` binary not on PATH | Print to stderr, exit code 2: `lazyjust requires 'just' in PATH. Install: https://github.com/casey/just`. No TUI started. |
| No justfiles found under scan root | Enter TUI with empty-state screen: `No justfiles found under <path>. Press q to quit.` |
| `just --dump` fails for a justfile (syntax, missing import) | Skip that justfile; record error. Banner: `N justfiles failed to load. Press e to view.` → modal with per-file errors. Rest of app functional. |
| `ignore` walk errors (permission denied) | Silently skip, log at debug. |
| Terminal below hard floor (40×10) | Full-screen `Terminal too small` message until resized. |

### Runtime failures

| Failure | Handling |
|---------|----------|
| PTY spawn fails | Modal: `Failed to spawn session: <err>`. App continues. |
| Child process crashes or signalled | Session `status = Exited(signal_code)`, indicator red, output retained. |
| `just --show` fails for preview | Preview pane shows the error text. Recipe can still be attempted. |
| PTY reader read error | `status = Exited(-1)`, log reason. No panic. |
| `vt100` parser panic (unexpected) | Guarded by `catch_unwind`; session marked `BrokenOutput`, error line shown in pane. |
| Terminal write error | Restore terminal state, flush to stderr, graceful exit. |

### Cleanup and shutdown

- `q` with running sessions → modal `N sessions running. [q] quit & kill all, [c] cancel`.
- On exit (graceful or panic): terminal `AlternateScreen` torn down via `Drop` guard; raw mode disabled; cursor restored; PTYs killed (SIGHUP to children on Unix, `Child::kill` on Windows).
- Panic hook installed at startup. On panic, first restore terminal, then print panic to stderr.

### Logging

Two distinct log streams:

- **Application log** — `tracing` + `tracing-appender` daily rotation.
  - Location: `$XDG_STATE_HOME/lazyjust/lazyjust.log` (Unix); `%LOCALAPPDATA%\lazyjust\lazyjust.log` (Windows).
  - Default level `warn`; `--log-level=debug|info|warn|error` CLI flag.
  - Contents: startup, discovery errors, spawn failures, panic traces, lifecycle markers.
- **Per-session logs** — raw PTY byte stream captured as each session runs.
  - Location: `$XDG_STATE_HOME/lazyjust/sessions/YYYY-MM-DD/<recipe>_<hhmmss>_<pid>.log`.
  - Format: raw bytes including ANSI sequences.
  - Size cap: 10 MB per session, rotating `.log.1` on overflow.
  - Retention: older session log directories pruned on startup (default 7 days).
  - UI: `L` on focused session copies log path and echoes it in the status bar.

### Security considerations

- Recipes execute user-authored shell commands by design; no sandboxing. Trust model: the user is running their own justfile.
- Param inputs are passed as separate `argv` entries to the `just` subprocess. lazyjust does not shell-interpolate them; `just` handles its own quoting.
- No network, auto-update, or telemetry.

---

## Testing

### Unit tests

| Target | Coverage |
|--------|----------|
| `discovery` | Fixture trees: single justfile, nested justfiles, gitignored justfile, broken justfile, custom names (`*.just`). |
| `discovery::parse_just_dump` | Known `just --dump` JSON fixtures → expected `Recipe` tree. Plain, params+defaults, variadic, grouped, imported, modded. |
| `app` reducer | Pure `reduce(state, action) → state` tests. Filter, cursor, run, param flow, switch justfile, close session. |
| `input::keymap` | Table-driven: `(event, mode) → Action`. |
| `session::ansi_marker` | Parse `OSC 1337 LazyjustDone=<code>` correctly and strip from visible output. |

### Integration tests

- `session::spawn` against real `sh` (Unix CI):
  - `echo hi` → exit 0, expected output.
  - `read pw; echo got $pw` with `hello\n` written → `got hello`.
  - Long output (10k lines) → no backpressure deadlock.
  - Kill session → child reaped, no zombies.
- End-to-end TUI snapshots via `insta`:
  - Render initial screen against a fixture justfile.
  - Run a fake recipe with `echo done`, wait for exit, snapshot.
  - Filter / param modal snapshots.
- Gated for Unix (`cfg(not(windows))`) where PTY behaviour diverges; Windows runs unit tests initially.

### Manual test matrix (pre-release)

| Scenario | Expected |
|----------|----------|
| `sudo echo hi` in a recipe | Password prompt visible, input masked, command succeeds. |
| `vim` opened inside a recipe | Vim renders fully; `:q` returns to wrapper shell. |
| Long-running `watch -n1 date` | Indicator stays blue; can switch away and back. |
| Quit lazyjust mid-run | Children cleaned up (`pgrep just` returns empty). |
| Resize terminal window | Session PTY reflects new size (`tput lines`). |
| Recipe with non-zero exit | Red `✗` indicator, shell-after-exit still reachable. |

### Testing crates

- `insta`, `pretty_assertions`, `rstest`, `tempfile`.

---

## Milestones (suggested order)

| # | Milestone | Validates |
|---|-----------|-----------|
| M1 | Crate skeleton + `just --dump` parser + `Recipe` model. | Can read a justfile and print recipes. |
| M2 | Static ratatui layout: top bar, left list, right preview pane, resizable panes. Hardcoded recipe list. | Visual shell; navigation works. |
| M3 | `discovery` module + justfile dropdown. Filter (`/`). Groups rendered. | Full read-only browsing of any project. |
| M4 | `session::spawn` with `portable-pty` + `vt100` render in right pane. Run single no-arg recipe. Inline session indicators. | Core interactive run works. |
| M5 | Param modal. Shell-after-exit via OSC marker. Multiple concurrent sessions. History per recipe (`h`/`l`). Unread tracking. | Feature-complete against this spec. |
| M6 | Logging (app + per-session), error modals, signal handling, panic hook. Cross-platform validation (macOS/Linux/Windows). | Production ready. |
| M7 | Snapshot tests, PTY integration tests, CI matrix, docs (README, asciinema demo), release workflow. | Shippable. |

---

## Open questions (decide during implementation)

1. Kill running session — single key, or confirmation modal? Leaning `K` with modal default-yes.
2. Recipe ordering — alphabetical within group, or preserve declaration order? Leaning declaration order (matches `just --list`).
3. Root justfile ambiguity — if multiple justfiles at project root depth, which wins? Leaning `just`'s own resolution order.
4. Max concurrent sessions cap — hard limit? Leaning none; log warning above 20.
5. Session log retention — 7 days default; confirm during implementation.
6. Dropdown fuzzy filter threshold — enable when justfile count > 10; confirm during build.
7. Mouse drag on splitter bar — v1 keyboard only; v2 candidate.
8. Session-to-list escape key — default `F12`; confirm a key that doesn't collide with tmux / vim / job control.

## Out of scope (v1)

- Editing justfiles.
- Recipe dependency graph viewer.
- Remote execution / SSH.
- Session persistence across restarts (reattach to running PTYs after quit).
- Custom themes and user config file (split ratio, keybindings, colors).
- Multiple-project side-by-side view.
- Mouse support beyond resize detection.
- `.env` / dotenv management UI.
- Recipe favorites / custom ordering.
- Collaboration / shared sessions.
