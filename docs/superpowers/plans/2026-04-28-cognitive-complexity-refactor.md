# Cognitive Complexity Refactor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Mechanically split three high-complexity Rust functions so SonarCloud's `rust:S3776` rule reports them under the project threshold of 18, with no behavior change.

**Architecture:** Pure source reorganization. Each target function delegates to free helper functions in the same file; no new modules, no new dependencies, no behavior changes. The compiler enforces signatures and the existing test suite (including `cargo insta` snapshots) guards behavior.

**Tech Stack:** Rust 1.79+, ratatui, vt100, tokio, insta (snapshot tests).

**Spec:** `docs/superpowers/specs/2026-04-28-cognitive-complexity-refactor-design.md`

**Repo conventions:**
- Run all `cargo` and `git` commands from the repo root (`/Users/nick.hartjes/projects/nh/lazyjust-rs`).
- Tests run with `--test-threads=1` because of a pre-existing env-mutex flake in `config::paths`.
- Snapshot tests live under `tests/snapshots/` and must pass without `cargo insta accept`.
- The default branch is `main`; PR target is `main`; squash-merge style commits.

**Conventions used in this plan:**
- Steps ordered: change code → verify locally → commit.
- "Verify" steps name the exact command + an "Expected" line; partial substring match in the output is OK.
- Code blocks show the entire block to write or replace, not diffs.

---

## File Structure

| Path | Change | Responsibility |
|---|---|---|
| `src/config/merge.rs` | Modify | `merge` becomes 4-line dispatcher; `merge_ui`/`merge_paths`/`merge_logging`/`merge_engine` private helpers below. |
| `src/ui/session_pane.rs` | Modify | `render` keeps layout/header/scrollbar orchestration; `cell_to_span` helper handles per-cell span construction. |
| `src/app/reducer.rs` | Modify | `reduce` becomes pure dispatch; one private helper per non-trivial `Action` arm. Existing `cycle_history` and `filtered_justfile_indices` stay put. |

No new files, no module moves.

---

## Task 0: Branch and baseline

Create the working branch and confirm the baseline tests are green so any regression introduced later is unambiguous.

**Files:**
- None modified (branch + verification only)

- [ ] **Step 1: Make sure you're on a clean main**

Run:

```bash
git status
git checkout main
git pull --ff-only origin main
```

Expected: `working tree clean`, branch up to date with `origin/main`.

- [ ] **Step 2: Cut the working branch**

Run:

```bash
git checkout -b refactor/sonarcloud-cognitive-complexity
```

Expected: `Switched to a new branch 'refactor/sonarcloud-cognitive-complexity'`.

- [ ] **Step 3: Run the full local check suite**

Run:

```bash
cargo build
cargo clippy --all-targets -- -D warnings
cargo fmt --all -- --check
cargo test --all-targets -- --test-threads=1
```

Expected: every command exits 0. No FAILED test results.

If any of these fail before any code change, stop — investigate and fix
the baseline before continuing. The rest of the plan assumes a green
baseline so a later regression is attributable to the refactor.

---

## Task 1: Refactor `src/config/merge.rs`

`merge` walks four optional `ConfigFile` sections (`ui`, `paths`, `logging`,
`engine`) inline. Lift each section into its own private helper. The
top-level becomes a four-call dispatcher.

**Files:**
- Modify: `src/config/merge.rs`

- [ ] **Step 1: Replace the top of `merge.rs` (lines 1-56) with the refactored version**

Open `src/config/merge.rs`. Replace everything from `use super::file::ConfigFile;` (line 1) through the closing `}` of `pub fn merge` (line 56, inclusive) with the block below. Leave the `#[cfg(test)] mod tests { ... }` block (line 58 onward) untouched.

