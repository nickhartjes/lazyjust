# UI Redesign — Milestone 3: Full UI Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rewrite every chrome renderer in `src/ui/*` to match the rich-dashboard layout from the UI redesign spec, using only theme slots for color. Add `[ui].icon_style` config plumbing and PID capture for the session header. Expand snapshot coverage to two themes × three sizes × three icon styles and wire the color-leak gate into CI.

**Architecture:** Rendering-only refactor on top of the M1 config + M2 theme system. `top_bar` / `list` / `preview` / `session_pane` / `status_bar` / `modal` / `help` / `param_modal` / `theme_picker` each get rewritten against the spec's layout. A shared `src/ui/modal_base.rs` helper centralizes rounded-border + title rendering. Layout min-sizes tighten from 20/40 to 28/48 cols and the "terminal too small" screen becomes themed. `[ui].icon_style` joins `UiSection` + `Config` and feeds the list renderer. `SessionMeta.pid: Option<u32>` is captured at spawn so the session header can print `pid 48231`. Zero behavior changes beyond these two state extensions — the reducer, event loop, and session manager are unchanged structurally.

**Tech Stack:** Rust · ratatui · insta (snapshots) · portable_pty · toml / serde · rg (CI gate) · just

**Spec:** `docs/superpowers/specs/2026-04-24-ui-redesign-design.md` (Milestone 3 section).

**Out of scope (explicit):**

- `[keys]` config-driven remapping — `t`, `q`, `?`, etc. stay hardcoded.
- `[ui].split_ratio` from config — M3 keeps `default_split_ratio` in `defaults()`; consuming the TOML key is future work.
- `show_inline_deps` toggle — spec §249 says "always rendered"; toggle not honored.
- Scroll thumb animation, pane-focus animations, running-indicator animation — v2 polish.
- External "logs ↗" opener — spec §300 says text-only in v1.
- Theme code changes — no new slots, no parse changes. If `mono-amber` doesn't yet exist as a built-in, M3 adds it as a minimal second theme only because snapshot fan-out needs it; no engine change.

---

## File Structure

**Modify:**

- `src/ui/layout.rs` — tighten min sizes (28/48), add the small-terminal split.
- `src/ui/mod.rs` — themed too-small screen, route through new modal base, wire icon_style from app.
- `src/ui/top_bar.rs` — full rewrite.
- `src/ui/list.rs` — full rewrite (section bars, inline deps, icon styles, row-level highlight).
- `src/ui/preview.rs` — full rewrite (structured blocks).
- `src/ui/session_pane.rs` — add header strip, status pill, scroll thumb, remove border. Keep vt100 color translation.
- `src/ui/status_bar.rs` — per-mode hint text, separators, right-aligned warn/error message.
- `src/ui/focus.rs` — delete `pane_block`; replace with `focus_bar` helper that returns a colored `▍` glyph.
- `src/ui/modal.rs` — delegate title/border to new `modal_base`, restyle all modals.
- `src/ui/help.rs` — restyle title + active section to use themed modal base.
- `src/ui/param_modal.rs` — restyle via modal base.
- `src/ui/theme_picker.rs` — restyle via modal base.
- `src/app/state.rs` — hold `icon_style: IconStyle` alongside theme.
- `src/app/types.rs` — add `SessionMeta.pid: Option<u32>`.
- `src/config.rs` — add `icon_style: IconStyle` field.
- `src/config/file.rs` — add `UiSection.icon_style: Option<String>`.
- `src/config/merge.rs` — parse and apply `icon_style` string → enum.
- `src/config/defaults.rs` — default `IconStyle::Round`.
- `src/session/manager.rs` — capture `child.process_id()` after spawn, thread into `SessionMeta`.
- `src/app/reducer.rs` — populate `pid` when constructing `SessionMeta`.
- `justfile` — keep `color-gate` recipe as-is; add to `ci` (already present).
- `.github/workflows/ci.yml` — add `just color-gate` step.
- `tests/snapshots.rs` — parameterize on theme × size × icon_style; add a snapshot per pane × state.

**Create:**

- `src/ui/icon_style.rs` — `IconStyle` enum + glyph mapping.
- `src/ui/modal_base.rs` — rounded border + title helper; centered-rect helper moved here.
- `src/ui/session_header.rs` — the 1-row header strip above the vt100 grid.
- `src/ui/scrollbar.rs` — the 1-col thumb helper for the session output pane.
- `assets/themes/mono-amber.toml` — second built-in theme (if not present) for snapshot fan-out.

---

## Task 1: `IconStyle` enum and config plumbing

**Files:**
- Create: `src/ui/icon_style.rs`
- Modify: `src/ui/mod.rs`, `src/config.rs`, `src/config/file.rs`, `src/config/merge.rs`, `src/config/defaults.rs`

- [ ] **Step 1: Write the failing test for enum parsing**

Add to `src/ui/icon_style.rs` (will exist after step 3):

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_known_names_case_insensitive() {
        assert_eq!(IconStyle::parse("round"), Some(IconStyle::Round));
        assert_eq!(IconStyle::parse("ASCII"), Some(IconStyle::Ascii));
        assert_eq!(IconStyle::parse("None"), Some(IconStyle::None));
    }

    #[test]
    fn unknown_returns_none() {
        assert_eq!(IconStyle::parse("fancy"), None);
    }

    #[test]
    fn glyphs_by_style() {
        let r = IconStyle::Round.glyphs();
        assert_eq!(r.unselected, "○");
        assert_eq!(r.running, "●");
        assert_eq!(r.cursor, "▶");

        let a = IconStyle::Ascii.glyphs();
        assert_eq!(a.unselected, "o");
        assert_eq!(a.running, "*");
        assert_eq!(a.cursor, ">");

        let n = IconStyle::None.glyphs();
        assert_eq!(n.unselected, "");
        assert_eq!(n.running, "");
        assert_eq!(n.cursor, "");
    }
}
```

- [ ] **Step 2: Run and verify it fails**

Run: `cargo test -p lazyjust --lib ui::icon_style`
Expected: FAIL — module not found.

- [ ] **Step 3: Implement `IconStyle`**

`src/ui/icon_style.rs`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconStyle {
    Round,
    Ascii,
    None,
}

pub struct Glyphs {
    pub unselected: &'static str,
    pub running: &'static str,
    pub cursor: &'static str,
}

impl IconStyle {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "round" => Some(Self::Round),
            "ascii" => Some(Self::Ascii),
            "none" => Some(Self::None),
            _ => None,
        }
    }

    pub fn glyphs(self) -> Glyphs {
        match self {
            Self::Round => Glyphs { unselected: "○", running: "●", cursor: "▶" },
            Self::Ascii => Glyphs { unselected: "o", running: "*", cursor: ">" },
            Self::None => Glyphs { unselected: "", running: "", cursor: "" },
        }
    }
}
```

Register in `src/ui/mod.rs` (add `pub mod icon_style;` alongside existing `pub mod` lines).

- [ ] **Step 4: Run test — verify pass**

Run: `cargo test -p lazyjust --lib ui::icon_style`
Expected: PASS.

- [ ] **Step 5: Extend `UiSection` with `icon_style`**

In `src/config/file.rs`, replace `UiSection`:

