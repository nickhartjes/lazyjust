# UI Redesign — Milestone 2: Theme System + Picker Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a themeable color system with 11 built-in themes, user-defined theme TOML files under `<config_dir>/lazyjust/themes/`, and an in-app theme picker modal (key `t`) that live-previews on highlight and persists the choice back to `config.toml` via comment-preserving `toml_edit`.

**Architecture:** A new `src/theme/` module owns `Theme` (14 semantic color slots), color parsing (hex / named / 256-palette), built-in theme registry (TOMLs embedded via `include_str!`), and user-theme loading from the config dir. `App` holds a `Theme` built at startup from the resolved `[ui].theme` value. All chrome rendering in `src/ui/*` replaces hardcoded `Color::Cyan` / `Color::Yellow` etc. with `theme.accent` / `theme.fg` / etc. A new `Mode::ThemePicker` with live preview plus an `Enter`-to-save path using `toml_edit::DocumentMut` completes the loop. Zero layout changes (that's M3) — this is purely a color + mode addition.

**Tech Stack:** Rust, existing `lazyjust` crate, adds `toml_edit = "0.22"`. Reuses `ratatui::style::Color`, `serde`, `tracing`, existing `insta` snapshots.

**Spec:** `docs/superpowers/specs/2026-04-24-ui-redesign-design.md` (Milestone 2 section).

---

## File Structure

| File | Action | Responsibility |
|---|---|---|
| `Cargo.toml` | Modify | Add `toml_edit = "0.22"` for comment-preserving config writes. |
| `src/lib.rs` | Modify | Declare `pub mod theme;`. |
| `src/theme/mod.rs` | Create | Public `Theme` struct + default theme name constant. Re-exports. |
| `src/theme/parse.rs` | Create | `parse_color(&str) -> Result<Color, ThemeError>` (hex / named / index). Serde `ThemeFile` with all 14 slots required. `Theme::from_toml(&str)`. |
| `src/theme/builtin.rs` | Create | `pub const BUILTIN_THEMES: &[(&str, &str)]` mapping name → embedded TOML string via `include_str!`. |
| `src/theme/registry.rs` | Create | `resolve(name: &str) -> Theme` — user-dir shadow → built-in → fallback-to-default. `list() -> Vec<String>` — all available theme names deduped/sorted. |
| `assets/themes/catppuccin-latte.toml` | Create | Built-in theme. |
| `assets/themes/catppuccin-frappe.toml` | Create | Built-in theme. |
| `assets/themes/catppuccin-macchiato.toml` | Create | Built-in theme. |
| `assets/themes/catppuccin-mocha.toml` | Create | Built-in theme. |
| `assets/themes/tokyo-night.toml` | Create | Built-in theme (default). |
| `assets/themes/gruvbox-dark.toml` | Create | Built-in theme. |
| `assets/themes/dracula.toml` | Create | Built-in theme. |
| `assets/themes/nord.toml` | Create | Built-in theme. |
| `assets/themes/solarized-dark.toml` | Create | Built-in theme. |
| `assets/themes/one-dark.toml` | Create | Built-in theme. |
| `assets/themes/mono-amber.toml` | Create | Built-in theme. |
| `src/config/file.rs` | Modify | Add `pub ui: Option<UiSection>` with `theme: Option<String>`. |
| `src/config/defaults.rs` | Modify | Add `theme_name: "tokyo-night".into()` to defaults. |
| `src/config/merge.rs` | Modify | Merge `ui.theme` → `Config.theme_name`. |
| `src/config.rs` | Modify | Add `pub theme_name: String` to `Config`. |
| `src/config/writer.rs` | Create | `pub fn set_theme(path: &Path, name: &str) -> Result<(), WriterError>` using `toml_edit::DocumentMut` — preserves comments, writes a fresh file if missing. |
| `src/app/state.rs` | Modify | Add `pub theme: Theme` field. `App::new` takes a `Theme`. |
| `src/app/types.rs` | Modify | Add `Mode::ThemePicker { original_name: String, highlighted: usize, names: Vec<String> }`. |
| `src/app/event_loop.rs` | Modify | `t` key in Normal opens picker; `j`/`k` / arrows move highlight + live-apply; `Enter` persists + closes; `Esc` restores `original_name` + closes. |
| `src/app/reducer.rs` | Modify | Action handlers for picker open / highlight-change / confirm / cancel. |
| `src/app/action.rs` | Modify | New actions: `OpenThemePicker`, `PickerHighlight(isize)`, `PickerConfirm`, `PickerCancel`. |
| `src/ui/theme_picker.rs` | Create | Centered modal listing theme names with highlight bar. |
| `src/ui/mod.rs` | Modify | Expose `theme_picker`. Dispatch from top-level render based on `Mode::ThemePicker`. |
| `src/ui/top_bar.rs` | Modify | Replace `Color::Cyan/Yellow/Gray/Red` with theme slots. |
| `src/ui/list.rs` | Modify | Replace status-dot and selection colors with theme slots. |
| `src/ui/preview.rs` | Modify | Replace heading/doc/hint colors with theme slots. |
| `src/ui/focus.rs` | Modify | Replace focused-border colors with theme slots. |
| `src/ui/modal.rs` | Modify | Replace border / title / subtitle colors with theme slots. |
| `src/ui/param_modal.rs` | Modify | Replace hint color with theme slot. |
| `src/ui/help.rs` | Modify | Replace border / accent colors with theme slots. |
| `src/ui/status_bar.rs` | Modify | Any hardcoded color → theme slot. (Audit during Task 13.) |
| `tests/theme_registry.rs` | Create | Integration test: all 11 built-ins resolve; user-dir override works; missing name falls back. |
| `tests/theme_picker_persist.rs` | Create | Integration test: writing a theme via `set_theme` preserves comments. |
| `justfile` | Modify | Add `color-gate` recipe that `rg`-greps `src/ui/` for hardcoded `Color::*` (other than vt100 palette mappings in `session_pane.rs`). |
| `.github/workflows/ci.yml` | Modify (if exists — skip if no CI workflow) | Run `just color-gate` after tests. Otherwise document in README. |

### Out of scope (explicit non-goals for M2)

- Layout changes, new glyphs, section bars, inline deps, session-pane header strip, status-bar consolidation — all M3.
- `[keys]` config-driven remapping. `t` is hardcoded. Plan for a future milestone.
- vt100 palette mapping in `src/ui/session_pane.rs:60-77` — those translate terminal output, not chrome. Keep as-is.
- Command-palette / `:theme` access.
- Snapshot-tests-under-two-themes rig. Single-theme snapshots keep passing after this milestone; multi-theme fan-out lands in M3.

---

## Task 1: Add `toml_edit` dependency

**Files:**
- Modify: `Cargo.toml`

- [ ] **Step 1: Add `toml_edit` to `[dependencies]`**

Insert alphabetically after `toml = "0.8"`:

```toml
toml_edit = "0.22"
```

- [ ] **Step 2: Verify build**

Run: `cargo build`
Expected: builds; `toml_edit 0.22.x` appears in `Cargo.lock` as a direct dep (it was already transitive).

- [ ] **Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock
git commit -m "build: add toml_edit 0.22 for comment-preserving config writes"
```

---

## Task 2: Theme struct + color parser

**Files:**
- Create: `src/theme/mod.rs`
- Create: `src/theme/parse.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: Create `src/theme/mod.rs`**

```rust
//! Color themes. Each theme is a set of 14 semantic color slots used
//! across the UI. Themes load from TOML — either built-in (compiled in
//! via `include_str!`) or user files under `<config_dir>/lazyjust/themes/`.

pub mod builtin;
mod parse;
pub mod registry;

pub use parse::{parse_theme, ThemeError};

use ratatui::style::Color;

pub const DEFAULT_THEME_NAME: &str = "tokyo-night";

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub bg: Color,
    pub fg: Color,
    pub dim: Color,
    pub accent: Color,
    pub highlight: Color,
    pub selected_fg: Color,
    pub success: Color,
    pub warn: Color,
    pub error: Color,
    pub running: Color,
    pub info: Color,
    pub badge_bg: Color,
    pub badge_fg: Color,
}
```

- [ ] **Step 2: Create `src/theme/parse.rs`**

```rust
use super::Theme;
use ratatui::style::Color;
use serde::Deserialize;

#[derive(Debug, thiserror::Error)]
pub enum ThemeError {
    #[error("theme parse error: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("invalid color {0:?}: expected #rrggbb, a named ratatui color, or a 0-255 index")]
    Color(String),
}

#[derive(Debug, Deserialize)]
struct ThemeFile {
    name: String,
    bg: String,
    fg: String,
    dim: String,
    accent: String,
    highlight: String,
    selected_fg: String,
    success: String,
    warn: String,
    error: String,
    running: String,
    info: String,
    badge_bg: String,
    badge_fg: String,
}

pub fn parse_theme(input: &str) -> Result<Theme, ThemeError> {
    let f: ThemeFile = toml::from_str(input)?;
    Ok(Theme {
        name: f.name,
        bg: parse_color(&f.bg)?,
        fg: parse_color(&f.fg)?,
        dim: parse_color(&f.dim)?,
        accent: parse_color(&f.accent)?,
        highlight: parse_color(&f.highlight)?,
        selected_fg: parse_color(&f.selected_fg)?,
        success: parse_color(&f.success)?,
        warn: parse_color(&f.warn)?,
        error: parse_color(&f.error)?,
        running: parse_color(&f.running)?,
        info: parse_color(&f.info)?,
        badge_bg: parse_color(&f.badge_bg)?,
        badge_fg: parse_color(&f.badge_fg)?,
    })
}

fn parse_color(s: &str) -> Result<Color, ThemeError> {
    let t = s.trim();
    // 256-palette index: a bare number 0..=255
    if let Ok(n) = t.parse::<u8>() {
        return Ok(Color::Indexed(n));
    }
    // Hex form: #rrggbb
    if let Some(hex) = t.strip_prefix('#') {
        if hex.len() == 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| ThemeError::Color(s.into()))?;
            let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| ThemeError::Color(s.into()))?;
            let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| ThemeError::Color(s.into()))?;
            return Ok(Color::Rgb(r, g, b));
        }
        return Err(ThemeError::Color(s.into()));
    }
    // Named ratatui color
    Ok(match t.to_ascii_lowercase().as_str() {
        "black" => Color::Black,
        "red" => Color::Red,
        "green" => Color::Green,
        "yellow" => Color::Yellow,
        "blue" => Color::Blue,
        "magenta" => Color::Magenta,
        "cyan" => Color::Cyan,
        "gray" | "grey" => Color::Gray,
        "dark_gray" | "darkgray" | "dark_grey" | "darkgrey" => Color::DarkGray,
        "light_red" | "lightred" | "bright_red" | "brightred" => Color::LightRed,
        "light_green" | "lightgreen" | "bright_green" | "brightgreen" => Color::LightGreen,
        "light_yellow" | "lightyellow" | "bright_yellow" | "brightyellow" => Color::LightYellow,
        "light_blue" | "lightblue" | "bright_blue" | "brightblue" => Color::LightBlue,
        "light_magenta" | "lightmagenta" | "bright_magenta" | "brightmagenta" => Color::LightMagenta,
        "light_cyan" | "lightcyan" | "bright_cyan" | "brightcyan" => Color::LightCyan,
        "white" => Color::White,
        _ => return Err(ThemeError::Color(s.into())),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_parses() {
        assert_eq!(parse_color("#89b4fa").unwrap(), Color::Rgb(137, 180, 250));
    }

    #[test]
    fn named_parses_case_insensitive() {
        assert_eq!(parse_color("Blue").unwrap(), Color::Blue);
        assert_eq!(parse_color("bright_red").unwrap(), Color::LightRed);
    }

    #[test]
    fn palette_index_parses() {
        assert_eq!(parse_color("21").unwrap(), Color::Indexed(21));
    }

    #[test]
    fn empty_string_is_error() {
        assert!(matches!(parse_color(""), Err(ThemeError::Color(_))));
    }

    #[test]
    fn short_hex_is_error() {
        assert!(matches!(parse_color("#abc"), Err(ThemeError::Color(_))));
    }

    #[test]
    fn unknown_name_is_error() {
        assert!(matches!(parse_color("mauve"), Err(ThemeError::Color(_))));
    }

    #[test]
    fn theme_round_trips() {
        let toml = r#"
            name = "test"
            bg = "#000000"
            fg = "#ffffff"
            dim = "gray"
            accent = "cyan"
            highlight = "#202020"
            selected_fg = "white"
            success = "green"
            warn = "yellow"
            error = "red"
            running = "blue"
            info = "cyan"
            badge_bg = "dark_gray"
            badge_fg = "white"
        "#;
        let t = parse_theme(toml).unwrap();
        assert_eq!(t.name, "test");
        assert_eq!(t.bg, Color::Rgb(0, 0, 0));
        assert_eq!(t.dim, Color::Gray);
        assert_eq!(t.accent, Color::Cyan);
    }

    #[test]
    fn missing_slot_fails() {
        // no `badge_fg`
        let toml = r#"
            name = "test"
            bg = "#000000"
            fg = "#ffffff"
            dim = "gray"
            accent = "cyan"
            highlight = "#202020"
            selected_fg = "white"
            success = "green"
            warn = "yellow"
            error = "red"
            running = "blue"
            info = "cyan"
            badge_bg = "dark_gray"
        "#;
        assert!(matches!(parse_theme(toml), Err(ThemeError::Parse(_))));
    }
}
```

- [ ] **Step 3: Register the `theme` module in `src/lib.rs`**

In `src/lib.rs`, after `pub mod session;` and before `pub mod ui;`, add:

```rust
pub mod theme;
```

(Alphabetical insertion.)

- [ ] **Step 4: Create `src/theme/builtin.rs` and `src/theme/registry.rs` as empty stubs so `mod.rs` compiles**

`src/theme/builtin.rs`:

```rust
// Filled in by Task 3.
pub const BUILTIN_THEMES: &[(&str, &str)] = &[];
```

`src/theme/registry.rs`:

```rust
// Filled in by Task 4.
use super::{Theme, DEFAULT_THEME_NAME};

pub fn resolve(_name: &str) -> Theme {
    // Minimal placeholder so callers can compile before Task 4.
    super::parse_theme(
        r#"
        name = "placeholder"
        bg = "black"
        fg = "white"
        dim = "gray"
        accent = "cyan"
        highlight = "dark_gray"
        selected_fg = "white"
        success = "green"
        warn = "yellow"
        error = "red"
        running = "blue"
        info = "cyan"
        badge_bg = "dark_gray"
        badge_fg = "white"
    "#,
    )
    .expect("placeholder theme must parse")
}

pub fn list() -> Vec<String> {
    vec![DEFAULT_THEME_NAME.to_string()]
}
```

- [ ] **Step 5: Run tests**

Run: `cargo test --lib theme::parse`
Expected: all 8 tests pass.

Run: `cargo test --all`
Expected: full suite green (67 + 8 = 75).

- [ ] **Step 6: Commit**

```bash
git add src/theme src/lib.rs
git commit -m "feat(theme): Theme struct + color parser + module skeleton"
```

---

## Task 3: Built-in theme TOML files + registry

**Files:**
- Create: `assets/themes/catppuccin-latte.toml`
- Create: `assets/themes/catppuccin-frappe.toml`
- Create: `assets/themes/catppuccin-macchiato.toml`
- Create: `assets/themes/catppuccin-mocha.toml`
- Create: `assets/themes/tokyo-night.toml`
- Create: `assets/themes/gruvbox-dark.toml`
- Create: `assets/themes/dracula.toml`
- Create: `assets/themes/nord.toml`
- Create: `assets/themes/solarized-dark.toml`
- Create: `assets/themes/one-dark.toml`
- Create: `assets/themes/mono-amber.toml`
- Modify: `src/theme/builtin.rs`

- [ ] **Step 1: Create each theme TOML with the 14 slots**

Each file below. All colors are from published project palettes.

`assets/themes/tokyo-night.toml` (default):

```toml
name         = "Tokyo Night"
bg           = "#1a1b26"
fg           = "#c0caf5"
dim          = "#565f89"
accent       = "#7aa2f7"
highlight    = "#283457"
selected_fg  = "#ffffff"
success      = "#9ece6a"
warn         = "#e0af68"
error        = "#f7768e"
running      = "#7aa2f7"
info         = "#7dcfff"
badge_bg     = "#1f2335"
badge_fg     = "#a9b1d6"
```

`assets/themes/catppuccin-mocha.toml`:

```toml
name         = "Catppuccin Mocha"
bg           = "#1e1e2e"
fg           = "#cdd6f4"
dim          = "#6c7086"
accent       = "#cba6f7"
highlight    = "#313244"
selected_fg  = "#f5e0dc"
success      = "#a6e3a1"
warn         = "#f9e2af"
error        = "#f38ba8"
running      = "#89b4fa"
info         = "#74c7ec"
badge_bg     = "#313244"
badge_fg     = "#cdd6f4"
```

`assets/themes/catppuccin-macchiato.toml`:

```toml
name         = "Catppuccin Macchiato"
bg           = "#24273a"
fg           = "#cad3f5"
dim          = "#6e738d"
accent       = "#c6a0f6"
highlight    = "#363a4f"
selected_fg  = "#f4dbd6"
success      = "#a6da95"
warn         = "#eed49f"
error        = "#ed8796"
running      = "#8aadf4"
info         = "#7dc4e4"
badge_bg     = "#363a4f"
badge_fg     = "#cad3f5"
```

`assets/themes/catppuccin-frappe.toml`:

```toml
name         = "Catppuccin Frappé"
bg           = "#303446"
fg           = "#c6d0f5"
dim          = "#737994"
accent       = "#ca9ee6"
highlight    = "#414559"
selected_fg  = "#f2d5cf"
success      = "#a6d189"
warn         = "#e5c890"
error        = "#e78284"
running      = "#8caaee"
info         = "#85c1dc"
badge_bg     = "#414559"
badge_fg     = "#c6d0f5"
```

`assets/themes/catppuccin-latte.toml`:

```toml
name         = "Catppuccin Latte"
bg           = "#eff1f5"
fg           = "#4c4f69"
dim          = "#8c8fa1"
accent       = "#8839ef"
highlight    = "#dce0e8"
selected_fg  = "#1e1e2e"
success      = "#40a02b"
warn         = "#df8e1d"
error        = "#d20f39"
running      = "#1e66f5"
info         = "#04a5e5"
badge_bg     = "#dce0e8"
badge_fg     = "#4c4f69"
```

`assets/themes/gruvbox-dark.toml`:

```toml
name         = "Gruvbox Dark"
bg           = "#282828"
fg           = "#ebdbb2"
dim          = "#7c6f64"
accent       = "#fe8019"
highlight    = "#3c3836"
selected_fg  = "#fbf1c7"
success      = "#b8bb26"
warn         = "#fabd2f"
error        = "#fb4934"
running      = "#83a598"
info         = "#8ec07c"
badge_bg     = "#3c3836"
badge_fg     = "#ebdbb2"
```

`assets/themes/dracula.toml`:

```toml
name         = "Dracula"
bg           = "#282a36"
fg           = "#f8f8f2"
dim          = "#6272a4"
accent       = "#bd93f9"
highlight    = "#44475a"
selected_fg  = "#ffffff"
success      = "#50fa7b"
warn         = "#f1fa8c"
error        = "#ff5555"
running      = "#8be9fd"
info         = "#8be9fd"
badge_bg     = "#44475a"
badge_fg     = "#f8f8f2"
```

`assets/themes/nord.toml`:

```toml
name         = "Nord"
bg           = "#2e3440"
fg           = "#d8dee9"
dim          = "#616e88"
accent       = "#88c0d0"
highlight    = "#434c5e"
selected_fg  = "#eceff4"
success      = "#a3be8c"
warn         = "#ebcb8b"
error        = "#bf616a"
running      = "#81a1c1"
info         = "#8fbcbb"
badge_bg     = "#3b4252"
badge_fg     = "#d8dee9"
```

`assets/themes/solarized-dark.toml`:

```toml
name         = "Solarized Dark"
bg           = "#002b36"
fg           = "#839496"
dim          = "#586e75"
accent       = "#268bd2"
highlight    = "#073642"
selected_fg  = "#eee8d5"
success      = "#859900"
warn         = "#b58900"
error        = "#dc322f"
running      = "#268bd2"
info         = "#2aa198"
badge_bg     = "#073642"
badge_fg     = "#93a1a1"
```

`assets/themes/one-dark.toml`:

```toml
name         = "One Dark"
bg           = "#282c34"
fg           = "#abb2bf"
dim          = "#5c6370"
accent       = "#61afef"
highlight    = "#3e4451"
selected_fg  = "#ffffff"
success      = "#98c379"
warn         = "#e5c07b"
error        = "#e06c75"
running      = "#61afef"
info         = "#56b6c2"
badge_bg     = "#3e4451"
badge_fg     = "#abb2bf"
```

`assets/themes/mono-amber.toml`:

```toml
name         = "Mono Amber"
bg           = "#0d1117"
fg           = "#c9d1d9"
dim          = "#484f58"
accent       = "#ffb454"
highlight    = "#161b22"
selected_fg  = "#ffb454"
success      = "#ffb454"
warn         = "#ffb454"
error        = "#ff7b72"
running      = "#ffb454"
info         = "#c9d1d9"
badge_bg     = "#161b22"
badge_fg     = "#ffb454"
```

- [ ] **Step 2: Fill in `src/theme/builtin.rs`**

Replace the placeholder with:

```rust
//! Built-in themes. Each entry is (name, raw TOML string) embedded at
//! compile time via `include_str!`. Loaded through the registry in
//! `super::registry`.

pub const BUILTIN_THEMES: &[(&str, &str)] = &[
    (
        "catppuccin-latte",
        include_str!("../../assets/themes/catppuccin-latte.toml"),
    ),
    (
        "catppuccin-frappe",
        include_str!("../../assets/themes/catppuccin-frappe.toml"),
    ),
    (
        "catppuccin-macchiato",
        include_str!("../../assets/themes/catppuccin-macchiato.toml"),
    ),
    (
        "catppuccin-mocha",
        include_str!("../../assets/themes/catppuccin-mocha.toml"),
    ),
    (
        "tokyo-night",
        include_str!("../../assets/themes/tokyo-night.toml"),
    ),
    (
        "gruvbox-dark",
        include_str!("../../assets/themes/gruvbox-dark.toml"),
    ),
    ("dracula", include_str!("../../assets/themes/dracula.toml")),
    ("nord", include_str!("../../assets/themes/nord.toml")),
    (
        "solarized-dark",
        include_str!("../../assets/themes/solarized-dark.toml"),
    ),
    (
        "one-dark",
        include_str!("../../assets/themes/one-dark.toml"),
    ),
    (
        "mono-amber",
        include_str!("../../assets/themes/mono-amber.toml"),
    ),
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::parse_theme;

    #[test]
    fn every_builtin_parses() {
        for (name, toml) in BUILTIN_THEMES {
            parse_theme(toml)
                .unwrap_or_else(|e| panic!("built-in {name:?} failed to parse: {e}"));
        }
    }

    #[test]
    fn default_name_is_registered() {
        assert!(BUILTIN_THEMES
            .iter()
            .any(|(n, _)| *n == super::super::DEFAULT_THEME_NAME));
    }

    #[test]
    fn all_names_unique() {
        let mut seen = std::collections::HashSet::new();
        for (n, _) in BUILTIN_THEMES {
            assert!(seen.insert(*n), "duplicate theme name: {n}");
        }
    }
}
```

- [ ] **Step 3: Run tests**

Run: `cargo test --lib theme::builtin`
Expected: three tests pass.

Run: `cargo test --all`
Expected: suite green.

- [ ] **Step 4: Commit**

```bash
git add assets/themes src/theme/builtin.rs
git commit -m "feat(theme): ship 11 built-in themes with parse validation"
```

---

## Task 4: Theme registry with user-dir shadow + fallback

**Files:**
- Modify: `src/theme/registry.rs`

- [ ] **Step 1: Replace `src/theme/registry.rs` with the real implementation**

```rust
//! Resolves a theme name to a `Theme`. User theme files under
//! `<config_dir>/lazyjust/themes/` shadow built-ins of the same name.
//! Missing / invalid name logs a warning and falls back to the default.

use super::{builtin::BUILTIN_THEMES, parse_theme, Theme, DEFAULT_THEME_NAME};
use std::collections::BTreeSet;
use std::path::PathBuf;

pub fn resolve(name: &str) -> Theme {
    // 1. User theme file, if present.
    if let Some(t) = load_user_theme(name) {
        return t;
    }
    // 2. Built-in with matching name.
    if let Some(t) = load_builtin(name) {
        return t;
    }
    // 3. Fallback to default.
    if name != DEFAULT_THEME_NAME {
        tracing::warn!(
            target: "lazyjust::theme",
            requested = %name,
            fallback = %DEFAULT_THEME_NAME,
            "theme not found, using default",
        );
    }
    load_builtin(DEFAULT_THEME_NAME).expect("default theme must be a built-in")
}

/// All theme names available — built-in + user — deduped and sorted.
pub fn list() -> Vec<String> {
    let mut out: BTreeSet<String> = BTreeSet::new();
    for (n, _) in BUILTIN_THEMES {
        out.insert((*n).to_string());
    }
    if let Some(dir) = user_themes_dir() {
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for e in entries.flatten() {
                let p = e.path();
                if p.extension().and_then(|s| s.to_str()) == Some("toml") {
                    if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
                        out.insert(stem.to_string());
                    }
                }
            }
        }
    }
    out.into_iter().collect()
}

fn user_themes_dir() -> Option<PathBuf> {
    Some(crate::config::paths::user_themes_dir())
}

fn load_user_theme(name: &str) -> Option<Theme> {
    let dir = user_themes_dir()?;
    let path = dir.join(format!("{name}.toml"));
    let contents = std::fs::read_to_string(&path).ok()?;
    match parse_theme(&contents) {
        Ok(t) => Some(t),
        Err(e) => {
            tracing::warn!(
                target: "lazyjust::theme",
                path = %path.display(),
                error = %e,
                "user theme failed to parse; falling through to built-in",
            );
            None
        }
    }
}

fn load_builtin(name: &str) -> Option<Theme> {
    BUILTIN_THEMES
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, raw)| parse_theme(raw).expect("built-in theme must parse"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::paths::OVERRIDE_ENV;

    fn with_themes_dir<T>(files: &[(&str, &str)], body: impl FnOnce() -> T) -> T {
        let tmp = tempfile::tempdir().unwrap();
        let themes = tmp.path().join("themes");
        std::fs::create_dir_all(&themes).unwrap();
        for (name, contents) in files {
            std::fs::write(themes.join(format!("{name}.toml")), contents).unwrap();
        }
        std::env::set_var(OVERRIDE_ENV, tmp.path());
        let out = body();
        std::env::remove_var(OVERRIDE_ENV);
        out
    }

    #[test]
    fn resolves_builtin_by_name() {
        let t = resolve("tokyo-night");
        assert_eq!(t.name, "Tokyo Night");
    }

    #[test]
    fn unknown_name_falls_back_to_default() {
        let t = resolve("this-does-not-exist");
        assert_eq!(t.name, "Tokyo Night");
    }

    #[test]
    fn user_theme_shadows_builtin() {
        let custom = r#"
            name = "Custom Nord"
            bg = "#000001"
            fg = "#ffffff"
            dim = "gray"
            accent = "cyan"
            highlight = "dark_gray"
            selected_fg = "white"
            success = "green"
            warn = "yellow"
            error = "red"
            running = "blue"
            info = "cyan"
            badge_bg = "dark_gray"
            badge_fg = "white"
        "#;
        with_themes_dir(&[("nord", custom)], || {
            let t = resolve("nord");
            assert_eq!(t.name, "Custom Nord");
        });
    }

    #[test]
    fn malformed_user_theme_falls_through_to_builtin() {
        with_themes_dir(&[("nord", "broken = = =")], || {
            let t = resolve("nord");
            // Built-in Nord has name "Nord".
            assert_eq!(t.name, "Nord");
        });
    }

    #[test]
    fn list_includes_builtins_and_user_themes() {
        let custom = r#"
            name = "Ocean"
            bg = "#000001"
            fg = "#ffffff"
            dim = "gray"
            accent = "cyan"
            highlight = "dark_gray"
            selected_fg = "white"
            success = "green"
            warn = "yellow"
            error = "red"
            running = "blue"
            info = "cyan"
            badge_bg = "dark_gray"
            badge_fg = "white"
        "#;
        with_themes_dir(&[("ocean", custom)], || {
            let names = list();
            assert!(names.iter().any(|n| n == "tokyo-night"));
            assert!(names.iter().any(|n| n == "ocean"));
            // sorted + deduped
            let mut expected = names.clone();
            expected.sort();
            expected.dedup();
            assert_eq!(names, expected);
        });
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test --lib theme::registry`
Expected: 5 tests pass.

Run: `cargo test --all`
Expected: suite green.

- [ ] **Step 3: Commit**

```bash
git add src/theme/registry.rs
git commit -m "feat(theme): registry with user-dir shadow + default fallback"
```

---

## Task 5: Wire `[ui].theme` into `Config`

**Files:**
- Modify: `src/config.rs`
- Modify: `src/config/defaults.rs`
- Modify: `src/config/file.rs`
- Modify: `src/config/merge.rs`

- [ ] **Step 1: Add `theme_name` field to `Config` in `src/config.rs`**

In `src/config.rs`, inside the `#[derive(Debug, Clone)] pub struct Config { ... }` block, add after `tick_interval`:

```rust
    /// Resolved theme name — matches a built-in or user theme file.
    pub theme_name: String,
```

- [ ] **Step 2: Update defaults in `src/config/defaults.rs`**

At the end of the returned `Config { ... }` literal, add:

```rust
        theme_name: crate::theme::DEFAULT_THEME_NAME.to_string(),
```

- [ ] **Step 3: Add `UiSection` to `src/config/file.rs`**

In `src/config/file.rs`, add at the bottom of the struct definitions (before the `impl ConfigFile` block):

```rust
#[derive(Debug, Default, Deserialize)]
pub struct UiSection {
    pub theme: Option<String>,
}
```

Also add the field to `ConfigFile`:

```rust
#[derive(Debug, Default, Deserialize)]
pub struct ConfigFile {
    pub ui: Option<UiSection>,
    pub paths: Option<PathsSection>,
    pub logging: Option<LoggingSection>,
    pub engine: Option<EngineSection>,
}
```

(Insert the `ui:` line as the first field of `ConfigFile`.)

- [ ] **Step 4: Merge in `src/config/merge.rs`**

Add a new block inside `pub fn merge(file: ConfigFile, base: Config) -> Config { ... }`, after the `let mut out = base;` line and before the existing `if let Some(p) = file.paths { ... }`:

```rust
    if let Some(u) = file.ui {
        if let Some(theme) = u.theme {
            out.theme_name = theme;
        }
    }
```

- [ ] **Step 5: Add a merge test**

In `src/config/merge.rs`, inside `#[cfg(test)] mod tests`, add:

```rust
    #[test]
    fn ui_theme_override_applies() {
        let file = ConfigFile {
            ui: Some(crate::config::file::UiSection {
                theme: Some("gruvbox-dark".into()),
            }),
            ..Default::default()
        };
        let merged = merge(file, defaults());
        assert_eq!(merged.theme_name, "gruvbox-dark");
    }

    #[test]
    fn ui_theme_defaults_to_tokyo_night() {
        let merged = merge(ConfigFile::default(), defaults());
        assert_eq!(merged.theme_name, "tokyo-night");
    }
```

- [ ] **Step 6: Run tests**

Run: `cargo test --lib config::merge`
Expected: 6 tests pass (4 previous + 2 new).

Run: `cargo test --all`
Expected: suite green.

- [ ] **Step 7: Commit**

```bash
git add src/config.rs src/config/defaults.rs src/config/file.rs src/config/merge.rs
git commit -m "feat(config): read [ui].theme into Config.theme_name"
```

---

## Task 6: `App` holds `Theme`; resolved at startup

**Files:**
- Modify: `src/app/state.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: Add `theme` and `theme_name` fields to `App` in `src/app/state.rs`**

Add to the `App` struct (suggest: after `split_ratio`):

```rust
    pub theme: crate::theme::Theme,
    pub theme_name: String,
```

`theme_name` is the stem (e.g. `"tokyo-night"`) used to look up the theme in the registry and to write back to `[ui].theme`. `theme` is the resolved palette used by every UI render fn.

Modify the `App::new` signature to take both. Replace the entire `impl App { pub fn new(...) -> Self { ... } }` block with:

```rust
impl App {
    pub fn new(
        justfiles: Vec<Justfile>,
        startup_errors: Vec<(PathBuf, String)>,
        split_ratio: f32,
        theme: crate::theme::Theme,
        theme_name: String,
    ) -> Self {
        Self {
            justfiles,
            active_justfile: 0,
            filter: String::new(),
            list_cursor: 0,
            sessions: Vec::new(),
            active_session: None,
            focus: Focus::List,
            mode: Mode::Normal,
            split_ratio,
            theme,
            theme_name,
            collapsed_groups: Default::default(),
            startup_errors,
            next_session_id: 1,
            status_message: None,
        }
    }
}
```

- [ ] **Step 2: Pass theme at startup in `src/lib.rs`**

In `async_main`, replace the line

```rust
let app = app::App::new(disc.justfiles, disc.errors, cfg.default_split_ratio);
```

with:

```rust
let theme = theme::registry::resolve(&cfg.theme_name);
let app = app::App::new(
    disc.justfiles,
    disc.errors,
    cfg.default_split_ratio,
    theme,
    cfg.theme_name.clone(),
);
```

- [ ] **Step 3: Fix call sites**

Run: `cargo build 2>&1 | head -20`
Expected: any test helper or integration test calling `App::new(..., split_ratio)` breaks with "expected 5 arguments".

For every such call site reported by the compiler, add the two trailing arguments:

```rust
crate::theme::registry::resolve(crate::theme::DEFAULT_THEME_NAME),
crate::theme::DEFAULT_THEME_NAME.to_string(),
```

(For tests, building a fresh theme via the registry is fine; it's O(µs) and doesn't need test isolation.)

- [ ] **Step 4: Run tests**

Run: `cargo test --all`
Expected: suite green (all call sites fixed).

- [ ] **Step 5: Commit**

```bash
git add src/app src/lib.rs tests
git commit -m "feat(app): thread resolved Theme through App at startup"
```

---

## Task 7: Colorize `src/ui/top_bar.rs`

**Files:**
- Modify: `src/ui/top_bar.rs`

- [ ] **Step 1: Read the current `top_bar.rs`**

Run: `cat src/ui/top_bar.rs`
Expected: shows the existing `render` fn with 5 hardcoded `Color::Cyan` / `Color::Yellow` / `Color::Gray` / `Color::Red` calls.

- [ ] **Step 2: Update the render signature to take `&Theme`**

The render fn currently signs as something like `pub fn render(f: &mut Frame, area: Rect, justfile: &str, errors: usize)`. Add `theme: &crate::theme::Theme` as the final parameter. Replace each hardcoded color:

| Old | New |
|---|---|
| `Color::Cyan` (lazyjust label) | `theme.accent` |
| `Color::Yellow` (justfile path) | `theme.info` |
| `Color::Gray` (`?` / `q` hints) | `theme.dim` |
| `Color::Red` (error badge) | `theme.error` |

Also: remove the `use ratatui::style::Color;` line if it becomes unused — the compiler flags this as warning and clippy at `-D warnings` fails if it's left in.

- [ ] **Step 3: Update caller**

Find who calls `top_bar::render`. Likely `src/ui/mod.rs` or `src/ui/layout.rs`. Pass `&app.theme` as the new last argument.

Run: `cargo build`
Expected: builds clean.

- [ ] **Step 4: Run tests and refresh snapshots**

Run: `cargo test --all`
Expected: some snapshot tests touching the top bar will fail with a diff (new colors). Use `cargo insta review` or `cargo insta accept` to update.

Run: `cargo test --all`
Expected: suite green on second run.

- [ ] **Step 5: Commit**

```bash
git add src/ui/top_bar.rs src/ui tests
git commit -m "feat(ui): route top_bar colors through Theme"
```

---

## Task 8: Colorize `src/ui/list.rs`

**Files:**
- Modify: `src/ui/list.rs`

- [ ] **Step 1: Update list render**

The current `list.rs` has:

- `Color::DarkGray` as selection bg → `theme.highlight` (with `.fg(theme.selected_fg)`)
- `Color::Magenta` on group header → `theme.accent`
- `Color::DarkGray` on ungrouped header → `theme.dim`
- `Color::Blue` for running (●) → `theme.running`
- `Color::Green` / `Color::DarkGray` (success ✓, read/unread) → `theme.success` / `theme.dim`
- `Color::Red` / `Color::DarkGray` (failure ✗) → `theme.error` / `theme.dim`
- `Color::Yellow` (broken !) → `theme.warn`

Add `theme: &crate::theme::Theme` to the render signature. Thread it into the status-dot color helper.

- [ ] **Step 2: Remove unused `use ratatui::style::Color;` if any**

Compiler flags; delete if unused.

- [ ] **Step 3: Update caller**

Pass `&app.theme` to `list::render` wherever it is called.

- [ ] **Step 4: Build + test + refresh snapshots**

```bash
cargo build
cargo test --all    # expect snapshot diffs
cargo insta accept  # or review
cargo test --all    # green on second run
```

- [ ] **Step 5: Commit**

```bash
git add src/ui tests
git commit -m "feat(ui): route list colors + status dots through Theme"
```

---

## Task 9: Colorize `src/ui/preview.rs`

**Files:**
- Modify: `src/ui/preview.rs`

- [ ] **Step 1: Swap colors**

| Old | New |
|---|---|
| `Color::Yellow` (recipe: heading) | `theme.accent` |
| `Color::Gray` (doc line) | `theme.dim` |
| `Color::Cyan` (Enter to run hint) | `theme.success` |

Add `theme: &crate::theme::Theme` param, thread from caller.

- [ ] **Step 2: Build + snapshots + test**

```bash
cargo build
cargo test --all
cargo insta accept
cargo test --all
```

- [ ] **Step 3: Commit**

```bash
git add src/ui tests
git commit -m "feat(ui): route preview colors through Theme"
```

---

## Task 10: Colorize `src/ui/focus.rs`

**Files:**
- Modify: `src/ui/focus.rs`

- [ ] **Step 1: Swap colors**

| Old | New |
|---|---|
| `Color::Cyan` (active border) | `theme.accent` |
| `Color::DarkGray` (inactive border) | `theme.dim` |
| `Color::Black` (title fg on active) | `theme.bg` |
| `Color::Cyan` (title bg on active) | `theme.accent` |
| `Color::Gray` (inactive title) | `theme.dim` |

Add `theme: &crate::theme::Theme` param(s) as needed.

- [ ] **Step 2: Update callers**

The focus helpers are used in multiple panes — grep:

```bash
rg "focus::" src/ui/
```

Thread `&app.theme` through each call.

- [ ] **Step 3: Build + snapshots + test**

```bash
cargo build
cargo test --all
cargo insta accept
cargo test --all
```

- [ ] **Step 4: Commit**

```bash
git add src/ui tests
git commit -m "feat(ui): route focus border colors through Theme"
```

---

## Task 11: Colorize modals (`modal.rs`, `param_modal.rs`, `help.rs`)

**Files:**
- Modify: `src/ui/modal.rs`
- Modify: `src/ui/param_modal.rs`
- Modify: `src/ui/help.rs`

- [ ] **Step 1: Swap colors in `src/ui/modal.rs`**

| Old | New |
|---|---|
| `Color::DarkGray` (modal bg / backdrop) | `theme.bg` |
| `Color::Yellow` (title) | `theme.accent` |
| `Color::Gray` (subtitle / hint) | `theme.dim` |

Add `theme: &crate::theme::Theme` param.

- [ ] **Step 2: Swap in `src/ui/param_modal.rs`**

| Old | New |
|---|---|
| `Color::Gray` (hint) | `theme.dim` |

- [ ] **Step 3: Swap in `src/ui/help.rs`**

| Old | New |
|---|---|
| `Color::Cyan` (active section / border accent) | `theme.accent` |
| `Color::Yellow` (section header) | `theme.fg` (bold via existing modifier) |

- [ ] **Step 4: Update callers**

Thread `&app.theme` to each modal render fn.

- [ ] **Step 5: Build + snapshots + test**

```bash
cargo build
cargo test --all
cargo insta accept
cargo test --all
```

- [ ] **Step 6: Commit**

```bash
git add src/ui tests
git commit -m "feat(ui): route modal / help / param colors through Theme"
```

---

## Task 12: Audit remaining UI files for hardcoded colors

**Files:**
- Modify: any remaining `src/ui/*.rs` flagged by the grep

- [ ] **Step 1: Grep**

```bash
rg -n "Color::" src/ui/ | grep -v "src/ui/session_pane.rs"
```

Expected: the only remaining matches should be in `session_pane.rs` (vt100 palette mapping — intentionally not themed) plus `src/ui/status_bar.rs` (if any).

- [ ] **Step 2: Fix status_bar.rs if needed**

If `src/ui/status_bar.rs` uses hardcoded colors, add `theme` parameter and swap to `theme.dim` / `theme.warn` / `theme.error` as appropriate based on context.

- [ ] **Step 3: Confirm session_pane.rs chrome is not touched**

`session_pane.rs:60-77` maps vt100 palette indices to `ratatui::Color::X`. These are correct — they translate terminal output colors, not app chrome. Leave them as-is.

Any other hardcoded colors in `session_pane.rs` (chrome, focus border) should swap to theme slots; run the grep above to be sure.

- [ ] **Step 4: Build + snapshots + test**

```bash
cargo build
cargo test --all
cargo insta accept
cargo test --all
```

- [ ] **Step 5: Commit**

```bash
git add src/ui tests
git commit -m "feat(ui): final theme wiring across status_bar / any remaining chrome"
```

---

## Task 13: Integration test — theme registry under env override

**Files:**
- Create: `tests/theme_registry.rs`

- [ ] **Step 1: Write the test**

```rust
use lazyjust::config::paths::OVERRIDE_ENV;
use lazyjust::theme::registry;
use std::sync::{Mutex, OnceLock};

fn guard() -> &'static Mutex<()> {
    static M: OnceLock<Mutex<()>> = OnceLock::new();
    M.get_or_init(|| Mutex::new(()))
}

#[test]
fn user_theme_shadows_builtin_via_registry() {
    let _lock = guard().lock().unwrap_or_else(|e| e.into_inner());
    let tmp = tempfile::tempdir().unwrap();
    let themes = tmp.path().join("themes");
    std::fs::create_dir_all(&themes).unwrap();
    std::fs::write(
        themes.join("tokyo-night.toml"),
        r#"
            name = "Shadowed Tokyo"
            bg = "#000001"
            fg = "#ffffff"
            dim = "gray"
            accent = "cyan"
            highlight = "dark_gray"
            selected_fg = "white"
            success = "green"
            warn = "yellow"
            error = "red"
            running = "blue"
            info = "cyan"
            badge_bg = "dark_gray"
            badge_fg = "white"
        "#,
    )
    .unwrap();

    std::env::set_var(OVERRIDE_ENV, tmp.path());
    let t = registry::resolve("tokyo-night");
    std::env::remove_var(OVERRIDE_ENV);

    assert_eq!(t.name, "Shadowed Tokyo");
}

#[test]
fn list_includes_user_theme_via_registry() {
    let _lock = guard().lock().unwrap_or_else(|e| e.into_inner());
    let tmp = tempfile::tempdir().unwrap();
    let themes = tmp.path().join("themes");
    std::fs::create_dir_all(&themes).unwrap();
    std::fs::write(
        themes.join("solarpunk.toml"),
        r#"
            name = "Solarpunk"
            bg = "#000001"
            fg = "#ffffff"
            dim = "gray"
            accent = "cyan"
            highlight = "dark_gray"
            selected_fg = "white"
            success = "green"
            warn = "yellow"
            error = "red"
            running = "blue"
            info = "cyan"
            badge_bg = "dark_gray"
            badge_fg = "white"
        "#,
    )
    .unwrap();

    std::env::set_var(OVERRIDE_ENV, tmp.path());
    let names = registry::list();
    std::env::remove_var(OVERRIDE_ENV);

    assert!(names.iter().any(|n| n == "solarpunk"));
    assert!(names.iter().any(|n| n == "tokyo-night"));
}
```

- [ ] **Step 2: Run**

Run: `cargo test --test theme_registry`
Expected: 2 tests pass.

- [ ] **Step 3: Commit**

```bash
git add tests/theme_registry.rs
git commit -m "test(theme): registry shadowing via user themes dir"
```

---

## Task 14: Theme picker — data model + actions

**Files:**
- Modify: `src/app/types.rs`
- Modify: `src/app/action.rs`
- Modify: `src/app/reducer.rs`

- [ ] **Step 1: Add `Mode::ThemePicker` variant in `src/app/types.rs`**

Inside the `pub enum Mode { ... }` block, add (before the closing `}` and after `ErrorsList,`):

```rust
    ThemePicker {
        original_name: String,
        highlighted: usize,
        names: Vec<String>,
    },
```

- [ ] **Step 2: Add actions in `src/app/action.rs`**

Append to the existing `pub enum Action { ... }`:

```rust
    OpenThemePicker,
    PickerMove(isize),      // +1 next, -1 prev
    PickerConfirm,
    PickerCancel,
```

- [ ] **Step 3: Handle actions in `src/app/reducer.rs`**

Add new arms in the match statement. Pseudocode — fit into the existing pattern:

```rust
Action::OpenThemePicker => {
    let names = crate::theme::registry::list();
    let current = app.theme.name.clone(); // Theme::name is the display name
    // Find index of current theme by display name — but `list()` returns
    // file-stem names, not display names. Use the theme_name we stored in
    // config mirror: we need to thread Config.theme_name. Simplest: store
    // the resolved file-stem name alongside the theme.
    let original_name = app.theme_name.clone();
    let highlighted = names.iter().position(|n| *n == original_name).unwrap_or(0);
    app.mode = Mode::ThemePicker { original_name, highlighted, names };
}
Action::PickerMove(delta) => {
    if let Mode::ThemePicker { highlighted, names, .. } = &mut app.mode {
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
Action::PickerConfirm => {
    if let Mode::ThemePicker { .. } = app.mode {
        let stem = app.theme_name.clone();
        let path = crate::config::paths::config_file_path();
        if let Err(e) = crate::config::writer::set_theme(&path, &stem) {
            tracing::warn!(target: "lazyjust::theme", error = %e, "failed to persist theme");
            app.status_message = Some(format!("theme persist failed: {e}"));
        }
        app.mode = Mode::Normal;
    }
}
Action::PickerCancel => {
    if let Mode::ThemePicker { original_name, .. } = &app.mode {
        let original = original_name.clone();
        app.theme = crate::theme::registry::resolve(&original);
        app.theme_name = original;
        app.mode = Mode::Normal;
    }
}
```

Note: `App.theme_name` already exists (added in Task 6). No signature change here — just update the field during picker actions.

- [ ] **Step 4: Build**

Run: `cargo build`
Expected: clean build. Fix any call-site signature mismatches for `App::new`.

- [ ] **Step 5: Commit**

```bash
git add src/app
git commit -m "feat(app): Mode::ThemePicker + picker actions with live preview"
```

---

## Task 15: Theme picker — event loop wiring + reducer unit tests

**Files:**
- Modify: `src/app/event_loop.rs`
- Modify: `src/app/reducer.rs` (add unit tests)

- [ ] **Step 1: Wire the `t` key**

In `src/app/event_loop.rs`, find the `Mode::Normal` key handler block. Add a case for `KeyCode::Char('t')`:

```rust
KeyCode::Char('t') => dispatch(Action::OpenThemePicker),
```

Then add a `Mode::ThemePicker` handler that treats the mode as modal:

```rust
Mode::ThemePicker { .. } => match key.code {
    KeyCode::Char('j') | KeyCode::Down => dispatch(Action::PickerMove(1)),
    KeyCode::Char('k') | KeyCode::Up => dispatch(Action::PickerMove(-1)),
    KeyCode::Enter => dispatch(Action::PickerConfirm),
    KeyCode::Esc => dispatch(Action::PickerCancel),
    _ => {}
},
```

(Keep the existing patterns for dispatch used elsewhere in the file — `dispatch` here is illustrative; match to the real pattern.)

- [ ] **Step 2: Add reducer tests in `src/app/reducer.rs`**

```rust
#[cfg(test)]
mod theme_picker_tests {
    use super::*;
    use crate::app::action::Action;
    use crate::app::types::Mode;

    fn test_app() -> App {
        App::new(
            vec![],
            vec![],
            0.3,
            crate::theme::registry::resolve(crate::theme::DEFAULT_THEME_NAME),
            crate::theme::DEFAULT_THEME_NAME.to_string(),
        )
    }

    #[test]
    fn open_picker_enters_mode_with_current_theme_highlighted() {
        let mut app = test_app();
        apply(&mut app, Action::OpenThemePicker);
        match &app.mode {
            Mode::ThemePicker { original_name, highlighted, names } => {
                assert_eq!(original_name, "tokyo-night");
                assert_eq!(names[*highlighted], "tokyo-night");
            }
            _ => panic!("expected ThemePicker mode"),
        }
    }

    #[test]
    fn picker_move_wraps_around() {
        let mut app = test_app();
        apply(&mut app, Action::OpenThemePicker);
        // back from first -> last
        apply(&mut app, Action::PickerMove(-1));
        let last_name = match &app.mode {
            Mode::ThemePicker { names, highlighted, .. } => names[*highlighted].clone(),
            _ => panic!(),
        };
        assert_eq!(app.theme_name, last_name);
        assert_ne!(app.theme_name, "tokyo-night");
    }

    #[test]
    fn picker_cancel_restores_original() {
        let mut app = test_app();
        apply(&mut app, Action::OpenThemePicker);
        apply(&mut app, Action::PickerMove(1));
        assert_ne!(app.theme_name, "tokyo-night");
        apply(&mut app, Action::PickerCancel);
        assert_eq!(app.theme_name, "tokyo-night");
        assert!(matches!(app.mode, Mode::Normal));
    }
}
```

(Adjust `apply` / the reducer entry point to match the actual function name — grep the file.)

- [ ] **Step 3: Run tests**

Run: `cargo test --lib app::reducer::theme_picker_tests`
Expected: 3 tests pass.

Run: `cargo test --all`
Expected: full suite green.

- [ ] **Step 4: Commit**

```bash
git add src/app
git commit -m "feat(app): bind t/j/k/Enter/Esc to theme picker actions"
```

---

## Task 16: `set_theme` writer — toml_edit round-trip

**Files:**
- Create: `src/config/writer.rs`
- Modify: `src/config.rs`

- [ ] **Step 1: Create `src/config/writer.rs`**

```rust
//! Comment-preserving writes to the user config file. Used by the
//! theme picker to persist `[ui].theme` without blowing away the rest
//! of the file.

use std::path::Path;
use toml_edit::{value, DocumentMut};

#[derive(Debug, thiserror::Error)]
pub enum WriterError {
    #[error("config file IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("config file parse error: {0}")]
    Parse(#[from] toml_edit::TomlError),
}

pub fn set_theme(path: &Path, name: &str) -> Result<(), WriterError> {
    let mut doc: DocumentMut = match std::fs::read_to_string(path) {
        Ok(s) => s.parse()?,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => DocumentMut::new(),
        Err(e) => return Err(WriterError::Io(e)),
    };

    // Ensure [ui] table exists.
    if !doc.contains_key("ui") {
        let mut t = toml_edit::Table::new();
        t.set_implicit(false);
        doc["ui"] = toml_edit::Item::Table(t);
    }

    doc["ui"]["theme"] = value(name);

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, doc.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_fresh_file_when_missing() {
        let tmp = tempfile::tempdir().unwrap();
        let p = tmp.path().join("config.toml");
        set_theme(&p, "gruvbox-dark").unwrap();
        let s = std::fs::read_to_string(&p).unwrap();
        assert!(s.contains("theme = \"gruvbox-dark\""));
        assert!(s.contains("[ui]"));
    }

    #[test]
    fn preserves_comments_and_other_keys() {
        let tmp = tempfile::tempdir().unwrap();
        let p = tmp.path().join("config.toml");
        let original = r#"# top-level comment
[ui]
# keep this comment
theme = "tokyo-night"
split_ratio = 0.30

[engine]
render_throttle_ms = 16
"#;
        std::fs::write(&p, original).unwrap();
        set_theme(&p, "dracula").unwrap();
        let s = std::fs::read_to_string(&p).unwrap();
        assert!(s.contains("# top-level comment"));
        assert!(s.contains("# keep this comment"));
        assert!(s.contains("theme = \"dracula\""));
        assert!(s.contains("split_ratio = 0.3"));
        assert!(s.contains("render_throttle_ms = 16"));
    }

    #[test]
    fn adds_ui_section_if_missing() {
        let tmp = tempfile::tempdir().unwrap();
        let p = tmp.path().join("config.toml");
        std::fs::write(&p, "[engine]\nrender_throttle_ms = 8\n").unwrap();
        set_theme(&p, "nord").unwrap();
        let s = std::fs::read_to_string(&p).unwrap();
        assert!(s.contains("[ui]"));
        assert!(s.contains("theme = \"nord\""));
        assert!(s.contains("render_throttle_ms = 8"));
    }
}
```

- [ ] **Step 2: Register the submodule in `src/config.rs`**

Add (keeping existing declarations):

```rust
pub mod writer;
```

- [ ] **Step 3: Run tests**

Run: `cargo test --lib config::writer`
Expected: 3 tests pass.

Run: `cargo test --all`
Expected: full suite green.

- [ ] **Step 4: Commit**

```bash
git add src/config src/config/writer.rs
git commit -m "feat(config): toml_edit writer — set_theme preserves comments"
```

---

## Task 17: Theme picker modal UI

**Files:**
- Create: `src/ui/theme_picker.rs`
- Modify: `src/ui/mod.rs`

- [ ] **Step 1: Create the renderer**

`src/ui/theme_picker.rs`:

```rust
use crate::app::state::App;
use crate::app::types::Mode;
use crate::theme::Theme;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    prelude::*,
    style::{Modifier, Style},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let (names, highlighted) = match &app.mode {
        Mode::ThemePicker {
            names, highlighted, ..
        } => (names, *highlighted),
        _ => return,
    };

    let theme: &Theme = &app.theme;

    let outer = centered(area, 40, 60);
    f.render_widget(Clear, outer);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.accent))
        .title(Span::styled(
            " Theme ",
            Style::default()
                .fg(theme.fg)
                .add_modifier(Modifier::BOLD),
        ))
        .title_alignment(Alignment::Center)
        .style(Style::default().bg(theme.bg).fg(theme.fg));
    let inner = block.inner(outer);
    f.render_widget(block, outer);

    let [list_area, hint_area] = *Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(inner)
    else {
        return;
    };

    let items: Vec<ListItem> = names
        .iter()
        .map(|n| ListItem::new(Span::raw(n.clone())))
        .collect();

    let list = List::new(items)
        .highlight_style(
            Style::default()
                .bg(theme.highlight)
                .fg(theme.selected_fg)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▸ ");

    let mut state = ListState::default();
    state.select(Some(highlighted));
    f.render_stateful_widget(list, list_area, &mut state);

    let hint = Paragraph::new(Span::styled(
        "j/k select · Enter apply & save · Esc revert",
        Style::default().fg(theme.dim),
    ))
    .alignment(Alignment::Center);
    f.render_widget(hint, hint_area);
}

fn centered(area: Rect, min_w: u16, min_h: u16) -> Rect {
    let w = min_w.min(area.width.saturating_sub(4));
    let h = min_h.min(area.height.saturating_sub(4));
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    Rect { x, y, width: w, height: h }
}
```

- [ ] **Step 2: Dispatch from the top-level UI renderer**

In `src/ui/mod.rs`, find where existing modals render (e.g. `Mode::Help`, `Mode::Confirm`). Add an arm:

```rust
Mode::ThemePicker { .. } => theme_picker::render(f, area, app),
```

And declare the module at the top of `src/ui/mod.rs`:

```rust
pub mod theme_picker;
```

- [ ] **Step 3: Build + snapshot**

Run: `cargo build`
Expected: builds.

Run: `cargo test --all`
Expected: green. No snapshot covers the picker yet; we can add one in the next step but it's optional for now.

- [ ] **Step 4: Add a snapshot test**

Append to an existing snapshot test module (e.g. `tests/snapshots.rs` or the module that holds modal snapshots) a test that opens the picker and snapshots the buffer. Mirror the existing modal-snapshot style — do not reinvent the harness.

If no existing snapshot file applies cleanly, skip this step; reducer tests from Task 15 already exercise the behavior.

- [ ] **Step 5: Commit**

```bash
git add src/ui tests
git commit -m "feat(ui): theme picker modal with live preview hints"
```

---

## Task 18: CI color-leak gate

**Files:**
- Modify: `justfile`

- [ ] **Step 1: Add a `color-gate` recipe**

Open `justfile` and append:

```just
# Fail if any hardcoded ratatui Color::X sneaks into src/ui/ (outside
# session_pane.rs, which translates vt100 palette indices). Theme slots
# are the only colorization path for UI chrome.
color-gate:
    @ rg --glob '!src/ui/session_pane.rs' 'Color::(Red|Green|Blue|Yellow|Cyan|Magenta|White|Black|LightRed|LightGreen|LightBlue|LightYellow|LightCyan|LightMagenta|LightGray|DarkGray|Gray|Rgb)' src/ui/ && exit 1 || exit 0
```

- [ ] **Step 2: Wire into default `check`**

In the same `justfile`, find the `check` or `ci` recipe. Add `color-gate` to it (after tests, before fmt). Example, if `check` currently reads `just fmt lint test`:

```just
check: fmt lint test color-gate
```

Do not add a new dependency — call `color-gate` as a sibling step in the existing pipeline.

- [ ] **Step 3: Run**

Run: `just color-gate`
Expected: exits 0 (no leaks).

Sanity check by briefly introducing a leak:

```bash
echo "_ = ratatui::style::Color::Red;" >> src/ui/top_bar.rs
just color-gate; echo "exit: $?"
# Undo
git checkout -- src/ui/top_bar.rs
```

Expected: first run exits 1.

- [ ] **Step 4: Commit**

```bash
git add justfile
git commit -m "ci: color-gate blocks hardcoded Color::X in src/ui/ chrome"
```

---

## Task 19: Manual verification

Not a code task. Exercise the end-to-end flow before finishing the branch.

- [ ] **Step 1: Clean state**

```bash
rm -f "$(cargo run --quiet -- config path)"
```

- [ ] **Step 2: Launch with default theme**

Run: `cargo run`
Expected: UI launches using Tokyo Night colors (blue-violet accents).

- [ ] **Step 3: Open picker**

Press `t`.
Expected: modal appears centered, lists all 11 built-ins, highlighted row is `tokyo-night`.

- [ ] **Step 4: Live preview**

Press `j` a few times.
Expected: UI behind the modal recolors in real time as each theme becomes highlighted.

- [ ] **Step 5: Cancel**

Press `Esc`.
Expected: modal closes, UI reverts to Tokyo Night.

- [ ] **Step 6: Persist**

Press `t`, `j` until `gruvbox-dark` highlights, `Enter`.
Expected: modal closes, UI stays Gruvbox, `cat "$(cargo run --quiet -- config path)"` shows `[ui] theme = "gruvbox-dark"`.

- [ ] **Step 7: Restart persists**

Quit and relaunch `cargo run`.
Expected: opens straight into Gruvbox without needing to pick.

- [ ] **Step 8: User theme shadows built-in**

```bash
mkdir -p "$(dirname "$(cargo run --quiet -- config path)")/themes"
cat > "$(dirname "$(cargo run --quiet -- config path)")/themes/gruvbox-dark.toml" <<'EOF'
name         = "Shadowed Gruvbox"
bg           = "#2a0000"
fg           = "#ffdd88"
dim          = "#664400"
accent       = "#ff6622"
highlight    = "#552200"
selected_fg  = "#ffffff"
success      = "#44dd44"
warn         = "#ffaa22"
error        = "#ff3322"
running      = "#ff6622"
info         = "#ffdd88"
badge_bg     = "#552200"
badge_fg     = "#ffdd88"
EOF
cargo run
```

Expected: UI renders with the shadowed palette (dark red bg), not the original Gruvbox.

- [ ] **Step 9: Corrupt user theme, fallback works**

```bash
echo "broken = = =" > "$(dirname "$(cargo run --quiet -- config path)")/themes/gruvbox-dark.toml"
cargo run
```

Expected: UI launches with the built-in Gruvbox (fallback), tracing warn in the log.

- [ ] **Step 10: Clean up**

```bash
rm -rf "$(dirname "$(cargo run --quiet -- config path)")"
```

---

## Exit criteria for Milestone 2

All checkboxes above ticked.

- `cargo test --all` — green. Expect ~95 tests (67 from M1 + ~28 new).
- `cargo clippy --all-targets --all-features -- -D warnings` — clean.
- `cargo fmt --all -- --check` — clean.
- `just color-gate` — exits 0.
- Binary ships an in-app theme picker. Picking a theme updates `[ui].theme` in the user config while preserving comments and other keys.
- User-defined theme files under `<config_dir>/lazyjust/themes/` shadow built-ins; malformed user themes log a warning and fall back.
- No layout change, no glyph change — M3 territory.