```rust
use super::file::{ConfigFile, EngineSection, LoggingSection, PathsSection, UiSection};
use super::Config;
use std::path::PathBuf;
use std::time::Duration;

/// Overlay a parsed file onto a base `Config`, filling missing values
/// from the base. The returned `Config` is ready for use.
pub fn merge(file: ConfigFile, base: Config) -> Config {
    let mut out = base;
    if let Some(u) = file.ui {
        merge_ui(&mut out, u);
    }
    if let Some(p) = file.paths {
        merge_paths(&mut out, p);
    }
    if let Some(l) = file.logging {
        merge_logging(&mut out, l);
    }
    if let Some(e) = file.engine {
        merge_engine(&mut out, e);
    }
    out
}

fn merge_ui(out: &mut Config, u: UiSection) {
    if let Some(theme) = u.theme {
        out.theme_name = theme;
    }
    if let Some(icon) = u.icon_style.as_deref() {
        if let Some(parsed) = crate::ui::icon_style::IconStyle::parse(icon) {
            out.icon_style = parsed;
        } else {
            tracing::warn!(
                target: "lazyjust::config",
                value = %icon,
                "unknown [ui].icon_style, using default",
            );
        }
    }
}

fn merge_paths(out: &mut Config, p: PathsSection) {
    if let Some(d) = p.state_dir {
        out.state_dir = PathBuf::from(d);
    }
    if let Some(d) = p.sessions_log_dir {
        out.sessions_log_dir = PathBuf::from(d);
    }
}

fn merge_logging(out: &mut Config, l: LoggingSection) {
    if let Some(mb) = l.session_log_size_cap_mb {
        out.session_log_size_cap = mb.saturating_mul(1024 * 1024);
    }
    if let Some(days) = l.session_log_retention_days {
        out.session_log_retention = Duration::from_secs(days.saturating_mul(24 * 3600));
    }
}

fn merge_engine(out: &mut Config, e: EngineSection) {
    if let Some(ms) = e.render_throttle_ms {
        out.render_throttle = Duration::from_millis(ms);
    }
    if let Some(ms) = e.tick_interval_ms {
        out.tick_interval = Duration::from_millis(ms);
    }
}
```

Notes for the implementer:
- The new `use` line imports the four section types now referenced by the helpers.
- The `tests` module already imports `EngineSection`, `LoggingSection`, `PathsSection` from `super::file`, so the new top-level `use` will not collide.
- All transforms (`PathBuf::from`, `Duration::from_millis`, `mb * 1024 * 1024`, `days * 24 * 3600`) are byte-for-byte identical to the original.

- [ ] **Step 2: Build to confirm there are no compile errors**

Run:

```bash
cargo build
```

Expected: `Finished `dev` profile`, no errors. Warnings about unused imports mean a `use` was missed — fix and rebuild.

- [ ] **Step 3: Run the eight existing `merge` tests**

Run:

```bash
cargo test --lib config::merge -- --test-threads=1
```

Expected output ends with:

```
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Failed assertions in any of these tests indicate the refactor changed observable behavior — stop and reconcile against the original block before moving on.

- [ ] **Step 4: Run the full suite for safety**

Run:

```bash
cargo clippy --all-targets -- -D warnings
cargo fmt --all -- --check
cargo test --all-targets -- --test-threads=1
```

Expected: all green, no FAILED results.

- [ ] **Step 5: Commit**

Run:

```bash
git add src/config/merge.rs
git commit -m "refactor(config): split merge() into per-section helpers"
```

---

## Task 2: Refactor `src/ui/session_pane.rs`

The complexity hot spot is the inner cell loop in `render` that builds a
`Span` per terminal cell with a five-modifier style accumulation. Extract
that whole inner body into a private `cell_to_span` helper. `render` keeps
the layout, header, scrollbar, and outer iteration.

**Files:**
- Modify: `src/ui/session_pane.rs`

- [ ] **Step 1: Replace the body of `render` (lines 8-83) with the refactored version**

Open `src/ui/session_pane.rs`. Replace `pub fn render(...)` and its body (lines 8-83 inclusive) with the block below. Leave the existing `scrollback_dims` (line 85) and `convert_color` (line 92) helpers untouched.

```rust
pub fn render(
    f: &mut Frame,
    area: Rect,
    screen: &vt100::Parser,
    meta: &SessionMeta,
    active: bool,
    theme: &crate::theme::Theme,
) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(1),
        ])
        .split(area);
    let header_area = rows[0];
    let body_area = rows[2];

    crate::ui::session_header::render(f, header_area, meta, active, theme);

    // reserve last column of body for the scroll thumb
    let grid_area = Rect {
        width: body_area.width.saturating_sub(1),
        ..body_area
    };
    let scroll_area = Rect {
        x: body_area.x + body_area.width.saturating_sub(1),
        y: body_area.y,
        width: 1,
        height: body_area.height,
    };

    let grid = screen.screen();
    let rows_count = grid_area.height as usize;
    let cols = grid_area.width as usize;

    let mut lines = Vec::with_capacity(rows_count);
    for r in 0..rows_count {
        let mut spans = Vec::with_capacity(cols);
        for c in 0..cols {
            spans.push(cell_to_span(grid, r as u16, c as u16));
        }
        lines.push(Line::from(spans));
    }

    let p = Paragraph::new(lines);
    f.render_widget(p, grid_area);

    // scroll thumb from vt100 scrollback
    let (total, top) = scrollback_dims(screen, rows_count);
    let buf = f.buffer_mut();
    crate::ui::scrollbar::render(buf, scroll_area, total, rows_count, top, theme);
}