```rust
#[derive(Debug, Default, Deserialize)]
pub struct UiSection {
    pub theme: Option<String>,
    pub icon_style: Option<String>,
}
```

- [ ] **Step 6: Add `icon_style` to `Config`**

In `src/config.rs` append to struct:

```rust
pub icon_style: crate::ui::icon_style::IconStyle,
```

In `src/config/defaults.rs` add to the `Config { … }` literal:

```rust
icon_style: crate::ui::icon_style::IconStyle::Round,
```

- [ ] **Step 7: Write failing merge test**

Add to `src/config/merge.rs` tests:

```rust
#[test]
fn ui_icon_style_override_applies() {
    use crate::ui::icon_style::IconStyle;
    let file = ConfigFile {
        ui: Some(crate::config::file::UiSection {
            theme: None,
            icon_style: Some("ascii".into()),
        }),
        ..Default::default()
    };
    let merged = merge(file, defaults());
    assert_eq!(merged.icon_style, IconStyle::Ascii);
}

#[test]
fn ui_icon_style_unknown_falls_back_to_default() {
    let file = ConfigFile {
        ui: Some(crate::config::file::UiSection {
            theme: None,
            icon_style: Some("bogus".into()),
        }),
        ..Default::default()
    };
    let merged = merge(file, defaults());
    assert_eq!(merged.icon_style, crate::ui::icon_style::IconStyle::Round);
}
```

Run: `cargo test -p lazyjust --lib config::merge`
Expected: FAIL — `icon_style` not on merged result yet.

- [ ] **Step 8: Implement merge**

In `src/config/merge.rs`, inside the `if let Some(u) = file.ui { … }` block:

```rust
if let Some(icon) = u.icon_style.as_deref() {
    if let Some(parsed) = crate::ui::icon_style::IconStyle::parse(icon) {
        out.icon_style = parsed;
    } else {
        tracing::warn!(value = %icon, "unknown [ui].icon_style, using default");
    }
}
```

Add `icon_style: Default::default()` is not needed — default comes from `defaults()`. Derive `Default` isn't required.

- [ ] **Step 9: Run config tests**

Run: `cargo test -p lazyjust --lib config`
Expected: PASS.

- [ ] **Step 10: Commit**

```bash
git add src/ui/mod.rs src/ui/icon_style.rs src/config.rs src/config/file.rs src/config/merge.rs src/config/defaults.rs
git commit -m "feat(config): add [ui].icon_style with round|ascii|none"
```

---

## Task 2: Thread `icon_style` into `App`

**Files:**
- Modify: `src/app/state.rs`, every call site of `App::new`

- [ ] **Step 1: Find every call site**

Run: `rg -n 'App::new\(' src tests`
Expected: matches in `src/main.rs`, `tests/snapshots.rs`, possibly others.

- [ ] **Step 2: Add field to `App`**

In `src/app/state.rs`:

```rust
pub icon_style: crate::ui::icon_style::IconStyle,
```

Extend the `new` signature:

```rust
pub fn new(
    justfiles: Vec<Justfile>,
    startup_errors: Vec<(PathBuf, String)>,
    split_ratio: f32,
    theme: crate::theme::Theme,
    theme_name: String,
    icon_style: crate::ui::icon_style::IconStyle,
) -> Self {
    Self {
        // … existing fields …
        icon_style,
        // …
    }
}
```

- [ ] **Step 3: Update every call site**

In `src/main.rs` pass `cfg.icon_style` through to `App::new`.
In `tests/snapshots.rs` fixture pass `IconStyle::Round`.
Any integration tests: same.

- [ ] **Step 4: Run full build**

Run: `cargo check --all-targets`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/app/state.rs src/main.rs tests/snapshots.rs
git commit -m "feat(app): carry icon_style on App"
```

---

## Task 3: Capture PID on session spawn

**Files:**
- Modify: `src/app/types.rs`, `src/session/manager.rs`, `src/app/reducer.rs`

- [ ] **Step 1: Extend `SessionMeta`**

In `src/app/types.rs` add to `SessionMeta`:

```rust
pub pid: Option<u32>,
```

- [ ] **Step 2: Inspect spawn site**

Run: `rg -n 'spawn_recipe\|child.process_id' src`
Identify where `SpawnedPty { child, .. }` is destructured — likely `session/manager.rs` returning a handle, and `app/reducer.rs` constructing `SessionMeta`.

- [ ] **Step 3: Expose pid from spawn**

In `src/session/manager.rs`, after the `spawn(&argv, …)?` call, call `child.process_id()` before wrapping in the `SessionHandle` / thread move, and include it on the returned handle (add `pub pid: Option<u32>` to the struct that currently carries `child`). If that struct stores `child` after transfer into a thread, capture pid first:

```rust
let SpawnedPty { master, child, reader } = spawn(&argv, cwd, rows, cols)?;
let pid = child.process_id();
```

Return `pid` alongside whatever id/handle the manager already returns.

- [ ] **Step 4: Thread pid into `SessionMeta`**

In `src/app/reducer.rs`, at the site that constructs `SessionMeta { … }`, add `pid` from the spawn handle. If the reducer currently doesn't have pid in scope, widen the incoming spawn result type (or the `Action` variant that delivers spawn-success data) to carry `pid: Option<u32>`.

- [ ] **Step 5: Update any `SessionMeta { … }` literal in tests**

Run: `rg -n 'SessionMeta \{' src tests`
For each literal, add `pid: None,`.

- [ ] **Step 6: Run full build + tests**

```bash
cargo check --all-targets
cargo test --all-targets
```

Expected: PASS (existing snapshots may drift — if so, re-run with `cargo insta review` after later tasks rewrite the renderer).

- [ ] **Step 7: Commit**

```bash
git add src/app/types.rs src/session/manager.rs src/app/reducer.rs
git commit -m "feat(session): capture pid on spawn for header strip"
```

---

## Task 4: Extract recipe dependencies from `just --dump`

**Files:**
- Modify: `src/app/types.rs`, `src/discovery/parse.rs`
- Test: `tests/discovery_parse_tests.rs`

- [ ] **Step 1: Add `dependencies` to `Recipe` and helpers**

In `src/app/types.rs`:

```rust
pub struct Recipe {
    // … existing fields …
    pub dependencies: Vec<String>, // leaf dep names in declaration order
}

impl Recipe {
    pub fn has_deps(&self) -> bool { !self.dependencies.is_empty() }
    pub fn dep_names(&self) -> Vec<&str> { self.dependencies.iter().map(String::as_str).collect() }
}
```

- [ ] **Step 2: Write failing parse test**

Append to `tests/discovery_parse_tests.rs`:

```rust
#[test]
fn parse_extracts_dependencies_in_declaration_order() {
    let json = r#"{
        "recipes": {
            "ci": {
                "dependencies": [
                    {"recipe": "fmt"},
                    {"recipe": "lint"},
                    {"recipe": "test"}
                ],
                "body": []
            }
        }
    }"#;
    let recipes = lazyjust::discovery::parse::parse_dump(json).unwrap();
    let ci = recipes.iter().find(|r| r.name == "ci").unwrap();
    assert_eq!(ci.dependencies, vec!["fmt", "lint", "test"]);
}

