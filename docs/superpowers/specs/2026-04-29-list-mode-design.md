# list_mode: merged recipe list across discovered justfiles

## Problem

Today `lazyjust` walks the target directory recursively, picks the root
justfile as the active one, and exposes the rest through a dropdown opened
with `d`. Users with monorepos that contain many small justfiles want to see
every recipe in a single left-pane list — grouped by justfile — without
having to switch the active justfile to inspect or run something nested.

The current dropdown-driven UX stays useful for projects with one primary
justfile and a couple of throwaway ones in subdirectories. The merged view
is useful when several justfiles are roughly equal peers. The setting opts
into the second case.

## Goals

- A config-driven switch between the existing per-justfile view and a merged
  view that lists every recipe across every discovered justfile.
- The setting can be flipped via the config file or a CLI flag for one-off
  runs.
- Run dispatch, filtering, cursor movement, and session handling all work
  in the merged view without code paths assuming a single active justfile.

## Non-goals

- Runtime hotkey to toggle between modes.
- Onboarding hint that nudges users toward the merged mode.
- Two-level grouping (justfile → within-justfile `[group]` → recipes) in the
  merged view. The merged view ignores within-justfile groups.
- Persisting cursor position across runs.

## User-facing surface

### Config

Add a key under the existing `[ui]` section:

```toml
[ui]
list_mode = "active"   # "active" | "all"
```

- `active` (default): existing behavior. Active justfile drives the list;
  other discovered justfiles are reachable through the `d` dropdown.
- `all`: merged view. Recipes from every discovered justfile render in one
  list, grouped by justfile path. The `d` dropdown is disabled — pressing
  `d` shows a transient status message (`dropdown disabled in
  list_mode=all`) and is annotated in the help modal.

`lazyjust config init` writes the new key as a commented line under `[ui]`
with a one-line comment describing the alternatives.

An invalid value (`list_mode = "weird"`) logs a warning at config load and
falls back to `active`. This matches the existing handling of unknown theme
names.

### CLI

Add a flag mirroring `--log-level`:

```
--list-mode <active|all>
```

Parsed as a `clap::ValueEnum`; invalid values are rejected by clap. The flag
overrides the config value for the run.

## Design

### Approach

The merged view is purely a render-time concern. Discovery already returns
every justfile under the target path. The change is a single ordered list
that the renderer, cursor, filter, and run-dispatch all share. The list is
built from the existing `Vec<Justfile>` and rebuilt only when the mode
flips or discovery reloads.

### Types

```rust
// app::types
pub enum ListMode { Active, All }

pub enum RowRef {
    Header { jf_idx: usize },                        // only emitted in All mode
    Recipe { jf_idx: usize, recipe_idx: usize },
}

pub struct ListView { pub rows: Vec<RowRef> }
```

`Config` gains `pub list_mode: ListMode` (resolved from config file then
CLI override). `App` gains `pub list_mode: ListMode` and `pub view:
ListView`.

`active_justfile: usize` is retained — it still drives the header bar and
dropdown when `list_mode = Active`. In `All` mode it is ignored by the
renderer.

### View construction

`ListView::build(&[Justfile], ListMode) -> ListView`:

- `Active`: rows = `[Recipe { active_jf, i } for i in 0..recipes.len()]`. No
  `Header` rows. Existing within-justfile `[group]` separators are still
  rendered inside `ui::list::build_lines` based on `Recipe.group`.
- `All`: justfiles iterated in their existing path-sorted order. For each
  justfile with at least one recipe, emit `Header { jf_idx }` followed by
  `Recipe { jf_idx, i }` rows. Justfiles with zero recipes are skipped (a
  header without rows is noise).

The view is built in `App::new` and rebuilt when mode changes via
`Action::SetListMode(ListMode)`. (No key binding wires this up in the
current scope; the action exists for future toggles and tests.)

### Cursor model

`list_cursor` indexes Recipe rows only. Header rows are skipped during
`j`/`k` movement. `recipe_at_cursor()` walks the visible (filtered) rows,
counts Recipe rows, and returns the Recipe at index `list_cursor` plus the
owning justfile via `jf_idx`.