fn cell_to_span(grid: &vt100::Screen, r: u16, c: u16) -> Span<'static> {
    let Some(cell) = grid.cell(r, c) else {
        return Span::raw(" ");
    };
    let mut style = Style::default();
    if let Some(color) = convert_color(cell.fgcolor()) {
        style = style.fg(color);
    }
    if let Some(color) = convert_color(cell.bgcolor()) {
        style = style.bg(color);
    }
    if cell.bold() {
        style = style.add_modifier(Modifier::BOLD);
    }
    if cell.italic() {
        style = style.add_modifier(Modifier::ITALIC);
    }
    if cell.underline() {
        style = style.add_modifier(Modifier::UNDERLINED);
    }
    let ch = cell.contents();
    let text: String = if ch.is_empty() { " ".into() } else { ch.into() };
    Span::styled(text, style)
}
```

Notes:
- `cell_to_span` is inserted between `render` and the existing `scrollback_dims`. Place it on the line right after `render`'s closing `}`.
- The function takes `&vt100::Screen` (the type returned by `screen.screen()`); pass that, not the `vt100::Parser`, so each call avoids re-deriving the screen.
- Behavior is identical to the original inline body — no logic changed.

- [ ] **Step 2: Build to confirm no compile errors**

Run:

```bash
cargo build
```

Expected: `Finished `dev` profile`, no errors. If a missing `use` is reported (e.g. `Span` or `Modifier`), the existing import block at the top of the file already covers them — verify nothing was deleted by the edit.

- [ ] **Step 3: Run snapshot tests, no acceptance**

Run:

```bash
cargo test --test snapshots -- --test-threads=1
```

Expected output ends with:

```
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Any unexpected `snapshot mismatch` failure means the refactor changed pixel-level rendering. Do **not** run `cargo insta accept`. Compare the offending snapshot diff against the original cell-loop logic and reconcile.

- [ ] **Step 4: Run the full suite for safety**

Run:

```bash
cargo clippy --all-targets -- -D warnings
cargo fmt --all -- --check
cargo test --all-targets -- --test-threads=1
```

Expected: all green.

- [ ] **Step 5: Commit**

Run:

```bash
git add src/ui/session_pane.rs
git commit -m "refactor(ui): extract cell_to_span from session_pane::render"
```

---

## Task 3: Refactor `src/app/reducer.rs`

`reduce` is one giant `match` over the `Action` enum. Extract every
non-trivial arm body into a private free function in the same file.
Trivial arms (single state assignment, `NoOp`, the explicitly-stubbed
`Quit | ConfirmQuit`) stay inline.

**Files:**
- Modify: `src/app/reducer.rs`

- [ ] **Step 1: Replace `reduce` (lines 10-347) with dispatcher + helpers**

Open `src/app/reducer.rs`. Replace everything from the `#[allow(...)]` attribute on line 10 through the closing `}` of `pub fn reduce` on line 347 with the block below. Leave lines 1-9 (uses + constants), line 349 onward (`filtered_justfile_indices`, `cycle_history`, the test module) intact.