#[test]
fn parse_handles_missing_dependencies_field() {
    let json = r#"{"recipes": {"solo": {"body": []}}}"#;
    let recipes = lazyjust::discovery::parse::parse_dump(json).unwrap();
    assert!(recipes[0].dependencies.is_empty());
}
```

Run: `cargo test -p lazyjust --test discovery_parse_tests`
Expected: FAIL — `dependencies` field missing on `Recipe`; parser does not extract it.

- [ ] **Step 3: Extend `RawRecipe` and the mapper in `src/discovery/parse.rs`**

```rust
#[derive(Deserialize)]
struct RawRecipe {
    // … existing fields …
    #[serde(default)]
    dependencies: Vec<RawDep>,
}

#[derive(Deserialize)]
struct RawDep {
    recipe: String,
}
```

In the `.map(|(name, r)| Recipe { … })` closure, add:

```rust
dependencies: r.dependencies.into_iter().map(|d| d.recipe).collect(),
```

- [ ] **Step 4: Run parse tests — verify pass**

Run: `cargo test -p lazyjust --test discovery_parse_tests`
Expected: PASS.

- [ ] **Step 5: Update every `Recipe { … }` literal elsewhere**

Run: `rg -n 'Recipe \{' src tests`
Add `dependencies: Vec::new(),` (or realistic values) to each literal — notably `tests/snapshots.rs::fixture_app` for tests that exercise inline-deps coverage.

- [ ] **Step 6: Build + tests**

```bash
cargo check --all-targets
cargo test --all-targets
```

Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add src/app/types.rs src/discovery/parse.rs tests/discovery_parse_tests.rs tests/snapshots.rs
git commit -m "feat(discovery): extract recipe dependencies for inline dep rendering"
```

---

## Task 5: Tighten layout min-sizes and route the too-small screen

**Files:**
- Modify: `src/ui/layout.rs`, `src/ui/mod.rs`

- [ ] **Step 1: Write failing test for min-size computation**

Add to `src/ui/layout.rs` (or new `tests` module if absent):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::layout::Rect;

    fn rect(w: u16, h: u16) -> Rect { Rect { x: 0, y: 0, width: w, height: h } }

    #[test]
    fn list_pane_respects_min_left_cols() {
        let app = fake_app_with_split(0.01);
        let panes = compute(rect(120, 30), &app);
        assert!(panes.list.width >= 28, "list {} < 28", panes.list.width);
    }

    #[test]
    fn right_pane_respects_min_right_cols() {
        let app = fake_app_with_split(0.99);
        let panes = compute(rect(120, 30), &app);
        assert!(panes.right.width >= 48, "right {} < 48", panes.right.width);
    }

    fn fake_app_with_split(ratio: f32) -> crate::app::App {
        crate::app::App::new(
            vec![],
            vec![],
            ratio,
            crate::theme::registry::resolve(crate::theme::DEFAULT_THEME_NAME),
            crate::theme::DEFAULT_THEME_NAME.into(),
            crate::ui::icon_style::IconStyle::Round,
        )
    }
}
```

Run: `cargo test -p lazyjust --lib ui::layout`
Expected: FAIL — current compute uses `Percentage` without floor.

- [ ] **Step 2: Implement floors**

Rewrite `compute` body's horizontal split to clamp:

```rust
let total = vertical[1].width;
let mut left = ((app.split_ratio * total as f32).round() as u16).max(28);
if total.saturating_sub(left) < 48 {
    left = total.saturating_sub(48).max(0);
}
let right = total.saturating_sub(left);
let horizontal = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([Constraint::Length(left), Constraint::Length(right)])
    .split(vertical[1]);
```

- [ ] **Step 3: Run layout tests — verify pass**

Run: `cargo test -p lazyjust --lib ui::layout`
Expected: PASS.

- [ ] **Step 4: Themed too-small screen in `mod.rs`**

Replace the `if size.width < 40 || size.height < 10 { … }` block in `src/ui/render`:

```rust
use ratatui::style::Style;
use ratatui::text::{Line, Span};

let cfg_cols: u16 = 40;
let cfg_rows: u16 = 10;
if size.width < cfg_cols || size.height < cfg_rows {
    let theme = &app.theme;
    let filled = ratatui::widgets::Paragraph::new("")
        .style(Style::default().bg(theme.bg));
    f.render_widget(filled, size);
    let msg_text = format!(
        "Terminal too small — need at least {}×{}.",
        cfg_cols, cfg_rows,
    );
    let msg = Line::from(Span::styled(msg_text, Style::default().fg(theme.dim).bg(theme.bg)));
    let y = size.height / 2;
    let area = Rect { x: size.x, y: size.y + y, width: size.width, height: 1 };
    let para = ratatui::widgets::Paragraph::new(msg)
        .alignment(ratatui::layout::Alignment::Center);
    f.render_widget(para, area);
    return;
}
```

- [ ] **Step 5: Run build**

Run: `cargo check --all-targets`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src/ui/layout.rs src/ui/mod.rs
git commit -m "feat(ui): floor list/right min cols and theme too-small screen"
```

---

## Task 6: New `focus_bar` helper and remove pane blocks

**Files:**
- Modify: `src/ui/focus.rs`

- [ ] **Step 1: Drop `pane_block`**

Delete the `pane_block` fn from `src/ui/focus.rs`. Keep `is_list_active` and `is_right_active`.

Add:

```rust
use ratatui::style::Style;
use ratatui::text::Span;

/// 1-col focus indicator rendered at the left edge of a pane.
/// `▍` in accent when active, dim when not.
pub fn focus_bar(active: bool, theme: &crate::theme::Theme) -> Span<'static> {
    let color = if active { theme.accent } else { theme.dim };
    Span::styled("▍", Style::default().fg(color))
}
```

- [ ] **Step 2: Build will fail where `pane_block` is still called**

Run: `cargo check --all-targets`
Expected: FAIL — every call site (list, preview, session_pane) must be rewritten in later tasks.

Leave this failing until Task 7–10 replace each call site. **Do not commit yet** — the chrome rewrite tasks re-establish the build and commit together at the end of Task 10.

---

## Task 7: Rewrite `top_bar`

**Files:**
- Modify: `src/ui/top_bar.rs`
- Test: `tests/snapshots.rs` (existing `initial_render_snapshot`)

- [ ] **Step 1: Write rewritten top_bar**

Full replacement of `src/ui/top_bar.rs`:

```rust
use crate::app::App;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

pub fn render(f: &mut Frame, area: Rect, app: &App, theme: &crate::theme::Theme) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(1), Constraint::Length(badge_width(app))])
        .split(area);

    // left side: bleed bar · lazyjust · justfile path · recipe count · error badge
    let jf = app.active_justfile();
    let path = jf
        .map(|j| j.path.display().to_string())
        .unwrap_or_else(|| "<no justfile>".into());
    let count = jf.map(|j| j.recipes.len()).unwrap_or(0);

    let mut spans: Vec<Span> = vec![
        Span::styled("▌", Style::default().fg(theme.accent)),
        Span::raw(" "),
        Span::styled("lazyjust", Style::default().fg(theme.fg).add_modifier(Modifier::BOLD)),
        Span::styled("  · ", Style::default().fg(theme.dim)),
        Span::styled(path, Style::default().fg(theme.dim)),
        Span::styled("  · ", Style::default().fg(theme.dim)),
        Span::styled(format!("{count} recipes"), Style::default().fg(theme.dim)),
    ];
    if !app.startup_errors.is_empty() {
        spans.push(Span::styled("   ", Style::default()));
        spans.push(Span::styled(
            format!(" {} load errors ", app.startup_errors.len()),
            Style::default().fg(theme.error).bg(theme.bg).add_modifier(Modifier::BOLD),
        ));
    }
    f.render_widget(Paragraph::new(Line::from(spans)), cols[0]);

    // right side: breadcrumb pill (justfile parent directory)
    if let Some(j) = jf {
        let parent = j
            .path
            .parent()
            .and_then(|p| p.file_name())
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let pill = Line::from(vec![
            Span::styled(
                format!(" {parent} "),
                Style::default().fg(theme.badge_fg).bg(theme.badge_bg),
            ),
        ]);
        let right = Paragraph::new(pill).alignment(ratatui::layout::Alignment::Right);
        f.render_widget(right, cols[1]);
    }
}

fn badge_width(app: &App) -> u16 {
    app.active_justfile()
        .and_then(|j| j.path.parent())
        .and_then(|p| p.file_name())
        .map(|s| s.to_string_lossy().chars().count() as u16 + 2)
        .unwrap_or(0)
}
```

- [ ] **Step 2: Run build**

Run: `cargo check --all-targets`
Expected: still fails in `list`/`preview`/`session_pane` but `top_bar` compiles.

- [ ] **Step 3: Defer commit until list/preview/session renderers also compile**

Continue to Task 8. Commit after Task 10 when build is green again.

---

## Task 8: Rewrite `list`

**Files:**
- Modify: `src/ui/list.rs`

- [ ] **Step 1: Replace the file**

Full replacement of `src/ui/list.rs`:

```rust
use crate::app::types::{Recipe, Status};
use crate::app::App;
use crate::ui::focus::{focus_bar, is_list_active};
use crate::ui::icon_style::IconStyle;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

pub fn render(f: &mut Frame, area: Rect, app: &App, theme: &crate::theme::Theme) {
    let active = is_list_active(app.focus);
    let Some(jf) = app.active_justfile() else {
        f.render_widget(Paragraph::new(Line::from(focus_bar(active, theme))), area);
        return;
    };

    let glyphs = app.icon_style.glyphs();
    let lines = build_lines(
        jf.recipes.as_slice(),
        &app.filter,
        app,
        theme,
        app.icon_style,
        &glyphs,
        active,
        area.width,
    );
    f.render_widget(Paragraph::new(lines), area);
}

#[allow(clippy::too_many_arguments)]
fn build_lines<'a>(
    recipes: &'a [Recipe],
    filter: &str,
    app: &App,
    theme: &crate::theme::Theme,
    style: IconStyle,
    g: &crate::ui::icon_style::Glyphs,
    active: bool,
    width: u16,
) -> Vec<Line<'a>> {
    let names: Vec<&str> = recipes.iter().map(|r| r.name.as_str()).collect();
    let scored = crate::app::filter::fuzzy_match(&names, filter);

    let mut out: Vec<Line> = Vec::new();
    let mut current_group: Option<&str> = None;
    let mut displayed_idx = 0usize;
    let selected = app.list_cursor.min(scored.len().saturating_sub(1));

    for (idx, _score) in &scored {
        let r = &recipes[*idx];
        let group_name = r.group.as_deref();
        if group_name != current_group {
            let label = group_name.unwrap_or("RECIPES").to_ascii_uppercase();
            out.push(section_header(&label, theme, width, active, displayed_idx == 0));
            current_group = group_name;
        }
        let is_cursor = displayed_idx == selected;
        out.push(row(r, app, theme, style, g, is_cursor, width));
        displayed_idx += 1;
    }
    out
}

fn section_header<'a>(label: &str, theme: &crate::theme::Theme, width: u16, active: bool, first: bool) -> Line<'a> {
    let bar = crate::ui::focus::focus_bar(active && first, theme);
    let title = format!(" {label} ");
    let used = 1 + title.chars().count() as u16;
    let rule_len = width.saturating_sub(used);
    let rule: String = "─".repeat(rule_len as usize);
    Line::from(vec![
        bar,
        Span::styled(title, Style::default().fg(theme.accent)),
        Span::styled(rule, Style::default().fg(theme.dim)),
    ])
}

fn row<'a>(
    r: &'a Recipe,
    app: &App,
    theme: &crate::theme::Theme,
    style: IconStyle,
    g: &crate::ui::icon_style::Glyphs,
    is_cursor: bool,
    width: u16,
) -> Line<'a> {
    let (marker, bullet) = if is_cursor { (g.cursor, "") } else { ("", g.unselected) };
    let leading = if style == IconStyle::None {
        if is_cursor { "▶  ".to_string() } else { "   ".to_string() }
    } else if is_cursor {
        format!("{marker}  ")
    } else {
        format!("   {bullet} ")
    };
    let name_style = if is_cursor {
        Style::default().fg(theme.selected_fg).bg(theme.highlight).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.fg)
    };
    let row_bg = if is_cursor { Some(theme.highlight) } else { None };

    let mut spans = vec![Span::styled(leading.clone(), name_style)];
    spans.push(Span::styled(r.name.clone(), name_style));
    spans.extend(session_indicators_for(r, app, theme, style, g));

    if r.has_deps() {
        let deps = dep_line(r, width.saturating_sub(visible_width(&spans) as u16));
        if !deps.is_empty() {
            spans.push(Span::styled(format!("   → {deps}"), Style::default().fg(theme.dim)));
        }
    }

    if let Some(bg) = row_bg {
        // pad to eol so the highlight bar runs to the edge
        let used = visible_width(&spans) as u16;
        if used < width {
            spans.push(Span::styled(" ".repeat((width - used) as usize), Style::default().bg(bg)));
        }
    }
    Line::from(spans)
}

fn dep_line(r: &Recipe, avail: u16) -> String {
    let joined = r.dep_names().join(" · ");
    if (joined.chars().count() as u16) <= avail {
        joined
    } else {
        let mut acc = String::new();
        for ch in joined.chars() {
            if acc.chars().count() as u16 + 1 >= avail.saturating_sub(1) { break; }
            acc.push(ch);
        }
        acc.push('…');
        acc
    }
}

fn visible_width(spans: &[Span]) -> usize {
    spans.iter().map(|s| s.content.chars().count()).sum()
}

fn session_indicators_for<'a>(
    r: &'a Recipe,
    app: &App,
    theme: &crate::theme::Theme,
    style: IconStyle,
    g: &crate::ui::icon_style::Glyphs,
) -> Vec<Span<'a>> {
    let mut out = Vec::new();
    let mut emitted = 0usize;
    for &sid in r.runs.iter().rev() {
        if emitted >= 3 {
            out.push(Span::styled(
                format!("  +{}", r.runs.len() - 3),
                Style::default().fg(theme.dim),
            ));
            break;
        }
        if let Some(s) = app.session(sid) {
            out.push(Span::raw("  "));
            out.push(status_span(s.status, s.unread, theme, style, g));
            emitted += 1;
        }
    }
    out
}

fn status_span(
    status: Status,
    unread: bool,
    theme: &crate::theme::Theme,
    style: IconStyle,
    g: &crate::ui::icon_style::Glyphs,
) -> Span<'static> {
    let (icon, color) = match status {
        Status::Running => (if style == IconStyle::None { "" } else { g.running }, theme.running),
        Status::ShellAfterExit { code } | Status::Exited { code } if code == 0 => {
            ("✓", if unread { theme.success } else { theme.dim })
        }
        Status::ShellAfterExit { .. } | Status::Exited { .. } => {
            ("✗", if unread { theme.error } else { theme.dim })
        }
        Status::Broken => ("!", theme.warn),
    };
    Span::styled(icon.to_string(), Style::default().fg(color))
}
```

