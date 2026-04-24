# UI Redesign — Rich Dashboard + Theme System + Config File

**Date:** 2026-04-24
**Status:** Draft — pending user review

## Goal

Make lazyjust more intuitive and more beautiful overall. Ship a richer dashboard-style TUI, a pluggable theme system with 10+ built-ins and user-defined themes, and a real TOML config file that replaces the current hardcoded `Config::load`. Theme switching is live via an in-app picker modal and persists across restarts.

Non-goal: changing core behavior (recipe execution, session lifecycle, filter, navigation). This is a visual + configuration pass, not a behavior change.

## Delivery approach

Three milestones, each ships independently.

### Milestone 1 — Config file infrastructure

**Outcome:** `~/.config/lazyjust/config.toml` (platform-appropriate via `dirs::config_dir()`) is read at startup; all current hardcoded values in `src/config.rs` become defaults; missing file uses defaults; partial file merges with defaults; malformed file logs warning and falls back to defaults. No visible UI change.

**New CLI subcommands:**
- `lazyjust config path` — prints the config file path.
- `lazyjust config init` — writes a commented example config to the config path if none exists.

**File layout**

```
<config_dir>/lazyjust/config.toml
<config_dir>/lazyjust/themes/*.toml
```

`<config_dir>` resolves via `dirs::config_dir()`:
- Linux: `$XDG_CONFIG_HOME` or `~/.config`
- macOS: `~/Library/Application Support` (via `dirs` crate default)
- Windows: `%APPDATA%`

**`config.toml` shape** (all keys optional; missing → defaults):

```toml
[ui]
theme = "tokyo-night"        # built-in name or filename stem under themes/
split_ratio = 0.30
show_inline_deps = true      # render "→ dep1, dep2" under recipe name
icon_style = "round"         # "round" | "ascii" | "none"

[keys]
quit           = "q"
filter         = "/"
clear_filter   = "esc"
help           = "?"
theme_picker   = "t"
run            = "enter"
move_down      = "j"
move_up        = "k"
page_down      = "pgdn"
page_up        = "pgup"
focus_list     = "left"
focus_right    = "right"
errors_list    = "e"
# Keys accept: single characters ("q"), named keys ("enter", "esc", "tab",
# "pgup", "pgdn", "home", "end", "up", "down", "left", "right", "space"),
# and ctrl-/alt-prefixed chords ("ctrl+c", "alt+t"). Case-insensitive.

[paths]
state_dir = "~/Library/Application Support/lazyjust"   # optional override
sessions_log_dir = "~/Library/Application Support/lazyjust/sessions"

[logging]
session_log_size_cap_mb = 10
session_log_retention_days = 7

[engine]
render_throttle_ms = 16
tick_interval_ms = 250
```

**Module split**

- `src/config/mod.rs` — public `Config` (existing shape preserved) + `Config::load()`.
- `src/config/file.rs` — serde `ConfigFile` struct (all fields `Option<T>`), TOML IO.
- `src/config/defaults.rs` — today's hardcoded values exposed as `fn defaults() -> Config`.
- `src/config/merge.rs` — `merge(ConfigFile, Config) -> Config` filling missing from defaults.

**Behavior rules**

- Missing file → all defaults. Do not create a file on first run; `config init` is opt-in.
- Parse error → log warning via tracing, use defaults, continue startup.
- In-app theme switch writes back only the `[ui].theme` value using `toml_edit::DocumentMut` to preserve comments, spacing, and ordering of the rest of the file.
- When writing to a file that doesn't exist yet (because user hasn't run `config init`), write a minimal new file containing only `[ui]\ntheme = "<name>"`.

**Dependencies added**

- `toml_edit = "0.22"` — comment-preserving round-trip writes.
- `serde`, `serde_derive` — likely already present; verify during implementation.
- `dirs` — already present.

**Tests**

- Default load (no file exists) produces `Config` matching current hardcoded values.
- Partial file (only `[ui].theme` set) merges with defaults for everything else.
- Malformed TOML → defaults returned, warning logged (captured in test via `tracing_test`).
- `toml_edit` round-trip: given a file with comments and a custom section order, writing `[ui].theme` preserves both.
- Platform path resolution via `XDG_CONFIG_HOME` override env var for Linux test; no integration test for macOS/Windows paths beyond manual verification.

### Milestone 2 — Theme system

**Outcome:** lazyjust loads named themes (built-in or user-supplied), every rendering call uses semantic theme slots instead of hardcoded colors, and a new theme picker modal (key `t` in normal mode) lets the user browse, live-preview, and persist theme choice.

**Color input forms** (user writes any of these in theme TOML):

- `"#89b4fa"` → `ratatui::style::Color::Rgb(137, 180, 250)`
- `"blue"`, `"bright_red"`, `"dark_gray"` → named ratatui colors
- `21` → `Color::Indexed(21)` (256-color palette)