```rust
pub fn reduce(app: &mut App, action: Action) {
    match action {
        Action::NoOp => {}

        Action::CursorDown => cursor_down(app),
        Action::CursorUp => cursor_up(app),

        Action::EnterFilter => app.mode = Mode::FilterInput,
        Action::FilterChar(c) => filter_char(app, c),
        Action::FilterBackspace => filter_backspace(app),
        Action::CommitFilter => commit_filter(app),
        Action::CancelFilter => cancel_filter(app),

        Action::GrowLeftPane => grow_left_pane(app),
        Action::ShrinkLeftPane => shrink_left_pane(app),
        Action::ResetSplit => app.split_ratio = SPLIT_DEFAULT,

        Action::RequestQuit => request_quit(app),
        Action::CancelConfirm => app.mode = Mode::Normal,
        Action::Quit | Action::ConfirmQuit => {
            // handled by event loop — reducer leaves it as signal
        }

        Action::OpenHelp => open_help(app),
        Action::CloseHelp => app.mode = Mode::Normal,
        Action::HelpScrollDown(n) => help_scroll_down(app, n),
        Action::HelpScrollUp(n) => help_scroll_up(app, n),
        Action::HelpScrollHome => help_scroll_home(app),
        Action::HelpScrollEnd => help_scroll_end(app),

        Action::OpenErrors => app.mode = Mode::ErrorsList,
        Action::CloseErrors => app.mode = Mode::Normal,

        Action::OpenDropdown => open_dropdown(app),
        Action::DropdownChar(c) => dropdown_char(app, c),
        Action::DropdownBackspace => dropdown_backspace(app),
        Action::DropdownCursorDown => dropdown_cursor_down(app),
        Action::DropdownCursorUp => dropdown_cursor_up(app),
        Action::SelectDropdown => select_dropdown(app),
        Action::CancelDropdown => app.mode = Mode::Normal,

        Action::SessionExited { id, code } => session_exited(app, id, code),
        Action::RecipeExited { id, code } => recipe_exited(app, id, code),
        Action::MarkUnread(id) => mark_unread(app, id),
        Action::MarkRead(id) => mark_read(app, id),

        Action::CycleFocus => cycle_focus(app),
        Action::FocusList => app.focus = crate::app::types::Focus::List,
        Action::FocusSession => app.focus = crate::app::types::Focus::Session,
        Action::FocusNextSession => focus_next_session(app),
        Action::FocusPrevSession => focus_prev_session(app),

        Action::ParamChar(c) => param_char(app, c),
        Action::ParamBackspace => param_backspace(app),
        Action::ParamNext => param_next(app),
        Action::CancelParam => app.mode = Mode::Normal,
        // ParamCommit handled by event_loop (needs side effects)
        Action::CycleRecipeHistoryPrev => cycle_history(app, -1),
        Action::CycleRecipeHistoryNext => cycle_history(app, 1),

        Action::RequestKillSession => request_kill_session(app),
        Action::RequestCloseSession => request_close_session(app),
        Action::KillSession(id) => kill_session(app, id),
        Action::CloseSession(id) => close_session(app, id),
        Action::CopyLogPath => copy_log_path(app),

        Action::OpenThemePicker => open_theme_picker(app),
        Action::PickerMove(delta) => picker_move(app, delta),
        Action::PickerConfirm => picker_confirm(app),
        Action::PickerCancel => picker_cancel(app),

        // Remaining actions handled in later tasks.
        _ => {}
    }
}

fn cursor_down(app: &mut App) {
    if let Some(jf) = app.active_justfile() {
        let max = jf.recipes.len().saturating_sub(1);
        if app.list_cursor < max {
            app.list_cursor += 1;
        }
    }
}

fn cursor_up(app: &mut App) {
    app.list_cursor = app.list_cursor.saturating_sub(1);
}

fn filter_char(app: &mut App, c: char) {
    if app.mode == Mode::FilterInput {
        app.filter.push(c);
    }
}

fn filter_backspace(app: &mut App) {
    if app.mode == Mode::FilterInput {
        app.filter.pop();
    }
}

fn commit_filter(app: &mut App) {
    if app.mode == Mode::FilterInput {
        app.mode = Mode::Normal;
    }
}

fn cancel_filter(app: &mut App) {
    app.filter.clear();
    app.mode = Mode::Normal;
}

fn grow_left_pane(app: &mut App) {
    app.split_ratio = (app.split_ratio + SPLIT_STEP).min(SPLIT_MAX);
}

fn shrink_left_pane(app: &mut App) {
    app.split_ratio = (app.split_ratio - SPLIT_STEP).max(SPLIT_MIN);
}

fn request_quit(app: &mut App) {
    if app.sessions.iter().any(|s| {
        matches!(
            s.status,
            crate::app::types::Status::Running
                | crate::app::types::Status::ShellAfterExit { .. }
        )
    }) {
        app.mode = Mode::Confirm {
            prompt: "Sessions running. Quit & kill all?".into(),
            on_accept: ConfirmAction::QuitKillAll,
        };
    } else {
        // caller handles actual quit after reducer by checking Quit elsewhere.
        app.mode = Mode::Normal;
    }
}

fn open_help(app: &mut App) {
    let origin = crate::app::help_section::active_section(app);
    app.mode = Mode::Help { scroll: 0, origin };
}

fn help_scroll_down(app: &mut App, n: u16) {
    if let Mode::Help { scroll, .. } = &mut app.mode {
        *scroll = scroll.saturating_add(n);
    }
}

fn help_scroll_up(app: &mut App, n: u16) {
    if let Mode::Help { scroll, .. } = &mut app.mode {
        *scroll = scroll.saturating_sub(n);
    }
}

fn help_scroll_home(app: &mut App) {
    if let Mode::Help { scroll, .. } = &mut app.mode {
        *scroll = 0;
    }
}

fn help_scroll_end(app: &mut App) {
    if let Mode::Help { scroll, .. } = &mut app.mode {
        *scroll = u16::MAX;
    }
}

fn open_dropdown(app: &mut App) {
    app.mode = Mode::Dropdown {
        filter: String::new(),
        cursor: app.active_justfile,
    };
}

fn dropdown_char(app: &mut App, c: char) {
    if let Mode::Dropdown { filter, cursor } = &mut app.mode {
        filter.push(c);
        *cursor = 0;
    }
}

fn dropdown_backspace(app: &mut App) {
    if let Mode::Dropdown { filter, .. } = &mut app.mode {
        filter.pop();
    }
}

fn dropdown_cursor_down(app: &mut App) {
    let max = app.justfiles.len().saturating_sub(1);
    if let Mode::Dropdown { cursor, .. } = &mut app.mode {
        if *cursor < max {
            *cursor += 1;
        }
    }
}

fn dropdown_cursor_up(app: &mut App) {
    if let Mode::Dropdown { cursor, .. } = &mut app.mode {
        *cursor = cursor.saturating_sub(1);
    }
}

fn select_dropdown(app: &mut App) {
    if let Mode::Dropdown { cursor, filter } = app.mode.clone() {
        let filtered = filtered_justfile_indices(app, &filter);
        if let Some(&chosen) = filtered.get(cursor) {
            app.active_justfile = chosen;
            app.list_cursor = 0;
            app.filter.clear();
        }
        app.mode = Mode::Normal;
    }
}

fn session_exited(app: &mut App, id: crate::app::types::SessionId, code: i32) {
    if let Some(s) = app.session_mut(id) {
        if matches!(s.status, crate::app::types::Status::Running) {
            s.status = crate::app::types::Status::Exited { code };
            s.unread = true;
        } else if let crate::app::types::Status::ShellAfterExit { .. } = s.status {
            s.status = crate::app::types::Status::Exited { code };
        }
    }
}

fn recipe_exited(app: &mut App, id: crate::app::types::SessionId, code: i32) {
    let is_active = Some(id) == app.active_session;
    if let Some(s) = app.session_mut(id) {
        s.status = crate::app::types::Status::ShellAfterExit { code };
        if !is_active {
            s.unread = true;
        }
    }
}

fn mark_unread(app: &mut App, id: crate::app::types::SessionId) {
    if let Some(s) = app.session_mut(id) {
        s.unread = true;
    }
}

fn mark_read(app: &mut App, id: crate::app::types::SessionId) {
    if let Some(s) = app.session_mut(id) {
        s.unread = false;
    }
}

fn cycle_focus(app: &mut App) {
    app.focus = match app.focus {
        crate::app::types::Focus::List => crate::app::types::Focus::Session,
        crate::app::types::Focus::Session => crate::app::types::Focus::List,
        other => other,
    };
}

fn focus_next_session(app: &mut App) {
    let ids: Vec<_> = app.sessions.iter().map(|s| s.id).collect();
    if let Some(cur) = app.active_session {
        if let Some(i) = ids.iter().position(|id| *id == cur) {
            if let Some(next) = ids.get(i + 1) {
                app.active_session = Some(*next);
                if let Some(s) = app.session_mut(*next) {
                    s.unread = false;
                }
            }
        }
    } else if let Some(first) = ids.first() {
        app.active_session = Some(*first);
    }
}

fn focus_prev_session(app: &mut App) {
    let ids: Vec<_> = app.sessions.iter().map(|s| s.id).collect();
    if let Some(cur) = app.active_session {
        if let Some(i) = ids.iter().position(|id| *id == cur) {
            if i > 0 {
                let prev = ids[i - 1];
                app.active_session = Some(prev);
                if let Some(s) = app.session_mut(prev) {
                    s.unread = false;
                }
            }
        }
    }
}

fn param_char(app: &mut App, c: char) {
    if let Mode::ParamInput { values, cursor, .. } = &mut app.mode {
        if let Some(v) = values.get_mut(*cursor) {
            v.push(c);
        }
    }
}

fn param_backspace(app: &mut App) {
    if let Mode::ParamInput { values, cursor, .. } = &mut app.mode {
        if let Some(v) = values.get_mut(*cursor) {
            v.pop();
        }
    }
}

fn param_next(app: &mut App) {
    if let Mode::ParamInput { values, cursor, .. } = &mut app.mode {
        if *cursor + 1 < values.len() {
            *cursor += 1;
        }
    }
}

fn request_kill_session(app: &mut App) {
    if let Some(id) = app.active_session {
        app.mode = Mode::Confirm {
            prompt: format!("Kill session {id}?"),
            on_accept: crate::app::types::ConfirmAction::KillSession(id),
        };
    }
}

fn request_close_session(app: &mut App) {
    if let Some(id) = app.active_session {
        app.mode = Mode::Confirm {
            prompt: format!("Close session {id}?"),
            on_accept: crate::app::types::ConfirmAction::CloseSession(id),
        };
    }
}

fn kill_session(app: &mut App, id: crate::app::types::SessionId) {
    if let Some(s) = app.session_mut(id) {
        s.status = crate::app::types::Status::Exited { code: 130 };
    }
    // actual PTY kill done in event loop
}

fn close_session(app: &mut App, id: crate::app::types::SessionId) {
    app.sessions.retain(|s| s.id != id);
    if app.active_session == Some(id) {
        app.active_session = None;
    }
    for jf in &mut app.justfiles {
        for r in &mut jf.recipes {
            r.runs.retain(|rid| *rid != id);
        }
    }
}

fn copy_log_path(app: &mut App) {
    if let Some(id) = app.active_session {
        if let Some(s) = app.session(id) {
            if let Ok(mut cb) = arboard::Clipboard::new() {
                let _ = cb.set_text(s.log_path.display().to_string());
                app.status_message = Some(format!("copied {}", s.log_path.display()));
            }
        }
    }
}

fn open_theme_picker(app: &mut App) {
    let names = crate::theme::registry::list();
    let original_name = app.theme_name.clone();
    let highlighted = names.iter().position(|n| *n == original_name).unwrap_or(0);
    app.mode = Mode::ThemePicker {
        original_name,
        highlighted,
        names,
    };
}

fn picker_move(app: &mut App, delta: isize) {
    if let Mode::ThemePicker {
        highlighted, names, ..
    } = &mut app.mode
    {
        let len = names.len() as isize;
        if len > 0 {
            let mut idx = *highlighted as isize + delta;
            idx = idx.rem_euclid(len);
            *highlighted = idx as usize;
            let stem = names[*highlighted].clone();
            app.theme = crate::theme::registry::resolve(&stem);
            app.theme_name = stem;
        }
    }
}

fn picker_confirm(app: &mut App) {
    if let Mode::ThemePicker { .. } = app.mode {
        let stem = app.theme_name.clone();
        let path = crate::config::paths::config_file_path();
        if let Err(e) = crate::config::writer::set_theme(&path, &stem) {
            tracing::warn!(
                target: "lazyjust::theme",
                error = %e,
                "failed to persist theme",
            );
            app.status_message = Some(format!("theme persist failed: {e}"));
        }
        app.mode = Mode::Normal;
    }
}

fn picker_cancel(app: &mut App) {
    // Separate scope so we can re-borrow app immutably after restoring.
    let original = if let Mode::ThemePicker { original_name, .. } = &app.mode {
        Some(original_name.clone())
    } else {
        None
    };
    if let Some(original) = original {
        app.theme = crate::theme::registry::resolve(&original);
        app.theme_name = original;
        app.mode = Mode::Normal;
    }
}
```