- [ ] **Step 2: Run build**

`has_deps` / `dep_names` were added in Task 4, so the `r.has_deps()` and `r.dep_names()` calls above compile directly.

Run: `cargo check --all-targets`
Expected: `list` + `top_bar` compile; `preview` / `session_pane` still broken from Task 6's `pane_block` removal.

Continue to Task 9.

---

## Task 9: Rewrite `preview`

**Files:**
- Modify: `src/ui/preview.rs`

- [ ] **Step 1: Replace the file**

```rust
use crate::app::App;
use crate::ui::focus::{focus_bar, is_right_active};
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};
use ratatui::Frame;

pub fn render(f: &mut Frame, area: Rect, app: &App, theme: &crate::theme::Theme) {
    let active = is_right_active(app.focus);
    let Some(r) = app.recipe_at_cursor() else {
        let line = Line::from(focus_bar(active, theme));
        f.render_widget(Paragraph::new(line), area);
        return;
    };

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(vec![
        focus_bar(active, theme),
        Span::raw(" "),
        Span::styled(
            r.name.clone(),
            Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
        ),
    ]));
    if let Some(doc) = &r.doc {
        lines.push(Line::from(Span::styled(
            format!("  {doc}"),
            Style::default().fg(theme.dim),
        )));
    }

    if r.has_deps() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  depends on",
            Style::default().fg(theme.dim),
        )));
        for dep in r.dep_names() {
            lines.push(Line::from(vec![
                Span::raw("    "),
                Span::styled("▸ ", Style::default().fg(theme.success)),
                Span::styled(dep.to_string(), Style::default().fg(theme.fg)),
            ]));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  command",
        Style::default().fg(theme.dim),
    )));
    for (i, cmd_line) in r.command_preview.lines().enumerate() {
        let prefix = if i == 0 {
            vec![
                Span::raw("    "),
                Span::styled("$ ", Style::default().fg(theme.info)),
            ]
        } else {
            vec![Span::raw("      ")]
        };
        let mut spans = prefix;
        spans.push(Span::styled(cmd_line.to_string(), Style::default().fg(theme.fg)));
        lines.push(Line::from(spans));
    }

    if !r.params.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  params",
            Style::default().fg(theme.dim),
        )));
        for p in &r.params {
            let mut spans = vec![
                Span::raw("    "),
                Span::styled(p.name.clone(), Style::default().fg(theme.fg)),
            ];
            if let Some(d) = &p.default {
                spans.push(Span::styled(
                    format!("  (default: {d})"),
                    Style::default().fg(theme.dim),
                ));
            }
            lines.push(Line::from(spans));
        }
    }

    let p = Paragraph::new(lines).wrap(Wrap { trim: false });
    f.render_widget(p, area);
}
```

- [ ] **Step 2: Run build**

Run: `cargo check --all-targets`
Expected: fails only in `session_pane` now.

Continue to Task 10.

---

## Task 10: Session pane header + scroll thumb + remove border

**Files:**
- Create: `src/ui/session_header.rs`, `src/ui/scrollbar.rs`
- Modify: `src/ui/session_pane.rs`, `src/ui/mod.rs`

- [ ] **Step 1: Write `session_header.rs`**

```rust
use crate::app::types::{SessionMeta, Status};
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;
use std::time::Instant;

pub fn render(
    f: &mut Frame,
    area: Rect,
    meta: &SessionMeta,
    active: bool,
    theme: &crate::theme::Theme,
) {
    let bar = crate::ui::focus::focus_bar(active, theme);
    let elapsed = fmt_elapsed(meta.started_at.elapsed());
    let (glyph, glyph_color, label) = match meta.status {
        Status::Running => ("●", theme.running, format!("running · {elapsed}")),
        Status::Exited { code } if code == 0 => ("✓", theme.success, format!("done · {elapsed}")),
        Status::Exited { code } => ("✗", theme.error, format!("exit {code} · {elapsed}")),
        Status::ShellAfterExit { code } => ("⌁", theme.info, format!("shell (exited {code}) · press ^D to close")),
        Status::Broken => ("!", theme.warn, "broken".into()),
    };

    let mut left: Vec<Span> = vec![
        bar,
        Span::raw(" "),
        Span::styled(
            meta.recipe_name.clone(),
            Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(glyph, Style::default().fg(glyph_color)),
        Span::raw(" "),
        Span::styled(label, Style::default().fg(theme.dim)),
    ];

    let pid_text = meta.pid.map(|p| format!("pid {p} · logs ↗")).unwrap_or_else(|| "logs ↗".into());
    let right = Span::styled(pid_text, Style::default().fg(theme.dim));
    let used: usize = left.iter().map(|s| s.content.chars().count()).sum::<usize>()
        + right.content.chars().count();
    if (used as u16) < area.width {
        left.push(Span::raw(" ".repeat(area.width as usize - used)));
    }
    left.push(right);

    f.render_widget(Paragraph::new(Line::from(left)), area);
}

fn fmt_elapsed(d: std::time::Duration) -> String {
    let secs = d.as_secs();
    if secs < 60 { format!("{secs}s") }
    else if secs < 3600 { format!("{}m", secs / 60) }
    else { format!("{}h", secs / 3600) }
}
```

- [ ] **Step 2: Write `scrollbar.rs`**

```rust
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;

pub fn render(
    buf: &mut Buffer,
    area: Rect,
    total_rows: usize,
    viewport_rows: usize,
    top_row: usize,
    theme: &crate::theme::Theme,
) {
    if total_rows <= viewport_rows || area.height == 0 {
        return;
    }
    let track_h = area.height as usize;
    let thumb_h = ((viewport_rows * track_h) / total_rows).max(1);
    let thumb_y = (top_row * track_h) / total_rows;
    for y in 0..track_h {
        let cell = buf.get_mut(area.x, area.y + y as u16);
        let is_thumb = y >= thumb_y && y < thumb_y + thumb_h;
        cell.set_symbol("│");
        cell.set_style(Style::default().fg(if is_thumb { theme.accent } else { theme.dim }));
    }
}
```

- [ ] **Step 3: Rewrite `session_pane.rs`**

