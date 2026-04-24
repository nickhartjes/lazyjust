# Rich Help Modal — Design Specification

**Date:** 2026-04-24
**Status:** Draft, pending implementation plan
**Owner:** Nick Hartjes

## Overview

The help modal today is a hardcoded 22-line block that documents only the list-focus and session-focus keys. Five of the seven input modes (`FilterInput`, `Dropdown`, `ParamInput`, `Confirm`, `ErrorsList`) are undocumented — a user who hits them with no prior experience has no in-app recourse. Help is also unreachable from session focus, because `?` forwards to the PTY as a typed character.

This spec upgrades the modal to a full keybinding reference with descriptions, adds a globally interceptable `F1` shortcut so help is always accessible, and highlights the section that matches the user's current state so they can orient at a glance. Content becomes structured data (not a hardcoded string block) so it is easier to keep accurate as keybindings evolve.

## Goals

1. Every binding the user can press — across all seven modes plus session focus — appears in help with a short, accurate description.
2. `F1` opens help from any mode (including session focus), bypassing PTY forwarding. `?` continues to open it from list focus only.
3. The section matching the user's pre-help state is visually marked, so a lost user sees "you're here" at a glance.
4. Help content is scrollable when it exceeds the modal height.
5. Help content is structured data (Rust), not free-form text, so adding a new key binding requires touching exactly one place.

## Non-goals

- Fuzzy-search inside help (`/` inside the modal). Tracked as follow-up.
- Per-entry examples or multi-paragraph explanations. One-line descriptions only.
- Contextual hiding of inactive sections. All sections are always rendered; only the active one is highlighted.
- Mouse support for scrolling.
- Help inside ongoing modals (Confirm/Param/Dropdown) — help always returns to `Normal` on close.

---

## Behavioral contract

### Opening

- `F1` from any mode except `Mode::Help` opens help. The current `(Mode, Focus)` is snapshotted into `Mode::Help::origin`.
- `?` from `Mode::Normal + Focus::List` opens help (existing behavior, preserved).
- `?` from `Mode::Normal + Focus::Session` does NOT open help — it forwards to the PTY as a typed character (existing behavior, preserved so shell globs work).
- Opening help always resets `scroll` to `0`.

### Content

- One section per input mode plus one for session-focus key forwarding. Sections render in declaration order:
  1. List focus (Normal mode)
  2. Session focus
  3. Filter mode
  4. Justfile dropdown
  5. Param input
  6. Confirm prompt
  7. Errors list
  8. Help (this modal)

- Each section header appears in bold. The section whose `SectionId` matches `Mode::Help::origin` renders with a cyan title and a `▸` marker in the left margin — all other section titles render plain.

### Scrolling

- `j` / `↓` scroll 1 line down. `k` / `↑` scroll 1 line up.
- `PgDn` / `PgUp` scroll by 10 lines.
- `Home` / `End` jump to top / bottom.
- `scroll` is clamped at the render site using the actual laid-out content height; the reducer does not need to know the height.

### Closing

- `Esc`, `q`, `?`, `F1` close help. Mode returns to `Mode::Normal`. `origin` is discarded. Scroll is discarded.
- Focus is unchanged (whichever pane was focused before help stays focused after).

### Sizing

- Modal width: 72 columns.
- Modal height: `min(30, terminal_height - 4)`. Below a threshold (~16 rows), help renders whatever fits and the user scrolls.

---

## Implementation

### 1. New file: `src/ui/help.rs`

Holds the help content as `const` static data, the `active_section` resolver, and the render function.