Notes for the implementer:
- The `#[allow(clippy::collapsible_if, clippy::collapsible_match)]` attribute on the original `reduce` is intentionally dropped. If clippy reports either of those lints inside any of the new helpers, **first** confirm the lint genuinely fires on a helper (not another file), then add the attribute back on that specific helper. Don't reapply it preemptively.
- Helper bodies are lifted verbatim from the original arm bodies — don't rephrase or reorder mutations.
- The trailing `_ => {}` wildcard in `reduce` stays. It catches the `Action` variants explicitly stubbed in the original (`ToggleGroupCollapse`, `RunHighlighted`, `ParamCommit`, `Confirm`) plus future-added variants until they get their own arm.
- `cycle_history` and `filtered_justfile_indices` (lines 349 onward in the pre-edit file) are reused as-is; no changes.

- [ ] **Step 2: Build to confirm no compile errors**

Run:

```bash
cargo build
```

Expected: `Finished `dev` profile`, no errors.

- [ ] **Step 3: Run reducer-targeted tests**

Run:

```bash
cargo test --lib reducer -- --test-threads=1
cargo test --test reducer_tests -- --test-threads=1
```

Expected: every test passes. Both commands print a `test result: ok.` summary line. `cargo test --lib reducer` runs the four `theme_picker_tests` defined inline; `cargo test --test reducer_tests` runs the 15 tests in `tests/reducer_tests.rs`.