```rust
use crate::app::types::SessionMeta;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

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
        .constraints([Constraint::Length(1), Constraint::Min(1)])
        .split(area);
    let header_area = rows[0];
    let body_area = rows[1];

    crate::ui::session_header::render(f, header_area, meta, active, theme);

    // reserve last column of body for the scroll thumb
    let grid_area = Rect { width: body_area.width.saturating_sub(1), ..body_area };
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
            if let Some(cell) = grid.cell(r as u16, c as u16) {
                let mut style = Style::default();
                if let Some(color) = convert_color(cell.fgcolor()) { style = style.fg(color); }
                if let Some(color) = convert_color(cell.bgcolor()) { style = style.bg(color); }
                if cell.bold() { style = style.add_modifier(Modifier::BOLD); }
                if cell.italic() { style = style.add_modifier(Modifier::ITALIC); }
                if cell.underline() { style = style.add_modifier(Modifier::UNDERLINED); }
                let ch = cell.contents();
                spans.push(Span::styled(if ch.is_empty() { " ".into() } else { ch }, style));
            } else {
                spans.push(Span::raw(" "));
            }
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

fn scrollback_dims(screen: &vt100::Parser, viewport: usize) -> (usize, usize) {
    let grid = screen.screen();
    let total = viewport + grid.scrollback() as usize; // placeholder: adapt to the vt100 API actually in use
    let top = grid.scrollback() as usize;
    (total, top)
}

fn convert_color(c: vt100::Color) -> Option<Color> {
    use vt100::Color as V;
    match c {
        V::Default => None,
        V::Idx(0) => Some(Color::Black),
        V::Idx(1) => Some(Color::Red),
        V::Idx(2) => Some(Color::Green),
        V::Idx(3) => Some(Color::Yellow),
        V::Idx(4) => Some(Color::Blue),
        V::Idx(5) => Some(Color::Magenta),
        V::Idx(6) => Some(Color::Cyan),
        V::Idx(7) => Some(Color::Gray),
        V::Idx(8) => Some(Color::DarkGray),
        V::Idx(9) => Some(Color::LightRed),
        V::Idx(10) => Some(Color::LightGreen),
        V::Idx(11) => Some(Color::LightYellow),
        V::Idx(12) => Some(Color::LightBlue),
        V::Idx(13) => Some(Color::LightMagenta),
        V::Idx(14) => Some(Color::LightCyan),
        V::Idx(15) => Some(Color::White),
        V::Idx(n) => Some(Color::Indexed(n)),
        V::Rgb(r, g, b) => Some(Color::Rgb(r, g, b)),
    }
}
```

- [ ] **Step 4: Update `mod.rs` render dispatch**

In `src/ui/mod.rs`, change the session branch to pass `SessionMeta`:

```rust
if let Some(id) = app.active_session {
    if let (Some(screen), Some(meta)) = (screens.get(&id), app.session(id)) {
        session_pane::render(f, panes.right, screen, meta, right_active, &app.theme);
    } else {
        preview::render(f, panes.right, app, &app.theme);
    }
} else {
    preview::render(f, panes.right, app, &app.theme);
}
```

Also add `pub mod session_header;` and `pub mod scrollbar;`.

- [ ] **Step 5: Run build**

Run: `cargo check --all-targets`
Expected: PASS.

- [ ] **Step 6: Verify `color-gate`**

Run: `just color-gate`
Expected: PASS (session_pane.rs is excluded by glob).

- [ ] **Step 7: Commit chrome rewrite**

```bash
git add src/ui/focus.rs src/ui/top_bar.rs src/ui/list.rs src/ui/preview.rs \
        src/ui/session_pane.rs src/ui/session_header.rs src/ui/scrollbar.rs src/ui/mod.rs
git commit -m "feat(ui): rewrite top_bar/list/preview/session_pane per redesign"
```

---

## Task 11: New status bar

**Files:**
- Modify: `src/ui/status_bar.rs`, `src/ui/mod.rs`

- [ ] **Step 1: Replace the file**

```rust
use crate::app::types::Mode;
use crate::app::App;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

pub fn render(f: &mut Frame, area: Rect, app: &App, theme: &crate::theme::Theme) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    let hint = hint_for(&app.mode, app, theme);
    f.render_widget(Paragraph::new(hint), cols[0]);

    if let Some(msg) = &app.status_message {
        let style = Style::default().fg(if msg.starts_with("err") || msg.starts_with("Error") {
            theme.error
        } else {
            theme.warn
        });
        let right = Paragraph::new(Line::from(Span::styled(msg.clone(), style)))
            .alignment(ratatui::layout::Alignment::Right);
        f.render_widget(right, cols[1]);
    }
}

fn hint_for<'a>(mode: &'a Mode, app: &'a App, theme: &crate::theme::Theme) -> Line<'a> {
    let sep = Span::styled("  ·  ", Style::default().fg(theme.dim));
    let k = |s: &str| Span::styled(s.to_string(), Style::default().fg(theme.fg));
    let d = |s: &str| Span::styled(s.to_string(), Style::default().fg(theme.dim));
    match mode {
        Mode::Normal => Line::from(vec![
            k("⏎"), Span::raw(" "), d("run"), sep.clone(),
            k("/"), Span::raw(" "), d("filter"), sep.clone(),
            k("t"), Span::raw(" "), d("theme"), sep.clone(),
            k("?"), Span::raw(" "), d("help"), sep,
            k("q"), Span::raw(" "), d("quit"),
        ]),
        Mode::FilterInput => Line::from(vec![
            d("/"), k(&format!("{}_", app.filter)), sep.clone(),
            k("Esc"), Span::raw(" "), d("cancel"), sep,
            k("⏎"), Span::raw(" "), d("apply"),
        ]),
        Mode::ParamInput { values, cursor, recipe_idx } => {
            let jf = app.active_justfile();
            let name = jf
                .and_then(|j| j.recipes.get(*recipe_idx))
                .and_then(|r| r.params.get(*cursor))
                .map(|p| p.name.as_str())
                .unwrap_or("");
            let val = values.get(*cursor).cloned().unwrap_or_default();
            Line::from(vec![
                d(&format!("[{}/{}] ", cursor + 1, values.len().max(1))),
                k(&format!("{name} = {val}_")),
                sep.clone(),
                k("⏎"), Span::raw(" "), d("next"),
                sep,
                k("Esc"), Span::raw(" "), d("cancel"),
            ])
        }
        Mode::ThemePicker { .. } => Line::from(vec![
            k("j/k"), Span::raw(" "), d("select"), sep.clone(),
            k("⏎"), Span::raw(" "), d("apply & save"), sep,
            k("Esc"), Span::raw(" "), d("revert"),
        ]),
        Mode::Help { .. } => Line::from(vec![k("Esc / q"), Span::raw(" "), d("close")]),
        Mode::Confirm { prompt, .. } => Line::from(vec![
            d(prompt), Span::raw(" "),
            k("y"), Span::raw(" "), d("yes"), sep, k("n"), Span::raw(" "), d("no"),
        ]),
        Mode::Dropdown { filter, .. } => Line::from(vec![
            d("justfile: /"), k(&format!("{filter}_")), sep.clone(),
            k("⏎"), Span::raw(" "), d("pick"), sep,
            k("Esc"), Span::raw(" "), d("cancel"),
        ]),
        Mode::ErrorsList => Line::from(vec![k("Esc / q / e"), Span::raw(" "), d("close")]),
    }
}
```

- [ ] **Step 2: Update the call site**

In `src/ui/mod.rs` the line `status_bar::render(f, panes.status, app);` becomes `status_bar::render(f, panes.status, app, &app.theme);`.

- [ ] **Step 3: Build + color-gate**

```bash
cargo check --all-targets
just color-gate
```

Expected: both PASS.

- [ ] **Step 4: Commit**

