# Focus UX Improvements — Design Specification

**Date:** 2026-04-23
**Status:** Draft, pending implementation plan
**Owner:** Nick Hartjes
**Depends on:** v0.1.0 (commit `9ee28ff`, tag `v0.1.0`)

## Overview

v0.1.0 shipped with working focus state (`Focus::List` / `Focus::Session`) but no visible indication of which pane is active. Running a recipe also unconditionally jumps focus into the session pane, which trades agency for immediacy. This follow-up makes the focus indicator obvious and keeps the user in the recipe list after spawning a recipe.

## Goals

1. Make the current focus pane unambiguous via styling alone (no new UI chrome).
2. Spawn recipes without stealing focus. The user decides when to move into the session pane.
3. Offer a Mac-friendly alternate exit key alongside `F12`.

## Non-goals

- Top-bar "tab" labels or numbered pane indicators.
- Mouse support for focus.
- Configurable focus colors.
- "Preview while running" (show a different recipe's preview while a session runs in the background).

---

## Behavioral contract

### Focus indicator

- The active pane renders with a **cyan** border and an **inverted + bold** title (black foreground on cyan background, wrapped in a single space on each side).
- The inactive pane renders with a **dark gray** border and a **plain gray** title.
- Applies to the left pane (`recipes`) and the right pane (`session` or `preview`), not to the top bar or status bar.
- Modal overlays do not change the focus styling of the panes behind them.

### Spawning a recipe

- `Enter` on a recipe with **no** running session: spawn, set `active_session = Some(new_id)`, **leave** `focus` untouched. In practice this means the user stays in `Focus::List`.
- `Enter` on a recipe with an **existing** running or shell-after-exit session: focus jumps to that session (`focus = Focus::Session`) and `active_session` is pointed at it. Existing behavior preserved.
- `Shift+Enter` / `r`: always spawn a new session, always stay in the previous focus (list by default).
- A session that exits while focus is on the list updates its inline indicator (`●` → `✓` / `✗`); the right pane continues to display that session's final vt100 screen.

### Exiting the session pane

- `F12` unchanged.
- New: `Ctrl+g` also returns focus to the list when the session pane is focused.
- Both keys are intercepted before any PTY-forwarding; neither reaches the child process when session focus is active.

### Session pane with focus on the list

- The right pane continues to render the `active_session`'s vt100 screen if one exists, otherwise the highlighted-recipe preview.
- Keys are not forwarded to the PTY while focus is on the list — `j` / `k` / `/` / etc. drive the list as normal.
- Scrollback controls (`PgUp` / `PgDn` / `Home` / `End`) only apply when the session pane itself is focused.

---

## Implementation

### New file: `src/ui/focus.rs`

Small, single-responsibility helper module.

```rust
use crate::app::types::Focus;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders};

pub fn pane_block<'a>(title: &'a str, active: bool) -> Block<'a> {
    let border_color = if active { Color::Cyan } else { Color::DarkGray };
    let title_style = if active {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    };
    Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .title(ratatui::text::Span::styled(
            format!(" {title} "),
            title_style,
        ))
}

pub fn is_list_active(focus: Focus) -> bool {
    matches!(focus, Focus::List)
}

pub fn is_right_active(focus: Focus) -> bool {
    matches!(focus, Focus::Session)
}
```

### `src/ui/mod.rs`

- `pub mod focus;` added.
- Right-pane dispatch unchanged except that `session_pane::render` now receives an `active: bool` computed from `focus::is_right_active(app.focus)`.

### `src/ui/list.rs`

- Replace every `Block::default().borders(Borders::ALL).title("recipes")` (two sites: the empty-state fallback and the stateful list block) with `pane_block("recipes", is_list_active(app.focus))`.

### `src/ui/session_pane.rs`

- Signature becomes `pub fn render(f: &mut Frame, area: Rect, screen: &vt100::Parser, active: bool)`.
- Replace hardcoded block with `pane_block("session", active)`.
- Caller in `ui/mod.rs` passes `is_right_active(app.focus)`.

### `src/ui/preview.rs`

- Replace hardcoded block with `pane_block("preview", is_right_active(app.focus))`.

### `src/app/event_loop.rs`

- In `do_spawn`, remove the line `app.focus = crate::app::types::Focus::Session;`. Session spawns but focus is left alone.
- In the Session + Normal key-intercept branch, extend the F12 guard:

```rust
let is_exit_key = matches!(key.code, crossterm::event::KeyCode::F(12))
    || (key.code == crossterm::event::KeyCode::Char('g')
        && key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL));
if is_exit_key {
    crate::app::reducer::reduce(&mut app, Action::FocusList);
    dirty = true;
    continue;
}
```

- `spawn_highlighted`'s focus-existing-running branch is unchanged; it still sets `app.focus = Focus::Session` when it reuses a live session.

### `src/ui/modal.rs`

- Update the `render_help` body. Replace the Session-focus section with:

```
Session focus:
    PgUp/PgDn    scroll output
    Home/End     top / bottom of scrollback
    F12 / Ctrl+g  leave session pane
    (all other keys forwarded to the PTY)
```

### `README.md`

- Update the keybindings table row `F12` → `F12` / `Ctrl+g`.

---

## Testing

### Snapshot tests

Extend `tests/snapshots.rs` with a second case that sets `app.focus = Focus::Session` and snapshots the render. The golden output must differ from the list-focused snapshot (border + title styling swap). Regenerate with `INSTA_UPDATE=always cargo test --test snapshots` once.

### Reducer / behavioral tests

No reducer changes, so no new reducer tests are required. Existing tests continue to pass.

A test that asserts `do_spawn` does not mutate `focus` is desirable but requires a real PTY (it runs `just`); leave as a manual QA step unless the session integration test harness can be extended.

### Manual QA matrix

Run `lazyjust tests/fixtures/grouped`:

| Step | Expected |
|---|---|
| Initial load | List pane has cyan border + inverted title; preview pane dim. |
| `Tab` | Borders swap. Session pane bright; list dim. |
| `F12` | Back to list-active styling. |
| `Ctrl+g` (from session focus) | Same as `F12`. |
| `Enter` on `build` (idle) | `build` gets a `●` indicator; right pane shows output; list stays cyan-bordered. |
| `j` / `k` while the session runs | Cursor moves in the list; right pane keeps showing the running session. |
| `Enter` on `build` again while running | Focus jumps to session (cyan swaps). |
| Help modal (`?`) | Session-focus block shows `F12 / Ctrl+g`. |
| Resize terminal | Layout recomputes; focus indicator preserved. |

---

## Open questions

1. Preview-while-running — no key exists to hide a live session so the user can view a different recipe's preview. Deferred. Likely candidate: `Space` toggles a flag that prefers preview over session in the right pane.
2. Exit-flash / status-change cue when a session finishes while the list is focused. Not added in v1; inline `●`→`✓`/`✗` is assumed sufficient.
3. `Ctrl+g` conflicts with readline's abort inside an interactive shell. Accepted tradeoff; `F12` remains the conflict-free alternative. Documented in help modal and README.

## Rollback

All changes are additive to `src/ui/` and a two-line edit in `src/app/event_loop.rs`. Revert is a single-commit revert.