- [ ] **Step 4: Run snapshot + full suite**

Run:

```bash
cargo clippy --all-targets -- -D warnings
cargo fmt --all -- --check
cargo test --all-targets -- --test-threads=1
```

Expected: all green. If clippy fires `collapsible_if` or `collapsible_match` on a specific helper, add `#[allow(...)]` to that one helper only. Re-run clippy; expected green.

- [ ] **Step 5: Commit**

Run:

```bash
git add src/app/reducer.rs
git commit -m "refactor(reducer): dispatch each Action variant to a free fn"
```

---

## Task 4: Push and open the PR

Push the branch and open a PR targeting `main`. CI must go green and SonarCloud must report all three findings as resolved on the next scan.

**Files:**
- None modified (push + PR only).

- [ ] **Step 1: Push the branch**

Run:

```bash
git push -u origin refactor/sonarcloud-cognitive-complexity
```

Expected: `* [new branch] refactor/sonarcloud-cognitive-complexity -> refactor/sonarcloud-cognitive-complexity` and `branch 'refactor/sonarcloud-cognitive-complexity' set up to track ...`.

- [ ] **Step 2: Open the PR**

Run:

```bash
gh pr create --title "refactor: split high-complexity functions per SonarCloud findings" --body "$(cat <<'EOF'
## Summary

Mechanical split of three functions flagged by SonarCloud `rust:S3776`
(Cognitive Complexity) under the project's threshold of 18. No behavior
change.

| File | Function | Before | After (target) |
|---|---|---:|---:|
| `src/config/merge.rs` | `merge` | 24 | ~4 |
| `src/ui/session_pane.rs` | `render` | 32 | ~10 |
| `src/app/reducer.rs` | `reduce` | 137 | dispatch + per-arm helpers |

## What changed

- `merge` now delegates to four private helpers (`merge_ui`, `merge_paths`, `merge_logging`, `merge_engine`) — one per `ConfigFile` section. Transforms (`PathBuf::from`, `Duration::from_millis`, MB→bytes, days→seconds) are byte-for-byte identical to the original.
- `session_pane::render` extracts the inner cell loop into `cell_to_span(grid, r, c) -> Span<'static>`. The five `Style` modifiers (fg/bg/bold/italic/underline) and the empty-cell `" "` fallback move with it.
- `reducer::reduce` now consists of `match` + one-line dispatch per arm. Each non-trivial body is a private free function in the same file. `Action::Quit | Action::ConfirmQuit`, `NoOp`, single-assignment arms (`app.mode = Mode::X`) stay inline. The `_ => {}` wildcard for `ToggleGroupCollapse` / `RunHighlighted` / `ParamCommit` / `Confirm(...)` is preserved.