```bash
git add src/ui/status_bar.rs src/ui/mod.rs
git commit -m "feat(ui): new status bar with per-mode hints and warn/error right pane"
```

---

## Task 12: Modal shared base + restyle all modals

**Files:**
- Create: `src/ui/modal_base.rs`
- Modify: `src/ui/modal.rs`, `src/ui/help.rs`, `src/ui/param_modal.rs`, `src/ui/theme_picker.rs`, `src/ui/mod.rs`

- [ ] **Step 1: Write `modal_base.rs`**

```rust
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, BorderType, Borders, Clear};
use ratatui::Frame;

pub fn centered(parent: Rect, w: u16, h: u16) -> Rect {
    let v = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(parent.height.saturating_sub(h) / 2),
            Constraint::Length(h),
            Constraint::Min(0),
        ])
        .split(parent);
    let h_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(parent.width.saturating_sub(w) / 2),
            Constraint::Length(w),
            Constraint::Min(0),
        ])
        .split(v[1]);
    h_cols[1]
}

pub fn block<'a>(title: &'a str, theme: &crate::theme::Theme) -> Block<'a> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.accent))
        .title(Span::styled(
            format!(" {title} "),
            Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().fg(theme.fg).bg(theme.bg))
}

pub fn clear(f: &mut Frame, area: Rect) {
    f.render_widget(Clear, area);
}
```

Register in `src/ui/mod.rs`: `pub mod modal_base;`.

- [ ] **Step 2: Migrate `modal.rs`**

Remove the local `centered` fn. Use `modal_base::centered` + `modal_base::block` + `modal_base::clear` for every modal. Delete the `render_dropdown`/`render_confirm`/`render_errors` hardcoded borders; they now call into `modal_base::block("dropdown", theme)` etc. The dropdown highlight already uses theme slots — keep.

- [ ] **Step 3: Migrate `help.rs`**

Replace the `Block::default().borders(Borders::ALL).title("help")` with `modal_base::block("help", theme)`. Remove the local title/border style.

- [ ] **Step 4: Migrate `param_modal.rs`**

Replace its local block with `modal_base::block("params", theme)`. Keep the body layout.

- [ ] **Step 5: Migrate `theme_picker.rs`**

Replace the local block with `modal_base::block("theme", theme)`. Move the bottom-hint line under the picker into the parent status bar logic — actually spec §312 says the theme-picker hint lives in the status bar, so delete the bottom-hint chunk here and remove the `Layout` split; render the list in the full inner area.

- [ ] **Step 6: Build + color-gate + tests**

```bash
cargo check --all-targets
just color-gate
cargo test --all-targets
```

Existing snapshots will fail — accept intentional drift in Task 14.

- [ ] **Step 7: Commit**

```bash
git add src/ui/modal_base.rs src/ui/modal.rs src/ui/help.rs src/ui/param_modal.rs src/ui/theme_picker.rs src/ui/mod.rs
git commit -m "feat(ui): shared rounded modal base; restyle help/errors/confirm/param/theme"
```

---

## Task 13: Second built-in theme for snapshot fan-out

**Files:**
- Create: `assets/themes/mono-amber.toml` (if not present)
- Modify: `src/theme/builtin.rs`

- [ ] **Step 1: Check whether mono-amber already ships**

Run: `ls assets/themes/ 2>/dev/null && rg -n 'mono-amber' src/theme/`

If present: skip this task.

- [ ] **Step 2: Copy an existing theme as the seed**

Pick `tokyo-night.toml` contents; duplicate as `assets/themes/mono-amber.toml`; rewrite every color slot to amber-on-black:

```toml
name = "mono-amber"
bg = "#000000"
fg = "#ffb000"
dim = "#805800"
accent = "#ffcc33"
highlight = "#402c00"
selected_fg = "#ffe08a"
success = "#8fb000"
warn = "#ffcc33"
error = "#ff6040"
running = "#ffb000"
info = "#cc9000"
badge_bg = "#402c00"
badge_fg = "#ffcc33"
```

- [ ] **Step 3: Register in `builtin.rs`**

Wherever the registry has `include_str!("../../assets/themes/tokyo-night.toml")`, add an equivalent line for mono-amber and add a `("mono-amber", MONO_AMBER_TOML)` entry to the registry map.

- [ ] **Step 4: Test registry**

Run: `cargo test -p lazyjust --test theme_registry`
Expected: PASS (existing coverage).

- [ ] **Step 5: Commit**

```bash
git add assets/themes/mono-amber.toml src/theme/builtin.rs
git commit -m "feat(theme): add mono-amber built-in for snapshot fan-out"
```

---

## Task 14: Snapshot fan-out — theme × size × icon_style × state

**Files:**
- Modify: `tests/snapshots.rs`

- [ ] **Step 1: Factor `fixture_app` to accept theme + size + icon_style**

```rust
fn fixture_app(theme_name: &str, icon_style: IconStyle) -> App {
    let theme = lazyjust::theme::registry::resolve(theme_name);
    // …existing recipes…
    App::new(vec![jf], vec![], 0.3, theme, theme_name.into(), icon_style)
}
```

- [ ] **Step 2: Fan out `initial_render_snapshot` via a macro**

```rust
macro_rules! snap_case {
    ($name:ident, $theme:literal, $w:expr, $h:expr, $style:expr, $state:expr) => {
        #[test]
        fn $name() {
            let backend = TestBackend::new($w, $h);
            let mut terminal = Terminal::new(backend).unwrap();
            let mut app = fixture_app($theme, $style);
            $state(&mut app);
            let screens = ui::SessionScreens::new();
            terminal.draw(|f| ui::render(f, &app, &screens)).unwrap();
            let buf = terminal.backend().buffer().clone();
            insta::assert_snapshot!(buffer_to_string(&buf));
        }
    };
}
```

Cases required by spec §348-360:

- Top bar: no errors, with errors, long path truncation → 3 cases.
- List: grouped, ungrouped-only, mixed, short deps, long-deps truncated, no deps, all session-status states on same row → 7 cases.
- Preview: no-deps no-params, deps only, params only, both, long doc line wrap → 5 cases.
- Session header: running, exited-0, exited-nonzero, broken, shell-after-exit → 5 cases.
- Status bar: per mode (Normal, Filter, ParamInput, Help, Confirm, ThemePicker, ErrorsList) → 7 cases.
- Modals: help, errors, confirm, param, theme picker → 5 cases.
- Layouts: 40×10, 80×24, 160×50 × (Normal initial only) → 3 cases.
- Icon styles: 1 list snapshot per round/ascii/none → 3 cases.

Each case runs under **both** themes (`tokyo-night` + `mono-amber`) via the macro.

Spell every case out individually with a concrete fixture-mutation closure that prepares state. Do not use `"TODO"` — either fully write the case or drop it.

- [ ] **Step 3: Accept the first run**

Run: `cargo insta test --accept --unreferenced=delete -- --test snapshots`
Review every new `.snap` file manually before commit — open them and scan for sane output (accent bleed, section bars, pills).

- [ ] **Step 4: Re-run and verify deterministic**

Run: `cargo test -p lazyjust --test snapshots`
Expected: PASS, zero diffs.

- [ ] **Step 5: Commit**

```bash
git add tests/snapshots.rs tests/snapshots/
git commit -m "test(ui): fan out snapshots across themes, sizes, icon styles, states"
```