### Filter

`app::filter::fuzzy_match` runs over the recipe names from the view's
Recipe rows. The filtered output is a subset of Recipe rows. When rendering
the filtered list:

- A `Header { jf_idx }` is emitted only if at least one Recipe row with
  the same `jf_idx` survived the filter.
- Cursor clamps within the filtered Recipe rows.

### Render

`ui::list::render`:

- In `Active` mode the loop is unchanged; group headers come from
  `Recipe.group` transitions as today.
- In `All` mode the loop iterates view rows. `Header { jf_idx }` rows
  render via `section_header` with the path label below; `current_group` is
  reset to `None` at every header swap (within-justfile group separators
  are skipped).
- The header label is the source justfile path made relative to `cli.path`.
  Add a small helper, `relativize_to_root(path, root) -> String`, that
  prefers `path.strip_prefix(root)` and falls back to the absolute path
  when stripping fails.

### Run dispatch

Today `recipe_at_cursor()` returns `&Recipe` and the spawn site uses
`active_justfile().path` for `--justfile`. After this change, the spawn
site reads the source justfile path from the row's `jf_idx`
(`app.justfiles[jf_idx].path`). This works identically in both modes —
`Active` mode rows always carry `jf_idx == active_justfile`, so the result
matches today.

### Reducer

- `Action::SetListMode(ListMode)`: updates `app.list_mode`, calls
  `ListView::build`, resets `list_cursor` to `0`, clears the filter.
- `Action::OpenDropdown` (`d` key): in `Active` mode, today's behavior. In
  `All` mode, no-op plus `app.status_message = Some("dropdown disabled in
  list_mode=all".into())`.
- All other recipe-keyed actions (`Enter`, `Shift+Enter`, `K`, `x`, `L`,
  `h`/`l`, `Ctrl+o`/`Ctrl+i`) operate on the recipe under cursor by way of
  the row's `jf_idx`.

### Help modal

The keybind row for `d` annotates `(disabled in list_mode=all)`.

### Top bar

`ui::top_bar`:

- `Active` mode: unchanged — shows the active justfile's path.
- `All` mode: shows the discovery root and a count, e.g.
  `./ — 4 justfiles, 27 recipes`. Exact wording finalized during
  implementation.

### Onboarding hint

The first-run hint currently advertises `d` when more than one justfile is
discovered. In `All` mode that row is dropped (recipes from every justfile
are already visible). No replacement nudge is added in this scope.

## Errors and edge cases

- Zero justfiles discovered: existing startup-errors modal path; mode is
  irrelevant.
- One justfile in `All` mode: renders a single section header and its
  recipes. No fallback to `Active`.
- Justfile with zero recipes in `All` mode: skipped.
- Filter eliminates every recipe from a justfile: that section header drops
  out.
- Invalid `list_mode` value in config: warn-log, default to `active`.
- Invalid `--list-mode` value: clap rejects.

## Testing

- `config::file` — parse `list_mode = "active"|"all"`, missing key falls
  back to default, invalid value logs warning and falls back. Mirror the
  existing `malformed_file_falls_back_to_defaults_with_warning` pattern.
- `cli` — `--list-mode all` parses; bad value rejected by clap.
- `config::merge` — CLI override beats config file value.
- `app::view::build` — row order for one justfile in `Active`, three
  justfiles in `All`, zero-recipe justfile dropped, justfiles sort by path.
- `app::filter` integration — filtering narrows Recipe rows; section
  headers drop out when no Recipe rows from that `jf_idx` survive.
- Reducer — `Action::OpenDropdown` no-ops and sets a status message when
  mode is `All`; cursor j/k skips Header rows; `Action::SetListMode` resets
  cursor and filter.
- `ui::list::build_lines` — row count and leading header structure per
  mode.
- Run dispatch — recipe under cursor in `All` mode resolves to its source
  justfile path; the path is not pulled from a stale
  `active_justfile.path`.

## Out of scope

- Runtime hotkey to flip mode.
- Two-level grouping in `All` mode (justfile → `[group]` → recipes).
- Onboarding nudge toward `All` mode.
- Persisted cursor across runs.