## Verification

- `cargo build`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `cargo test --all-targets -- --test-threads=1` all green locally.
- 20 snapshots in `tests/snapshots/` pass without `cargo insta accept`.
- 15 tests in `tests/reducer_tests.rs` and 4 inline `theme_picker_tests` pass.
- 8 inline tests in `src/config/merge.rs::tests` pass.

## Test plan

- [ ] CI matrix passes (test + flake-check on linux + macos + windows where applicable)
- [ ] SonarQube workflow runs on push to main; the three S3776 findings drop off the project dashboard
- [ ] If any function still scores over 18, follow up with the bonus tactic noted in the spec (Modifier bitmask in `cell_to_span`, or further split in `reduce`)
EOF
)"
```

Expected: a PR URL is printed (e.g. `https://github.com/nickhartjes/lazyjust/pull/<n>`).

- [ ] **Step 3: Watch CI go green**

Run:

```bash
gh pr checks <pr-number> --repo nickhartjes/lazyjust
```

Replace `<pr-number>` with the PR number from Step 2's output.

Expected: every row eventually shows `pass`. If anything fails, do **not** merge — fix locally and push (which triggers a re-run).

- [ ] **Step 4: After merge, verify SonarCloud**

Once the PR is merged and SonarQube workflow runs on `main`:

1. Open the SonarCloud project page.
2. Filter Issues by rule `rust:S3776`.
3. Expected: zero open findings (or at most: a remaining one if a function still crosses 18). Each previously open issue should now be marked `RESOLVED`.

If any function remains over 18, apply the spec's stated fallbacks:
- `cell_to_span` over 18 → collapse the three `Modifier` `if`s into a single bitmask, as shown in the spec under Plan B.
- `reduce` dispatcher over 18 → group arms by domain into trivial intermediate dispatchers (cursor, filter, dropdown, session, theme, etc.). Open a follow-up PR; do not amend this one after merge.

---

## Risks (carried from the spec)

- **Mutation reordering inside an arm body.** Mitigation: helpers lift each arm body verbatim. Don't refactor while extracting.
- **`#[allow]` lifted off `reduce`.** Mitigation: clippy step in Task 3 catches any reintroduced lint; reapply per-helper if needed.
- **Snapshot test flake.** Mitigation: snapshots are stable on this codebase; investigate any diff before considering acceptance.
- **`reduce` dispatcher still over 18.** Mitigation: planned fallback (group arms by domain) deferred to follow-up if and only if Sonar confirms.

## Out of scope

- Submodule reorganization of `reducer.rs`.
- New tests for `Action` variants without dedicated coverage.
- The three borderline files (`ui/list.rs`, `session/reader.rs`, `theme/registry.rs`) now under the 18 threshold.