---

## Task 15: Icon-style integration test

**Files:**
- Modify: `tests/snapshots.rs` or new `tests/icon_style.rs`

- [ ] **Step 1: Cover round/ascii/none explicitly**

A minimal list render at 80×10 under each icon style, a cursor row, one unselected, one running session indicator. Snapshot.

- [ ] **Step 2: Accept and commit**

```bash
cargo insta test --accept
git add tests/
git commit -m "test(ui): snapshot list under each icon_style"
```

---

## Task 16: Theme-picker flow integration test

**Files:**
- Create: `tests/theme_picker_flow.rs`
- Uses: the same reducer interface the reducer tests already use.

- [ ] **Step 1: Write the test**

```rust
use lazyjust::app::action::Action;
use lazyjust::app::reducer::apply;
use lazyjust::app::App;

fn build_app() -> App {
    // same fixture as tests/snapshots.rs::fixture_app but with
    // lazyjust::theme::registry::resolve("tokyo-night").
    todo!("reuse the shared fixture helper or inline a minimal App")
}

#[test]
fn esc_reverts_theme_no_write() {
    let tmp = tempfile::tempdir().unwrap();
    std::env::set_var("LAZYJUST_CONFIG_DIR", tmp.path());
    let cfg_path = tmp.path().join("config.toml");
    std::fs::write(&cfg_path, "[ui]\ntheme = \"tokyo-night\"\n").unwrap();

    let mut app = build_app();
    apply(&mut app, Action::OpenThemePicker);
    apply(&mut app, Action::PickerMove(1)); // preview applies live
    assert_ne!(app.theme_name, "tokyo-night");
    apply(&mut app, Action::PickerCancel);
    assert_eq!(app.theme_name, "tokyo-night");
    assert_eq!(
        std::fs::read_to_string(&cfg_path).unwrap(),
        "[ui]\ntheme = \"tokyo-night\"\n",
    );
    std::env::remove_var("LAZYJUST_CONFIG_DIR");
}

#[test]
fn enter_writes_theme_only_preserving_other_keys() {
    let tmp = tempfile::tempdir().unwrap();
    std::env::set_var("LAZYJUST_CONFIG_DIR", tmp.path());
    let cfg_path = tmp.path().join("config.toml");
    let original = "# user comment\n[ui]\ntheme = \"tokyo-night\"\n\n[engine]\nrender_throttle_ms = 8\n";
    std::fs::write(&cfg_path, original).unwrap();

    let mut app = build_app();
    apply(&mut app, Action::OpenThemePicker);
    apply(&mut app, Action::PickerMove(1));
    apply(&mut app, Action::PickerConfirm);

    let after = std::fs::read_to_string(&cfg_path).unwrap();
    assert!(after.contains("# user comment"));
    assert!(after.contains("render_throttle_ms = 8"));
    assert!(!after.contains("theme = \"tokyo-night\""));
    std::env::remove_var("LAZYJUST_CONFIG_DIR");
}
```

Replace the `todo!()` in `build_app` with a real fixture before running. The `Action::PickerMove(isize)` / `PickerConfirm` / `PickerCancel` variants exist on the reducer as of M2 — no new reducer work needed.

- [ ] **Step 2: Run**

Run: `cargo test -p lazyjust --test theme_picker_flow`
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add tests/theme_picker_flow.rs
git commit -m "test(theme): picker revert and save paths"
```

---

## Task 17: Wire `color-gate` into CI

**Files:**
- Modify: `.github/workflows/ci.yml`

- [ ] **Step 1: Add step**

After the `test` step:

```yaml
      - name: color-gate
        if: runner.os != 'Windows'
        run: just color-gate
```

(Skip Windows because `rg` glob behavior differs and the existing gate uses `!`-prefixed Unix globs.)

- [ ] **Step 2: Verify locally**

Run: `just color-gate`
Expected: PASS.

Intentionally plant a `Color::Red` in `src/ui/list.rs`; run again — expect FAIL. Revert.

- [ ] **Step 3: Commit**

```bash
git add .github/workflows/ci.yml
git commit -m "ci: run color-gate on linux/macos"
```

---

## Task 18: Update template comments

**Files:**
- Modify: `assets/config-template.toml`

- [ ] **Step 1: Mark `icon_style` active**

Change the comment above `icon_style = "round"`:

```toml
# Glyph set for list indicators. "round" | "ascii" | "none".
# Active in M3.
icon_style = "round"
```

Remove the `show_inline_deps = true` line **or** comment-note that it's currently ignored (spec always renders inline deps). Prefer remove — cleaner.

- [ ] **Step 2: Run template parse test**

Run: `cargo test -p lazyjust --lib config::template`
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add assets/config-template.toml
git commit -m "docs(config): mark icon_style active, drop vaporware inline-deps toggle"
```

---

## Task 19: Manual verification pass

Not a code change — run the binary and work through the spec's manual checklist (§389):

- [ ] Run every mode: Normal, Filter, ParamInput, Help, Confirm, Dropdown, ThemePicker, ErrorsList. Each draws the new status-bar hint; modal borders are rounded and accent-colored.
- [ ] Resize: 40×10 boundary (min, still OK), 39×10 (too-small screen themed), 160×50 (wide layout breathes).
- [ ] Spawn a recipe with deps → inline deps render under list row; preview "depends on" block lists each dep.
- [ ] Spawn a recipe with params → param input bar shows `[1/N] name = _`; run completes; session header shows pid + running + elapsed.
- [ ] Kill a running session → confirm prompt styled; on accept, session header shows exit code non-zero with `✗`.
- [ ] Let a session hit `shell` after exit → header shows `⌁ shell (exited 0) · press ^D to close`.
- [ ] Cycle icon styles in `config.toml` (`round` → `ascii` → `none`); restart; list glyphs change.
- [ ] Open theme picker (`t`), arrow through; chrome live-previews each theme; Esc reverts; Enter writes `[ui].theme` only — confirm by diffing the config file (other keys and comments preserved).
- [ ] Long doc on a preview recipe wraps without jittering.
- [ ] Session scrollback: PgUp/PgDn shows the thumb moving; at top/bottom the thumb hits the edges; when content fits viewport, no thumb rendered.

Track any deviations → open follow-up issues; do not fold fixes into the M3 merge commit.

- [ ] **Commit nothing for this task** — it's verification.

---

## Task 20: Branch + PR

- [ ] **Step 1: Push**

```bash
git push -u origin feat/ui-redesign-m3
```

- [ ] **Step 2: Open PR against main with the spec's exit criteria checklist**

Copy the spec's "Exit criteria for Milestone 3" items into the PR body. CI must pass (fmt, clippy, test, color-gate).

---

## Exit criteria (from spec §389 / §375)

- Every chrome file uses only theme slots — `just color-gate` is green and runs in CI.
- Snapshots pass under both `tokyo-night` and `mono-amber`.
- Every mode renders the new status-bar hint, modals share the rounded-border base.
- Session header shows pid, elapsed, and status pill; scroll thumb reflects scrollback position.
- `[ui].icon_style` round/ascii/none end-to-end from config to list render.
- No config or theme engine changes beyond `icon_style` and the mono-amber built-in.
- No behavior changes to reducer, event loop, or session manager beyond pid capture.