**`Theme` struct — semantic slots only**

```rust
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

All slots required on load; missing slot → parse error, theme unloadable, fallback engaged.

**Theme file format** (`<config_dir>/lazyjust/themes/ocean.toml`)

```toml
name = "Ocean"
bg           = "#0a1628"
fg           = "#d7e3f4"
dim          = "#5a7290"
accent       = "#5ab0ff"
highlight    = "#14324a"
selected_fg  = "#ffffff"
success      = "#7dd3a0"
warn         = "#ffd26f"
error        = "#ff7b86"
running      = "#5ab0ff"
info         = "#8ab4ff"
badge_bg     = "#14324a"
badge_fg     = "#d7e3f4"
```

**Built-in themes** (shipped via `include_str!` so binary stands alone):

- `catppuccin-latte`, `catppuccin-frappe`, `catppuccin-macchiato`, `catppuccin-mocha`
- `tokyo-night` (default)
- `gruvbox-dark`
- `dracula`
- `nord`
- `solarized-dark`
- `one-dark`
- `mono-amber`

**Default:** `tokyo-night`. Chosen for high contrast on common dark terminals, broad popularity, and neutral-enough color mapping that `accent` stands in for our current cyan.

**Resolution order** (lookup by name from `[ui].theme`):

1. `<config_dir>/lazyjust/themes/<name>.toml` — users can shadow built-in names.
2. Built-in registry.
3. Missing / malformed → log warning, fall back to `tokyo-night`. Startup never fails on theme error.

**Module layout**

- `src/theme/mod.rs` — `Theme`, `load_theme(name)`, `list_themes()`.
- `src/theme/builtin.rs` — `const BUILTIN_THEMES: &[(&str, &str)]` mapping name → raw TOML string.
- `src/theme/parse.rs` — serde `ThemeFile` + `parse_color` for hex / named / indexed.
- `src/theme/registry.rs` — merges built-ins with user dir, lists unique names.

**Wiring into UI**

- `Theme` stored on `App` alongside `Config`.
- Every call site in `src/ui/*` that uses `Color::X` directly is rewritten to use `theme.<slot>`.
- CI gate (documented in testing section): `rg 'Color::(Red|Green|Blue|Yellow|Cyan|Magenta|White|Black|LightRed|LightGreen|LightBlue|LightYellow|LightCyan|LightMagenta|LightGray|DarkGray|Gray|Rgb)' src/ui/` must return zero matches after M3.

**Theme picker modal**

- New mode variant added next to existing `Filter`, `Help`, `Confirm`, `Dropdown`, `ParamInput`, `ErrorsList`. Name: `Mode::ThemePicker { original: String, highlighted: usize }`.
- Opened by the configured `theme_picker` key (`t` default) in normal mode.
- Modal lists all resolved theme names (user dir + built-ins, deduped, alphabetical within each group).
- `j`/`k` or arrows move the highlight. On highlight change, `App.theme` is replaced with the highlighted theme so the main UI behind the modal re-renders in that theme (live preview).
- `Enter` confirms: writes `[ui].theme = "<name>"` via `toml_edit` and closes modal. Theme stays applied.
- `Esc` cancels: restores `App.theme` to `original`, closes modal, no write.

**Keybinding conflict rule**

If the user remaps `theme_picker` to a different key in `[keys]`, use that. If the user maps another action to `t`, honor the user's mapping; the picker is still reachable via its configured key (which the user set). Conflicts between user-configured keys (two actions on one key) are validated at load time: warn via tracing, keep the first mapping defined, ignore the second. Command-palette access is a separate future spec.

**Tests**

- `parse_color` round-trip: hex, named, indexed.
- Each built-in theme parses successfully and fills every slot.
- User theme at `<dir>/themes/nord.toml` shadows built-in `nord`.
- Missing slot in a theme TOML → typed parse error, caller falls back without crash.
- Registry listing: built-ins + user themes, deduped, stable order.
- Picker applies preview without mutating config until Enter.
- Picker writes only `[ui].theme` on Enter, other keys and comments preserved.

### Milestone 3 — UI redesign

**Outcome:** the TUI renders in the rich-dashboard style described below, using theme slots for every color. Session output pane gets a header strip and scroll indicator. No behavior changes; only rendering.

**Overall layout**

Unchanged three-row structure (top bar, split body, status bar). Min sizes tightened to let each pane breathe: min left pane 28 cols, min right pane 48 cols, floor unchanged.

**Top bar**

```
▌ lazyjust  · justfile ·  7 recipes                              main
```

- `▌` in `theme.accent` (one-col bleed).
- `lazyjust` in `theme.fg` bold.
- Separators and counts in `theme.dim`.
- Right-side breadcrumb pill: justfile's parent directory name in `theme.badge_fg` on `theme.badge_bg`.
- Load errors retain their red-bold badge, restyled to `theme.error` on `theme.bg`.
- `?` / `q` hints move to the status bar (functional reminders, not branding).

**List pane**

```
▍ GENERAL ────────────────────────────
   ○ build
   ○ check   ✓
▶  default  ●  → check
   ○ ci            → fmt · lint · test
   ○ fmt
   ○ lint    ✗
   ○ test    ✓
```

- Section header: `▍ {GROUP}` in `theme.accent`, trailing rule `────` fills row in `theme.dim`.
- Non-selected rows: `   ○ name`. `○` in `theme.dim`.
- Selected row: `▶  name` on `theme.highlight` bg, text in `theme.selected_fg` bold.
- Inline deps: always rendered when a recipe has dependencies, in `theme.dim`, format `→ a · b · c`. Truncate with `…` when row would overflow.
- Session status glyphs retain current semantics, with color from theme slots:
  - `●` running → `theme.running`
  - `✓` success unread → `theme.success`, read → `theme.dim`
  - `✗` failure unread → `theme.error`, read → `theme.dim`
  - `!` broken → `theme.warn`
  - max 3 glyphs; `+N` overflow indicator in `theme.dim`.
- Ungrouped recipes render under header `▍ RECIPES`.
- `icon_style` (from `[ui].icon_style`) selects glyph set:
  - `round` (default): `○` / `●` / `▶`
  - `ascii`: `o` / `*` / `>`
  - `none`: no glyph, indent and color carry selection

**Preview pane** (no active session)

```
default
runs the check recipe

depends on
  ▸ check

command
  $ just check

params
  mode  (default: release)
```

- Recipe name alone on first line, `theme.fg` bold.
- Optional doc in `theme.dim` directly below.
- `depends on` block only when deps exist. `▸` in `theme.success`, dep name in `theme.fg`.
- `command` block: `$` in `theme.info`, command text `theme.fg`. Raw body rendering preserved (no `just --show` invocation).
- `params` block: name in `theme.fg`, `(default: X)` in `theme.dim`.
- Footer hint row removed — hints consolidate in status bar.

**Session output pane** (active session)

Replaces the current unframed vt100 paragraph.

```
▍ default  ●  running · 12s                               pid 48231 · logs ↗
```

- Header strip, 1 row above the vt100 grid.
- Recipe name in `theme.fg` bold; status pill adjacent:
  - Running → `●` in `theme.running`, "running · {elapsed}".
  - Exited code 0 → `✓` in `theme.success`, "done · {elapsed}".
  - Exited non-zero → `✗` in `theme.error`, "exit {code} · {elapsed}".
  - Broken → `!` in `theme.warn`, message.
  - ShellAfterExit → `⌁` in `theme.info`, "shell (exited {code}) · press ^D to close".
- Right side: pid and log-path reference text in `theme.dim`. "logs ↗" is text-only in v1 — no external opener.
- Scroll indicator: 1-col thumb bar on right edge of the vt100 viewport. Track in `theme.dim`, thumb in `theme.accent`. Hidden when content fits. Existing PgUp / PgDn / Home / End bindings unchanged.
- Running indicator is static `●` in v1; animation is a later polish.
- Focus treatment: `▍` bar renders `theme.accent` when the pane is focused, `theme.dim` when not. The old cyan focus border around the pane is removed.

**Status bar**

Consolidates all hints. Content is mode-dependent; all base text in `theme.dim` with active separators.

- Normal: `⏎ run  ·  / filter  ·  t theme  ·  ? help  ·  q quit`
- Filter input: `/text_  ·  Esc cancel  ·  ⏎ apply`
- Param input: `[1/3] mode_  ·  ⏎ next  ·  Esc cancel`
- Theme picker: `j/k select  ·  ⏎ apply & save  ·  Esc revert`
- Help / Errors / Confirm: per-modal hints as today, restyled.
- Status messages (load warnings, errors) appear right-aligned in `theme.warn` / `theme.error`.

**Modals**

Shared base:

```
╭─ Title ────────────────╮
│ body                   │
╰────────────────────────╯
```

- Rounded border in `theme.accent`, title in `theme.fg` bold.
- Body bg `theme.bg`, text `theme.fg`.
- All existing modals (Help, Errors, Confirm, Dropdown, ParamInput) re-rendered with this base. Theme picker uses the same shell.

**"Terminal too small" screen**

Themed. Single centered line: "Terminal too small — need at least {cols}×{rows}." in `theme.dim` on `theme.bg`.

**No pane borders.** Separation comes from section bars and spacing. Keeps the airy feel.

**Module changes**

- `src/ui/top_bar.rs` — rewrite render.
- `src/ui/list.rs` — rewrite for new glyphs, inline deps, section bars.
- `src/ui/preview.rs` — rewrite for structured blocks.
- `src/ui/session_pane.rs` — add header strip, scroll indicator, remove focus border.
- `src/ui/status_bar.rs` — extend for new modes and richer hints.
- `src/ui/modal.rs` (may need new helper module) — shared border/title helpers.
- `src/ui/theme_picker.rs` (new) — picker modal rendering and input handling.

## Testing strategy

**Snapshot tests** via `insta` (existing pattern):

- Each render test runs under two themes: `tokyo-night` and `mono-amber`, catching any hardcoded color.
- Per pane:
  - Top bar: no errors, with errors, long justfile path truncation.
  - List: grouped, ungrouped-only, mixed; short deps, long deps truncated, no deps; all session-status states on same row.
  - Preview: no deps no params, deps only, params only, both; long doc line wrap.
  - Session header: running, exited-0, exited-nonzero, broken, shell-after-exit.
  - Status bar: one per mode (Normal, Filter, ParamInput, Help, Confirm, ThemePicker, ErrorsList).
  - Modals: help, errors, confirm, param, theme picker.
- Layout sizes: 40×10 (floor boundary + 1), 80×24 (medium), 160×50 (wide).
- Icon styles: one list snapshot per `round` / `ascii` / `none`.

**Unit tests**

- Config: default load, partial TOML merge, malformed fallback, `toml_edit` round-trip preserves comments.
- Theme: every color form parses, each built-in loads fully, user theme shadows built-in, missing slot → parse error.
- Theme resolution order: user dir wins, fallback on invalid name.
- Inline dep truncation: deterministic output at each width boundary.
- Session status pill: formatting per `Status` variant and elapsed-time bucketing (s / m / h).

**Integration tests**

- Theme picker flow: open, highlight change previews, Esc reverts with no write, Enter writes `[ui].theme` only.
- First-run with no config: defaults applied, no file created.
- `lazyjust config init` writes commented example; `config path` prints correct platform path.

**CI color-leak gate**

Post-M3, CI grep:

```
rg 'Color::(Red|Green|Blue|Yellow|Cyan|Magenta|White|Black|LightRed|LightGreen|LightBlue|LightYellow|LightCyan|LightMagenta|LightGray|DarkGray|Gray|Rgb)' src/ui/ && exit 1 || exit 0
```

Fails the build if any hardcoded color sneaks back into `src/ui/`.

**Manual verification checklist per milestone**

- M1: delete config → works. Partial config → respected. Broken TOML → fallback + warn. `config init` writes file; `config path` prints it.
- M2: switch through every built-in theme live. User theme in dir appears in picker and selects cleanly. Corrupt user theme → picker lists it, select → fallback + warn, no crash.
- M3: run through each mode, resize to min and below, run recipe with deps, params, long doc, long command, failure path, session complete.

**Non-goals for testing**

- Pixel-perfect match across terminals.
- Mouse input.
- Windows-specific quirks beyond existing CI coverage (Windows gate lives on its own branch).

## Risks

1. **`toml_edit` round-trip on unusual TOML** — multi-line arrays or inline tables could reflow when we touch `[ui].theme`. Mitigate by using `DocumentMut` and touching only the target value node; snapshot-test with a deliberately messy fixture.

2. **Light-theme contrast** — `catppuccin-latte` needs a distinct `highlight` value so selection stays visible on light terminals. Each light-family built-in is manually verified on a light terminal before ship.

3. **Hardcoded color leaks** — easy to miss `Color::Cyan` in a rare branch. CI grep gate described above; snapshot tests run under two very different themes force any leak to show up in diff.

4. **Unicode glyph rendering on legacy Windows terminals** — `▍ ▌ ○ ● ▶ ▸` may render as tofu. Escape hatch is `icon_style = "ascii"`. The broader Windows gate is a separate branch; this spec inherits whatever that work produces.

5. **Picker live-preview churn** — repainting full UI on each `j`/`k` might feel laggy on large recipe lists. Existing `render_throttle_ms = 16` caps it; measure before adding extra throttling.

6. **Scope drift across milestones** — full config + theme system + layout + session polish is a lot. Strict gate: M1 adds no color changes; M2 adds no layout changes; M3 changes no config or theme code.

## Open questions resolved

- Default theme: `tokyo-night`.
- `lazyjust config init`: ships in M1.
- Running spinner: static `●` in v1; animated spinner deferred.
- Keybinding conflicts: user wins; conflicts warn at load, first mapping kept, later ones ignored.
- Command-palette access to actions: deferred to a future spec.
- Log viewer / session history drill-down: deferred to a future spec.