```rust
use crate::app::types::{Focus, Mode};
use crate::app::App;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

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
            Entry { keys: "j / k, ↓ / ↑",       desc: "move cursor within recipes list" },
            Entry { keys: "h / l, ← / →",       desc: "cycle through the recipe's previous run sessions" },
            Entry { keys: "Enter",              desc: "run recipe — if a session is already running for it, jump there" },
            Entry { keys: "Shift+Enter / r",    desc: "always spawn a new session (never reuse)" },
            Entry { keys: "/",                  desc: "enter fuzzy-filter mode" },
            Entry { keys: "d",                  desc: "open justfile-switcher dropdown" },
            Entry { keys: "Tab",                desc: "cycle focus between list and session pane" },
            Entry { keys: "K",                  desc: "kill the focused session (confirms)" },
            Entry { keys: "x",                  desc: "close the focused session (confirms)" },
            Entry { keys: "Ctrl+o / Ctrl+i",    desc: "jump to next / previous session with unread output" },
            Entry { keys: "L",                  desc: "copy the focused session's log path" },
            Entry { keys: "> / < / =",          desc: "grow / shrink / reset the left pane width" },
            Entry { keys: "F1 / ?",             desc: "open this help" },
            Entry { keys: "e",                  desc: "open the startup-errors modal" },
            Entry { keys: "q",                  desc: "quit — confirms if sessions are running" },
        ],
    },
    Section {
        id: SectionId::SessionFocus,
        title: "Session focus",
        entries: &[
            Entry { keys: "F12 / Ctrl+g",       desc: "return focus to the recipes list" },
            Entry { keys: "PgUp / PgDn",        desc: "scroll session output up / down" },
            Entry { keys: "Home / End",         desc: "jump to top / bottom of scrollback" },
            Entry { keys: "(all other keys)",   desc: "forwarded to the running shell as typed input" },
            Entry { keys: "F1",                 desc: "open help (globally intercepted; does not reach the shell)" },
        ],
    },
    Section {
        id: SectionId::Filter,
        title: "Filter mode (after /)",
        entries: &[
            Entry { keys: "a–z, 0–9, …",        desc: "extend the filter pattern" },
            Entry { keys: "Backspace",          desc: "remove last character" },
            Entry { keys: "Enter",              desc: "commit filter and return to list" },
            Entry { keys: "Esc",                desc: "discard filter and return to list" },
        ],
    },
    Section {
        id: SectionId::Dropdown,
        title: "Justfile dropdown (after d)",
        entries: &[
            Entry { keys: "a–z, …",             desc: "filter the justfile list" },
            Entry { keys: "j / k, ↑ / ↓",       desc: "move cursor" },
            Entry { keys: "Enter",              desc: "select justfile" },
            Entry { keys: "Esc",                desc: "cancel" },
        ],
    },
    Section {
        id: SectionId::Param,
        title: "Param input (modal)",
        entries: &[
            Entry { keys: "a–z, 0–9, …",        desc: "edit the current parameter" },
            Entry { keys: "Backspace",          desc: "remove last character" },
            Entry { keys: "Tab",                desc: "move to next parameter" },
            Entry { keys: "Enter",              desc: "commit all parameters and spawn the recipe" },
            Entry { keys: "Esc",                desc: "cancel" },
        ],
    },
    Section {
        id: SectionId::Confirm,
        title: "Confirm prompt (K / x / q on running sessions)",
        entries: &[
            Entry { keys: "y / Enter",          desc: "confirm" },
            Entry { keys: "n / c / Esc",        desc: "cancel" },
        ],
    },
    Section {
        id: SectionId::Errors,
        title: "Errors list (after e)",
        entries: &[
            Entry { keys: "Esc / q / e",        desc: "close" },
        ],
    },
    Section {
        id: SectionId::HelpItself,
        title: "Help (this modal)",
        entries: &[
            Entry { keys: "j / k, ↑ / ↓",       desc: "scroll by one line" },
            Entry { keys: "PgUp / PgDn",        desc: "scroll by ten lines" },
            Entry { keys: "Home / End",         desc: "jump to top / bottom" },
            Entry { keys: "Esc / q / ? / F1",   desc: "close" },
        ],
    },
];

pub fn active_section(app: &App) -> SectionId {
    match &app.mode {
        Mode::Help { origin, .. } => *origin,
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

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    // Builds Vec<Line> from SECTIONS, applies active-section highlight,
    // clamps scroll to max, renders into a Paragraph inside a bordered Block.
    // Details deferred to implementation — no surprises.
}
```

### 2. `src/ui/mod.rs`

Add `pub mod help;`, alphabetical.

### 3. `src/app/types.rs`

Replace `Mode::Help` with:

```rust
pub enum Mode {
    Normal,
    FilterInput,
    ParamInput { recipe_idx: usize, values: Vec<String>, cursor: usize },
    Dropdown { filter: String, cursor: usize },
    Help { scroll: u16, origin: crate::ui::help::SectionId },
    Confirm { prompt: String, on_accept: ConfirmAction },
    ErrorsList,
}
```

The `crate::ui::help::SectionId` reference creates a new `ui → app::types` dependency. If that's undesirable, move `SectionId` into `app::types` and have `ui/help.rs` re-export it. Plan will decide based on existing module-graph cleanliness.

### 4. `src/app/action.rs`

Four new variants:

```rust
pub enum Action {
    // ... existing ...
    HelpScrollUp(u16),
    HelpScrollDown(u16),
    HelpScrollHome,
    HelpScrollEnd,
}
```

### 5. `src/app/reducer.rs`

Handle each:

- `OpenHelp`: snapshot `help::active_section(&app)` into `Mode::Help { scroll: 0, origin }`. Replaces the current `Mode::Help` transition.
- `HelpScrollUp(n)`: `scroll = scroll.saturating_sub(n)`.
- `HelpScrollDown(n)`: `scroll = scroll.saturating_add(n)`. Clamp happens at render.
- `HelpScrollHome`: `scroll = 0`.
- `HelpScrollEnd`: `scroll = u16::MAX`. Render clamps.
- `CloseHelp`: `Mode::Normal`.

### 6. `src/input/keymap.rs`

`help_mode` grows the scroll bindings:

```rust
fn help_mode(k: &KeyEvent) -> Option<Action> {
    match k.code {
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') | KeyCode::F(1) => Some(Action::CloseHelp),
        KeyCode::Char('j') | KeyCode::Down   => Some(Action::HelpScrollDown(1)),
        KeyCode::Char('k') | KeyCode::Up     => Some(Action::HelpScrollUp(1)),
        KeyCode::PageDown                    => Some(Action::HelpScrollDown(10)),
        KeyCode::PageUp                      => Some(Action::HelpScrollUp(10)),
        KeyCode::Home                        => Some(Action::HelpScrollHome),
        KeyCode::End                         => Some(Action::HelpScrollEnd),
        _ => None,
    }
}
```

### 7. `src/app/event_loop.rs`

Add a global F1 intercept at the top of the `Event::Key(key)` handler, before the session-focus forward-to-PTY branch:

```rust
if let crossterm::event::Event::Key(key) = evt {
    if key.code == crossterm::event::KeyCode::F(1)
        && !matches!(app.mode, crate::app::types::Mode::Help { .. })
    {
        crate::app::reducer::reduce(&mut app, Action::OpenHelp);
        dirty = true;
        continue;
    }
    // ... existing session-focus branch ...
}
```

Keymap's Normal-mode handler already maps `?` to `OpenHelp`; no change there.

### 8. `src/ui/modal.rs`

`render_help` reduces to a one-line dispatch:

```rust
Mode::Help { .. } => {
    let area = centered(f.size(), 72, f.size().height.saturating_sub(4).min(30));
    f.render_widget(Clear, area);
    super::help::render(f, app, area);
}
```

The existing `render_help` fn is deleted. The modal dispatcher's `Mode::Help` arm calls `help::render` directly.

---

## Testing

### Unit tests — `src/ui/help.rs`

Inline `#[cfg(test)] mod tests`:

- `active_section_from_normal_list_focus`: build `App` with `Mode::Normal + Focus::List` → `ListFocus`.
- `active_section_from_normal_session_focus`: `Mode::Normal + Focus::Session` → `SessionFocus`.
- `active_section_from_each_modal_mode`: one case each for `FilterInput`, `Dropdown`, `ParamInput`, `Confirm`, `ErrorsList`.
- `active_section_from_help_uses_origin`: `Mode::Help { origin: SectionId::Dropdown, scroll: 0 }` → `Dropdown`.
- `sections_cover_every_variant`: for each `SectionId` variant, assert it appears in `SECTIONS`.

### Reducer tests — `tests/reducer_tests.rs`

- `help_open_from_list_records_origin_list`
- `help_open_from_filter_records_origin_filter`
- `help_scroll_down_monotonic`: dispatch 3 × `HelpScrollDown(1)` → `scroll` = 3.
- `help_scroll_up_floors_zero`: with `scroll = 2`, `HelpScrollUp(5)` → 0.
- `help_scroll_end_saturates_max`
- `help_close_returns_to_normal`

### Snapshot tests — `tests/snapshots.rs`

- Existing `initial_render_snapshot` and `session_focus_render_snapshot` remain.
- Add `help_modal_list_focus_render_snapshot`: 80×30 terminal, `Mode::Help { scroll: 0, origin: ListFocus }`. Snapshot catches layout, highlight, full content.

### Manual QA

1. `F1` from list focus → modal opens, "List focus" highlighted.
2. `Tab` → session focus. `F1` → modal opens, "Session focus" highlighted.
3. `/` to enter filter, type, `F1` → modal opens, "Filter mode" highlighted. Close → filter state preserved? Filter state already gets discarded by the current `OpenHelp` transition; this spec does not change that behavior. Verify.
4. `d` → dropdown. `F1` opens, "Justfile dropdown" highlighted.
5. `?` in session focus → typed into shell, help does NOT open.
6. `F1` in session focus → help opens.
7. Inside help: `j`, `k`, `PgUp`, `PgDn`, `Home`, `End` all scroll. `Esc` closes.

---

## Risks

1. **Filter / Param / Confirm / Dropdown state loss on F1.** The existing `OpenHelp` / `CloseHelp` flow transitions to/from `Mode::Normal`, which drops any in-progress modal state. `F1` from inside Filter therefore silently discards the filter buffer. This spec preserves that behavior because fixing it is out of scope; document in the manual-QA row. A future follow-up could make help a pure overlay that leaves the underlying mode intact.
2. **`SectionId` in `Mode::Help` couples `app::types` to `ui::help`.** Discussed in §3. Plan picks the module-graph side that fits.
3. **`u16::MAX` as "scroll to bottom" sentinel.** Clean enough given typed-scroll is clamped at render time. Avoids plumbing content height into the reducer.
4. **Content drift.** Adding a new keybinding without updating `SECTIONS` leaves help inaccurate. A linter-style test that greps `keymap.rs` for every `Action::*` and asserts each is documented in at least one `SECTIONS` entry would catch this — treated as a follow-up.

## Rollback

All work lives in additions: new file `src/ui/help.rs`, new variants on `Mode::Help` and `Action`, new reducer arms, four keymap lines, an eight-line event_loop block. Single-commit revert restores the static 22-line help and `Mode::Help` without fields.
