# Rich Help Modal Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the static 22-line help modal with a structured, scrollable, full-coverage help screen that documents every input mode, is reachable from session focus via a global `F1` shortcut, and highlights the section matching the user's pre-help state.

**Architecture:** A new `src/app/help_section.rs` defines `SectionId` (an app-layer enum that `Mode::Help` carries as an `origin` field) plus an `active_section` resolver that maps `(Mode, Focus)` → `SectionId`. A new `src/ui/help.rs` holds the help content as `const SECTIONS: &[Section]` plus the render function. `Mode::Help` grows a `scroll: u16` field. Four new actions (`HelpScrollUp`, `HelpScrollDown`, `HelpScrollHome`, `HelpScrollEnd`) drive scrolling. Event-loop layer gets a universal `F1` intercept so session-focus doesn't eat the key. Module direction stays clean: UI depends on app; app does not depend on UI.

**Tech Stack:** Rust, `ratatui`, existing `lazyjust` crate structure. No new deps.

**Spec:** `docs/superpowers/specs/2026-04-24-rich-help-modal-design.md`

---

## File Structure

| File | Action | Responsibility after this branch |
|---|---|---|
| `src/app/help_section.rs` | Create | `SectionId` enum + `active_section(app) -> SectionId` resolver. App-layer, no UI deps. |
| `src/app/mod.rs` | Modify | Add `pub mod help_section;` and re-export `SectionId` via `pub use help_section::SectionId;`. |
| `src/app/types.rs` | Modify | `Mode::Help` grows `{ scroll: u16, origin: SectionId }`. |
| `src/app/action.rs` | Modify | Add `HelpScrollUp(u16)`, `HelpScrollDown(u16)`, `HelpScrollHome`, `HelpScrollEnd`. |
| `src/app/reducer.rs` | Modify | `OpenHelp` snapshots `active_section(&app)` into `Mode::Help`; add the four scroll arms. |
| `src/input/keymap.rs` | Modify | `Mode::Help { .. }` pattern; `help_mode` handles scroll keys. |
| `src/app/event_loop.rs` | Modify | Global `F1` intercept ahead of session-forward-to-PTY. |
| `src/ui/help.rs` | Create | `const SECTIONS: &[Section]`, `pub fn render(f, app, area)`, inline unit tests for layout. |
| `src/ui/mod.rs` | Modify | `pub mod help;`. |
| `src/ui/modal.rs` | Modify | Delete `render_help`; dispatch `Mode::Help { .. }` to `help::render`. |
| `src/ui/status_bar.rs` | Modify | Update `Mode::Help` pattern (now carries fields). |
| `tests/reducer_tests.rs` | Modify | Add help-mode reducer cases. |
| `tests/snapshots.rs` | Modify | Add `help_modal_list_focus_render_snapshot`. |

Module direction: `ui/help.rs` depends on `app::help_section::SectionId`. `app/help_section.rs` has zero UI dependency. Nothing in `app/*` imports from `ui/*`.

---

## Task 1: `SectionId` + `active_section` resolver

**Files:**
- Create: `src/app/help_section.rs`
- Modify: `src/app/mod.rs`

- [ ] **Step 1: Create `src/app/help_section.rs`**

Write:

```rust
use crate::app::types::{Focus, Mode};
use crate::app::App;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum SectionId {
    ListFocus,
    SessionFocus,
    Filter,
    Dropdown,
    Param,
    Confirm,
    Errors,
    HelpItself,
}

pub fn active_section(app: &App) -> SectionId {
    match &app.mode {
        Mode::Help { .. } => SectionId::HelpItself,
        Mode::FilterInput => SectionId::Filter,
        Mode::Dropdown { .. } => SectionId::Dropdown,
        Mode::ParamInput { .. } => SectionId::Param,
        Mode::Confirm { .. } => SectionId::Confirm,
        Mode::ErrorsList => SectionId::Errors,
        Mode::Normal => match app.focus {
            Focus::Session => SectionId::SessionFocus,
            _ => SectionId::ListFocus,
        },
    }
}
```

Note: `active_section` returns `HelpItself` when called *from inside* help. The `OpenHelp` reducer (Task 3) computes `active_section` *before* transitioning, so it captures the pre-help state. Callers after transition don't need the origin — it's already stored in `Mode::Help::origin`.

- [ ] **Step 2: Register the module in `src/app/mod.rs`**

