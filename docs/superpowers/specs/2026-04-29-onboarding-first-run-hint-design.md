# Onboarding: first-run discoverability of `?`

**Date:** 2026-04-29
**Status:** Approved (design)
**Topic:** Make the help modal discoverable on first launch without adding state.

## Problem

A new user launches `lazyjust`, lands on the recipe list, and does not know
what keys are available. The status bar already renders a Normal-mode hint
(`⏎ run · / filter · t theme · ? help · q quit`), but the entire line uses
`theme.fg` and `theme.dim` and reads as one continuous block — the `?` does
not stand out, and the user does not realize a help modal exists.

The README's keybinding table is far below the install section, so a reader
who scans top-down sees the screenshot, then config, then themes before
keys.

## Goal

A new user spots `?` for help within ~2 seconds of first launch, and the
README signposts the same path before the full keybindings table.

## Non-goals

- First-launch detection (no config file => first run) and persisted
  "seen tour" flag.
- Help modal restyle.
- Animated GIF / screenshot of the help modal (tracked separately under
  the docs/screenshots backlog).
- Any change to the help modal contents.

## Design

### Change 1 — highlight `?` in status bar

File: `src/ui/status_bar.rs`

The `hint_for` function builds the Normal-mode hint and the
`Focus::Session` hint as a sequence of `Span`s using two local helpers:

```rust
let k = |s: &str| Span::styled(s.to_string(), Style::default().fg(theme.fg));
let d = |s: &str| Span::styled(s.to_string(), Style::default().fg(theme.dim));
```

Add a third helper for the accent color and use it for the `?` glyph in
both the `Mode::Normal` and `Mode::Normal if Focus::Session` arms:

```rust
let a = |s: &str| Span::styled(s.to_string(), Style::default().fg(theme.accent));
// ...
a("?"),
Span::raw(" "),
d("help"),
```

Only the `?` glyph itself is accented. The trailing word `help` stays dim so
the line keeps its rhythm.

### Change 2 — README "First run" section

File: `README.md`

Insert a new `## First run` section between the existing `## Usage` and
`## Configuration` sections:

```markdown
## First run

Launch `lazyjust` in any directory containing a `justfile`. The status bar at
the bottom lists the keys you need; the rest are in the help modal.

| Key | What it does |
|---|---|
| `?` / `F1` | Help modal — full keymap |
| `/` | Fuzzy-filter recipes |
| `Enter` | Run focused recipe |
| `t` | Theme picker (live preview) |
| `d` | Switch between discovered justfiles |
| `q` | Quit |

If no recipes show up, run `lazyjust --log-level debug` to see discovery
errors, or press `e` to open the startup-errors modal.
```

The full keybindings table further down stays as the reference.

## Testing

Manual: launch lazyjust against a sample repo, confirm `?` reads as accent
color in each bundled theme. No automated test added — `status_bar.rs` has
no existing snapshot or render tests, and adding one for a one-color
change is not worth the harness.

## Risks

- Accent color is theme-defined and a few themes may use a hue that does
  not contrast well against `bg`. All bundled themes already use `accent`
  for highlighted UI (e.g. selected recipe), so behavior is consistent
  with the rest of the app — if a user theme has poor contrast, the same
  problem already shows elsewhere.

## Out of scope follow-ups

- First-launch detection + transient banner.
- Help modal screenshot / GIF in README.
- Splitting README into `docs/usage.md` / `docs/themes.md` / `docs/config.md`.