Replace the contents of `src/app/mod.rs` with:

```rust
pub mod action;
pub mod event_loop;
pub mod filter;
pub mod help_section;
pub mod reducer;
pub mod state;
pub mod types;

pub use action::{Action, AppEvent};
pub use help_section::SectionId;
pub use state::App;
pub use types::*;
```

- [ ] **Step 3: Verify the crate compiles**

Run: `cargo build`
Expected: `Finished` with no errors. `active_section` compiles against the current `Mode::Help` (the no-field variant); Task 2 will flip the signature.

- [ ] **Step 4: Full tests + lint**

Run: `cargo test` → every target `test result: ok`.
Run: `cargo clippy --all-targets -- -D warnings` → clean.
Run: `cargo fmt --check` → clean.

- [ ] **Step 5: Commit**

```bash
git add src/app/help_section.rs src/app/mod.rs
git commit -m "feat(app): add SectionId enum + active_section resolver"
```

---

## Task 2: Reshape `Mode::Help` to carry `scroll` + `origin`

**Files:**
- Modify: `src/app/types.rs`
- Modify: `src/app/reducer.rs`
- Modify: `src/input/keymap.rs`
- Modify: `src/ui/modal.rs`
- Modify: `src/ui/status_bar.rs`

Atomic: the enum shape change forces every match site to update in the same commit.

- [ ] **Step 1: Edit `src/app/types.rs`**

Locate:

```rust
    Help,
```

Inside `pub enum Mode`. Replace with:

```rust
    Help { scroll: u16, origin: crate::app::help_section::SectionId },
```

- [ ] **Step 2: Edit `src/app/reducer.rs`**

Locate:

```rust
        Action::OpenHelp => app.mode = Mode::Help,
```

Replace with:

```rust
        Action::OpenHelp => {
            let origin = crate::app::help_section::active_section(app);
            app.mode = Mode::Help { scroll: 0, origin };
        }
```

`Action::CloseHelp` stays unchanged (already sets `Mode::Normal`).

- [ ] **Step 3: Edit `src/input/keymap.rs`**

Locate:

```rust
        Mode::Help => help_mode(key),
```

Replace with:

```rust
        Mode::Help { .. } => help_mode(key),
```

- [ ] **Step 4: Edit `src/ui/modal.rs`**

Locate:

```rust
        Mode::Help => render_help(f),
```

Replace with:

```rust
        Mode::Help { .. } => render_help(f),
```

(The `render_help` fn itself stays as-is in this task; Task 5 replaces it with the new `help::render`.)

- [ ] **Step 5: Edit `src/ui/status_bar.rs`**

Locate:

```rust
        Mode::Help => "Help — Esc / q to close".into(),
```

Replace with:

```rust
        Mode::Help { .. } => "Help — Esc / q to close".into(),
```

- [ ] **Step 6: Verify compile, tests, lint**

Run: `cargo build` → `Finished` no errors.
Run: `cargo test` → every target `ok`.
Run: `cargo clippy --all-targets -- -D warnings` → clean.
Run: `cargo fmt --check` → clean.

- [ ] **Step 7: Commit**

```bash
git add src/app/types.rs src/app/reducer.rs src/input/keymap.rs src/ui/modal.rs src/ui/status_bar.rs
git commit -m "refactor(app): Mode::Help carries scroll + origin"
```

---

## Task 3: Scroll actions + reducer arms + reducer tests

**Files:**
- Modify: `src/app/action.rs`
- Modify: `src/app/reducer.rs`
- Modify: `tests/reducer_tests.rs`

- [ ] **Step 1: Write the failing reducer tests**

Append to `tests/reducer_tests.rs`:

```rust
#[test]
fn help_open_from_list_records_origin_list_focus() {
    use lazyjust::app::help_section::SectionId;
    use lazyjust::app::types::Focus;
    let mut app = make_app();
    app.focus = Focus::List;
    reduce(&mut app, Action::OpenHelp);
    match app.mode {
        Mode::Help { scroll, origin } => {
            assert_eq!(scroll, 0);
            assert_eq!(origin, SectionId::ListFocus);
        }
        other => panic!("expected Mode::Help, got {other:?}"),
    }
}

#[test]
fn help_open_from_filter_records_origin_filter() {
    use lazyjust::app::help_section::SectionId;
    let mut app = make_app();
    app.mode = Mode::FilterInput;
    reduce(&mut app, Action::OpenHelp);
    match app.mode {
        Mode::Help { origin, .. } => assert_eq!(origin, SectionId::Filter),
        other => panic!("expected Mode::Help, got {other:?}"),
    }
}

#[test]
fn help_scroll_down_monotonic() {
    use lazyjust::app::help_section::SectionId;
    let mut app = make_app();
    app.mode = Mode::Help { scroll: 0, origin: SectionId::ListFocus };
    reduce(&mut app, Action::HelpScrollDown(1));
    reduce(&mut app, Action::HelpScrollDown(1));
    reduce(&mut app, Action::HelpScrollDown(1));
    match app.mode {
        Mode::Help { scroll, .. } => assert_eq!(scroll, 3),
        _ => panic!("not Help"),
    }
}

#[test]
fn help_scroll_up_floors_zero() {
    use lazyjust::app::help_section::SectionId;
    let mut app = make_app();
    app.mode = Mode::Help { scroll: 2, origin: SectionId::ListFocus };
    reduce(&mut app, Action::HelpScrollUp(5));
    match app.mode {
        Mode::Help { scroll, .. } => assert_eq!(scroll, 0),
        _ => panic!("not Help"),
    }
}

#[test]
fn help_scroll_home_zeroes() {
    use lazyjust::app::help_section::SectionId;
    let mut app = make_app();
    app.mode = Mode::Help { scroll: 42, origin: SectionId::ListFocus };
    reduce(&mut app, Action::HelpScrollHome);
    match app.mode {
        Mode::Help { scroll, .. } => assert_eq!(scroll, 0),
        _ => panic!("not Help"),
    }
}

#[test]
fn help_scroll_end_saturates_max() {
    use lazyjust::app::help_section::SectionId;
    let mut app = make_app();
    app.mode = Mode::Help { scroll: 0, origin: SectionId::ListFocus };
    reduce(&mut app, Action::HelpScrollEnd);
    match app.mode {
        Mode::Help { scroll, .. } => assert_eq!(scroll, u16::MAX),
        _ => panic!("not Help"),
    }
}

#[test]
fn help_close_returns_to_normal() {
    use lazyjust::app::help_section::SectionId;
    let mut app = make_app();
    app.mode = Mode::Help { scroll: 5, origin: SectionId::ListFocus };
    reduce(&mut app, Action::CloseHelp);
    assert_eq!(app.mode, Mode::Normal);
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --test reducer_tests`
Expected: compile errors (the four `HelpScroll*` action variants don't exist yet) OR assertion failures.

- [ ] **Step 3: Add the four action variants**

In `src/app/action.rs`, inside `pub enum Action`, locate:

```rust
    OpenHelp,
    CloseHelp,
```

Insert four new variants immediately after `CloseHelp`:

```rust
    OpenHelp,
    CloseHelp,
    HelpScrollUp(u16),
    HelpScrollDown(u16),
    HelpScrollHome,
    HelpScrollEnd,
```

- [ ] **Step 4: Add the reducer arms**

In `src/app/reducer.rs`, locate the block:

```rust
        Action::OpenHelp => {
            let origin = crate::app::help_section::active_section(app);
            app.mode = Mode::Help { scroll: 0, origin };
        }
        Action::CloseHelp => app.mode = Mode::Normal,
```

Extend it with four new arms:

```rust
        Action::OpenHelp => {
            let origin = crate::app::help_section::active_section(app);
            app.mode = Mode::Help { scroll: 0, origin };
        }
        Action::CloseHelp => app.mode = Mode::Normal,
        Action::HelpScrollDown(n) => {
            if let Mode::Help { scroll, .. } = &mut app.mode {
                *scroll = scroll.saturating_add(n);
            }
        }
        Action::HelpScrollUp(n) => {
            if let Mode::Help { scroll, .. } = &mut app.mode {
                *scroll = scroll.saturating_sub(n);
            }
        }
        Action::HelpScrollHome => {
            if let Mode::Help { scroll, .. } = &mut app.mode {
                *scroll = 0;
            }
        }
        Action::HelpScrollEnd => {
            if let Mode::Help { scroll, .. } = &mut app.mode {
                *scroll = u16::MAX;
            }
        }
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test --test reducer_tests`
Expected: `test result: ok`. All new help tests pass.

- [ ] **Step 6: Full suite + lint**

Run: `cargo test` → every target `ok`.
Run: `cargo clippy --all-targets -- -D warnings` → clean.
Run: `cargo fmt --check` → clean.

- [ ] **Step 7: Commit**

```bash
git add src/app/action.rs src/app/reducer.rs tests/reducer_tests.rs
git commit -m "feat(app): help-mode scroll actions + reducer arms"
```

---

## Task 4: Keymap scroll bindings

**Files:**
- Modify: `src/input/keymap.rs`

- [ ] **Step 1: Update `help_mode`**

In `src/input/keymap.rs`, locate:

```rust
fn help_mode(k: &KeyEvent) -> Option<Action> {
    match k.code {
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') => Some(Action::CloseHelp),
        _ => None,
    }
}
```

Replace the entire fn with:

```rust
fn help_mode(k: &KeyEvent) -> Option<Action> {
    match k.code {
        KeyCode::Esc
        | KeyCode::Char('q')
        | KeyCode::Char('?')
        | KeyCode::F(1) => Some(Action::CloseHelp),
        KeyCode::Char('j') | KeyCode::Down => Some(Action::HelpScrollDown(1)),
        KeyCode::Char('k') | KeyCode::Up => Some(Action::HelpScrollUp(1)),
        KeyCode::PageDown => Some(Action::HelpScrollDown(10)),
        KeyCode::PageUp => Some(Action::HelpScrollUp(10)),
        KeyCode::Home => Some(Action::HelpScrollHome),
        KeyCode::End => Some(Action::HelpScrollEnd),
        _ => None,
    }
}
```

- [ ] **Step 2: Verify compile + tests + lint**

Run: `cargo build`
Run: `cargo test`
Run: `cargo clippy --all-targets -- -D warnings`
Run: `cargo fmt --check`
Expected: all green.

- [ ] **Step 3: Commit**

```bash
git add src/input/keymap.rs
git commit -m "feat(input): help-mode scroll + F1 close bindings"
```

---

## Task 5: UI content + render function + modal dispatch

**Files:**
- Create: `src/ui/help.rs`
- Modify: `src/ui/mod.rs`
- Modify: `src/ui/modal.rs`

- [ ] **Step 1: Create `src/ui/help.rs`**

Write:

```rust
//! Rich help modal: full keybinding reference with scrolling and
//! active-section highlighting. Content is a `const` table; the
//! section matching `Mode::Help::origin` is drawn cyan + bold.

use crate::app::help_section::SectionId;
use crate::app::types::Mode;
use crate::app::App;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

pub struct Entry {
    pub keys: &'static str,
    pub desc: &'static str,
}

pub struct Section {
    pub id: SectionId,
    pub title: &'static str,
    pub entries: &'static [Entry],
}

pub const SECTIONS: &[Section] = &[
    Section {
        id: SectionId::ListFocus,
        title: "List focus",
        entries: &[
            Entry { keys: "j / k, ↓ / ↑", desc: "move cursor within recipes list" },
            Entry { keys: "h / l, ← / →", desc: "cycle through the recipe's previous run sessions" },
            Entry { keys: "Enter", desc: "run recipe — if a session is already running for it, jump there" },
            Entry { keys: "Shift+Enter / r", desc: "always spawn a new session (never reuse)" },
            Entry { keys: "/", desc: "enter fuzzy-filter mode" },
            Entry { keys: "d", desc: "open justfile-switcher dropdown" },
            Entry { keys: "Tab", desc: "cycle focus between list and session pane" },
            Entry { keys: "K", desc: "kill the focused session (confirms)" },
            Entry { keys: "x", desc: "close the focused session (confirms)" },
            Entry { keys: "Ctrl+o / Ctrl+i", desc: "jump to next / previous session with unread output" },
            Entry { keys: "L", desc: "copy the focused session's log path" },
            Entry { keys: "> / < / =", desc: "grow / shrink / reset the left pane width" },
            Entry { keys: "F1 / ?", desc: "open this help" },
            Entry { keys: "e", desc: "open the startup-errors modal" },
            Entry { keys: "q", desc: "quit — confirms if sessions are running" },
        ],
    },
    Section {
        id: SectionId::SessionFocus,
        title: "Session focus",
        entries: &[
            Entry { keys: "F12 / Ctrl+g", desc: "return focus to the recipes list" },
            Entry { keys: "PgUp / PgDn", desc: "scroll session output up / down" },
            Entry { keys: "Home / End", desc: "jump to top / bottom of scrollback" },
            Entry { keys: "(all other keys)", desc: "forwarded to the running shell as typed input" },
            Entry { keys: "F1", desc: "open help (globally intercepted; does not reach the shell)" },
        ],
    },
    Section {
        id: SectionId::Filter,
        title: "Filter mode (after /)",
        entries: &[
            Entry { keys: "a–z, 0–9, …", desc: "extend the filter pattern" },
            Entry { keys: "Backspace", desc: "remove last character" },
            Entry { keys: "Enter", desc: "commit filter and return to list" },
            Entry { keys: "Esc", desc: "discard filter and return to list" },
        ],
    },
    Section {
        id: SectionId::Dropdown,
        title: "Justfile dropdown (after d)",
        entries: &[
            Entry { keys: "a–z, …", desc: "filter the justfile list" },
            Entry { keys: "j / k, ↑ / ↓", desc: "move cursor" },
            Entry { keys: "Enter", desc: "select justfile" },
            Entry { keys: "Esc", desc: "cancel" },
        ],
    },
    Section {
        id: SectionId::Param,
        title: "Param input (modal)",
        entries: &[
            Entry { keys: "a–z, 0–9, …", desc: "edit the current parameter" },
            Entry { keys: "Backspace", desc: "remove last character" },
            Entry { keys: "Tab", desc: "move to next parameter" },
            Entry { keys: "Enter", desc: "commit all parameters and spawn the recipe" },
            Entry { keys: "Esc", desc: "cancel" },
        ],
    },
    Section {
        id: SectionId::Confirm,
        title: "Confirm prompt (K / x / q on running sessions)",
        entries: &[
            Entry { keys: "y / Enter", desc: "confirm" },
            Entry { keys: "n / c / Esc", desc: "cancel" },
        ],
    },
    Section {
        id: SectionId::Errors,
        title: "Errors list (after e)",
        entries: &[
            Entry { keys: "Esc / q / e", desc: "close" },
        ],
    },
    Section {
        id: SectionId::HelpItself,
        title: "Help (this modal)",
        entries: &[
            Entry { keys: "j / k, ↑ / ↓", desc: "scroll by one line" },
            Entry { keys: "PgUp / PgDn", desc: "scroll by ten lines" },
            Entry { keys: "Home / End", desc: "jump to top / bottom" },
            Entry { keys: "Esc / q / ? / F1", desc: "close" },
        ],
    },
];

fn build_lines(origin: SectionId) -> Vec<Line<'static>> {
    let mut out: Vec<Line<'static>> = Vec::new();
    for (idx, section) in SECTIONS.iter().enumerate() {
        if idx > 0 {
            out.push(Line::from(""));
        }
        let is_active = section.id == origin;
        let marker = if is_active { "▸ " } else { "  " };
        let title_style = if is_active {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().add_modifier(Modifier::BOLD)
        };
        out.push(Line::from(vec![
            Span::raw(marker),
            Span::styled(section.title, title_style),
        ]));
        for e in section.entries {
            out.push(Line::from(vec![
                Span::raw("    "),
                Span::styled(
                    format!("{:<20}", e.keys),
                    Style::default().fg(Color::Yellow),
                ),
                Span::raw("  "),
                Span::raw(e.desc),
            ]));
        }
    }
    out
}

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let (scroll, origin) = match &app.mode {
        Mode::Help { scroll, origin } => (*scroll, *origin),
        _ => return,
    };
    let lines = build_lines(origin);
    // Inner height = area.height - 2 (borders). Clamp scroll.
    let inner_rows = area.height.saturating_sub(2);
    let max_scroll = (lines.len() as u16).saturating_sub(inner_rows);
    let clamped = scroll.min(max_scroll);
    let para = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title("help"))
        .wrap(Wrap { trim: false })
        .scroll((clamped, 0));
    f.render_widget(para, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_section_id_has_a_section() {
        let all = [
            SectionId::ListFocus,
            SectionId::SessionFocus,
            SectionId::Filter,
            SectionId::Dropdown,
            SectionId::Param,
            SectionId::Confirm,
            SectionId::Errors,
            SectionId::HelpItself,
        ];
        for id in all {
            assert!(
                SECTIONS.iter().any(|s| s.id == id),
                "missing section for {id:?}"
            );
        }
    }

    #[test]
    fn build_lines_marks_only_the_origin_section() {
        let lines = build_lines(SectionId::Filter);
        // Exactly one line should start with "▸ "; every other section title
        // starts with "  ".
        let active_count = lines
            .iter()
            .filter(|l| {
                l.spans
                    .first()
                    .map(|s| s.content.as_ref() == "▸ ")
                    .unwrap_or(false)
            })
            .count();
        assert_eq!(active_count, 1);
    }
}
```

- [ ] **Step 2: Register the module in `src/ui/mod.rs`**

Currently:

```rust
pub mod focus;
pub mod layout;
pub mod list;
pub mod modal;
pub mod param_modal;
pub mod preview;
pub mod session_pane;
pub mod status_bar;
pub mod top_bar;
```

Insert `pub mod help;` alphabetically:

```rust
pub mod focus;
pub mod help;
pub mod layout;
pub mod list;
pub mod modal;
pub mod param_modal;
pub mod preview;
pub mod session_pane;
pub mod status_bar;
pub mod top_bar;
```

- [ ] **Step 3: Replace `modal.rs` help dispatch**

In `src/ui/modal.rs`:

a. Locate:

```rust
        Mode::Help { .. } => render_help(f),
```

Replace with:

```rust
        Mode::Help { .. } => {
            let h = f.size().height.saturating_sub(4).min(30);
            let area = centered(f.size(), 72, h);
            f.render_widget(Clear, area);
            super::help::render(f, app, area);
        }
```

The `app` argument is already available on `render` — the match happens inside `pub fn render(f: &mut Frame, app: &App)`.

b. Delete the `fn render_help(f: &mut Frame) { ... }` function entirely. It is now dead.

- [ ] **Step 4: Verify compile + tests + lint**

Run: `cargo build` → `Finished`.
Run: `cargo test` → every target `ok`; inline `src/ui/help.rs` unit tests (2) pass.
Run: `cargo clippy --all-targets -- -D warnings` → clean.
Run: `cargo fmt --check` → clean.

- [ ] **Step 5: Commit**

```bash
git add src/ui/help.rs src/ui/mod.rs src/ui/modal.rs
git commit -m "feat(ui): rich help modal with SECTIONS + active-section highlight"
```

---

## Task 6: Global `F1` intercept

**Files:**
- Modify: `src/app/event_loop.rs`

- [ ] **Step 1: Add the F1 intercept**

In `src/app/event_loop.rs`, find the block that starts with:

```rust
                    if let crossterm::event::Event::Key(key) = evt {
                        if app.focus == crate::app::types::Focus::Session
                            && app.mode == crate::app::types::Mode::Normal
                        {
```

Insert a new F1 guard immediately inside the `if let ... = evt` block but *before* the session-focus check:

```rust
                    if let crossterm::event::Event::Key(key) = evt {
                        if key.code == crossterm::event::KeyCode::F(1)
                            && !matches!(app.mode, crate::app::types::Mode::Help { .. })
                        {
                            crate::app::reducer::reduce(&mut app, Action::OpenHelp);
                            dirty = true;
                            continue;
                        }
                        if app.focus == crate::app::types::Focus::Session
                            && app.mode == crate::app::types::Mode::Normal
                        {
```

(Indent to match surrounding code.)

- [ ] **Step 2: Verify compile + tests + lint**

Run: `cargo build`
Run: `cargo test` → every target `ok`.
Run: `cargo clippy --all-targets -- -D warnings` → clean.
Run: `cargo fmt --check` → clean.

- [ ] **Step 3: Commit**

```bash
git add src/app/event_loop.rs
git commit -m "feat(event_loop): global F1 intercept opens help from any mode"
```

---

## Task 7: Help-modal snapshot test

**Files:**
- Modify: `tests/snapshots.rs`

- [ ] **Step 1: Write the failing snapshot test**

Append to `tests/snapshots.rs`:

```rust
#[test]
fn help_modal_list_focus_render_snapshot() {
    use lazyjust::app::help_section::SectionId;
    use lazyjust::app::types::Mode;

    let backend = TestBackend::new(80, 30);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = fixture_app();
    app.mode = Mode::Help {
        scroll: 0,
        origin: SectionId::ListFocus,
    };
    let screens = ui::SessionScreens::new();
    terminal.draw(|f| ui::render(f, &app, &screens)).unwrap();
    let buf = terminal.backend().buffer().clone();
    insta::assert_snapshot!(buffer_to_string(&buf));
}
```

- [ ] **Step 2: Run the test to let `insta` record the snapshot**

Run: `INSTA_UPDATE=always cargo test --test snapshots help_modal_list_focus_render_snapshot`
Expected: test passes; a new `.snap` file is created at `tests/snapshots/snapshots__help_modal_list_focus_render_snapshot.snap`.

- [ ] **Step 3: Eyeball the snapshot**

Open `tests/snapshots/snapshots__help_modal_list_focus_render_snapshot.snap`. Confirm:
- Modal borders visible.
- "List focus" title highlighted (cyan fg + bold; prefixed by `▸ `).
- Other section titles plain bold, prefixed by `  `.
- Content readable, entries aligned.

If layout looks wrong, fix `src/ui/help.rs::build_lines` or `render` and re-record the snapshot with `INSTA_UPDATE=always`. Do not commit the snapshot until it is correct.

- [ ] **Step 4: Verify all three snapshot tests pass without regen**

Run: `cargo test --test snapshots`
Expected: `test result: ok. 3 passed`. (Existing `initial_render_snapshot` and `session_focus_render_snapshot` unchanged; new test green.)

- [ ] **Step 5: Full suite + lint**

Run: `cargo test` → every target `ok`.
Run: `cargo clippy --all-targets -- -D warnings` → clean.
Run: `cargo fmt --check` → clean.

- [ ] **Step 6: Commit**

```bash
git add tests/snapshots.rs tests/snapshots/snapshots__help_modal_list_focus_render_snapshot.snap
git commit -m "test(ui): snapshot help modal with list-focus active"
```

---

## Task 8: Manual QA

**Files:** none (verification only)

- [ ] **Step 1: Build release**

Run: `cargo build --release`
Expected: `Finished`.

- [ ] **Step 2: Run the binary and walk the QA matrix**

Launch: `./target/release/lazyjust .`

Verify each row:

| Action | Expected |
|---|---|
| Press `F1` from list focus | Help opens. "List focus" section highlighted cyan + `▸`. |
| Close help with `Esc` | Returns to list. |
| Press `?` from list focus | Same behavior as `F1`. |
| `Tab` to session, press `F1` | Help opens. "Session focus" highlighted. |
| Close, type `?` in session focus | `?` appears in the shell (no help modal). |
| `/`, type a few chars, `F1` | Help opens. "Filter mode (after /)" highlighted. Close — filter state is discarded (documented limitation). |
| `d`, `F1` | Help opens. "Justfile dropdown" highlighted. |
| Inside help: `j` `k` | Content scrolls 1 line. |
| Inside help: `PgUp` `PgDn` | Scrolls 10 lines. |
| Inside help: `End` | Jumps to bottom. |
| Inside help: `Home` | Back to top. |

- [ ] **Step 3: No commit**

Verification only. If any row fails, file a follow-up fix commit in the affected task's scope.

---

## Self-review notes

- **Spec coverage:** §Opening → Task 2 (reducer) + Task 6 (F1). §Content → Task 5 (SECTIONS). §Scrolling → Tasks 3 (actions/reducer) + 4 (keymap) + 5 (clamp in render). §Closing → Task 4 (keymap). §Sizing → Task 5 (modal dispatch computes width=72, height=min(30, h-4)). §Implementation 1 (`help_section.rs`) → Task 1. §Implementation 2 (`Mode::Help` shape) → Task 2. §Implementation 3 (`Action` variants) → Task 3. §Implementation 4 (reducer) → Tasks 2 + 3. §Implementation 5 (keymap) → Task 4. §Implementation 6 (event_loop) → Task 6. §Implementation 7 (`ui/help.rs`) → Task 5. §Implementation 8 (modal dispatch) → Task 5. §Testing (unit) → Task 5 inline tests. §Testing (reducer) → Task 3. §Testing (snapshot) → Task 7. §Testing (manual QA) → Task 8.
- **Placeholder scan:** none.
- **Type consistency:** `Mode::Help { scroll: u16, origin: SectionId }` used consistently across Tasks 2, 3, 5, 7. `HelpScrollDown(u16)` / `HelpScrollUp(u16)` / `HelpScrollHome` / `HelpScrollEnd` consistent across Tasks 3, 4. `SectionId` variants used match the `active_section` return type.
