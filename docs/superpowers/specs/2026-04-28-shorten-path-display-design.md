# Shorten Path Display in TUI

## Problem

Justfile paths shown in the dropdown picker (`src/ui/modal.rs:39`) and the
top bar (`src/ui/top_bar.rs:16`) are rendered as full absolute paths via
`PathBuf::display().to_string()`. The dropdown modal is fixed at 60 columns
wide, so any path longer than ~58 visible chars is silently clipped by
ratatui at the right border, often hiding the most informative tail
(parent dir + filename). On nested workspaces under
`/Users/<user>/projects/...`, every row in the picker can collapse into
indistinguishable prefixes.

## Goals

- Replace `$HOME` prefix with `~` everywhere a justfile path is shown.
- When the resulting display still exceeds the available width, apply a
  segment-aware middle truncation that preserves the root anchor and the
  tail (parent dirs + filename), inserting a single `…` separator.
- Make the dropdown modal width adapt to the terminal so we use available
  space before truncating.
- Keep behavior predictable: same path always shortens to the same string
  given the same width budget. No cwd-relative magic.

## Non-Goals

- Configurable shortening rules or user themes for the ellipsis.
- Cwd-relative path display.
- Any change to the on-disk paths used to invoke `just --justfile <p>`.
  Display shortening is presentation-only; `Justfile.path` stays absolute.
- Status-bar treatment: the status bar does not render a justfile path
  today (it shows the filter prompt). Out of scope.

## Design

### New module: `src/ui/path_display.rs`

Single public function:

```rust
pub fn shorten(path: &Path, max_width: usize) -> String;
```

Behavior, in order:

1. Render the path via `path.display().to_string()`.
2. If the result starts with `$HOME` followed by `/` (or equals `$HOME`),
   replace the prefix with `~`. Read `$HOME` once via
   `std::env::var_os("HOME")`. If `HOME` is unset or empty, skip this step.
3. If `display_width(s) <= max_width`, return `s` unchanged.
4. Otherwise apply segment-aware middle truncation:
   - Split on `/`. Treat the path as `<root>` + middle segments +
     `<filename>`, where `<root>` is `~`, `""` (for an absolute path,
     yielding a leading `/` when re-joined), or the first segment of a
     relative path.
   - Always preserve `<root>` and `<filename>`.
   - Start the candidate as `<root>/…/<filename>`. Greedily insert
     middle segments back in from the right (closest to the filename
     first) one at a time, while the resulting width stays
     `<= max_width`. The form is always
     `<root>/…/<seg_k>/.../<seg_n-1>/<filename>` for some `k`.
   - If even `<root>/…/<filename>` exceeds `max_width`, return that
     string anyway and let ratatui clip — never truncate inside a
     segment, never split the filename.
5. Width is measured by `chars().count()`. Justfile paths are ASCII in
   practice; we deliberately avoid pulling `unicode-width` directly into
   `Cargo.toml` for a presentation helper. If we later see CJK paths in
   the wild we can swap the implementation without changing callers.

Output examples:

| `max_width` | Input                                                       | Output                                |
| ----------- | ----------------------------------------------------------- | ------------------------------------- |
| 56          | `/Users/nick/proj/foo/justfile`                             | `~/proj/foo/justfile`                 |
| 56          | `/Users/nick/projects/entrnce/trader/services/api/justfile` | `~/projects/entrnce/trader/services/api/justfile` |
| 28          | `/Users/nick/projects/entrnce/trader/services/api/justfile` | `~/…/services/api/justfile`           |
| 24          | `/var/very/deeply/nested/repo/sub/dir/justfile`             | `/…/sub/dir/justfile`                 |
| 12          | `/var/very/deeply/nested/repo/sub/dir/justfile`             | `/…/justfile`                         |
| any         | `/justfile`                                                 | `/justfile`                           |

### Integration

**Dropdown** (`src/ui/modal.rs`, `render_dropdown`):

- Replace the fixed `centered(f.area(), 60, 14)` call with an adaptive
  width:

  ```rust
  let w = f.area().width.saturating_sub(4).clamp(40, 100);
  let area = crate::ui::modal_base::centered(f.area(), w, 14);
  ```

- Compute the per-row budget once: `let row_max = (w as usize).saturating_sub(4);`
  (account for left border + space + right border + space).
- Map each indexed justfile via
  `path_display::shorten(&app.justfiles[i].path, row_max)` instead of
  `path.display().to_string()`.

**Top bar** (`src/ui/top_bar.rs`):

- The path span sits in `cols[0]` with the badge in `cols[1]`. Compute
  available width as
  `cols[0].width.saturating_sub(<fixed_chrome>)` where fixed chrome is
  the literal spans rendered around the path
  (`"▌ "`, `"lazyjust"`, `"  · "`, `"  · "`, recipe count, optional
  error pill).
- Cleanest implementation: build the surrounding spans first, sum their
  rendered widths, subtract from `cols[0].width`, clamp to a sensible
  minimum (e.g. 16), pass to `shorten`.
- If sizing helpers grow non-trivial, factor them into a private helper
  `available_path_width(area: Rect, count: usize, has_errors: bool, errors_n: usize) -> usize`
  in `top_bar.rs`.

### Module wiring

- Add `pub mod path_display;` to `src/ui/mod.rs`.
- No public re-export at crate root; callers use
  `crate::ui::path_display::shorten`.

## Tests

Unit tests live in `#[cfg(test)] mod tests` inside `path_display.rs`:

- `shorten_returns_unchanged_when_within_width`
- `shorten_replaces_home_with_tilde`
- `shorten_skips_home_replace_when_home_unset` (use a scoped env override
  via a small helper, or pass a path that doesn't start with `$HOME`).
- `shorten_middle_truncates_long_path` — asserts the result starts with
  the root anchor, contains `…`, and ends with the filename.
- `shorten_keeps_filename_when_budget_too_tight` — extremely small
  `max_width` returns `<root>/…/<filename>` (or just the filename if no
  root segment exists) without panicking.
- `shorten_handles_root_only_path` — `/justfile` → `/justfile`.
- `shorten_handles_relative_path` — input without leading `/` is passed
  through, optionally truncated.

Integration: existing render tests (if any) should not need new
fixtures; if there are no UI snapshot tests, do not introduce a new
testing framework as part of this change.

## Risks & Mitigations

- **Tilde misinterpretation by callers:** No caller uses the displayed
  string as an actual path argument; `Justfile.path` remains absolute
  for `just --justfile`. Display and execution paths stay separate.
- **Width math drift in `top_bar.rs`:** If the surrounding spans change
  in the future, the budget calculation can become stale and cause
  unnecessary truncation or overflow. Mitigation: extract the chrome
  spans into named locals so the width sum and the rendered spans share
  a single source of truth.
- **Modal feels wider than expected on huge terminals:** The 100-col
  cap keeps the dropdown from stretching across an ultrawide screen.

## Rollout

Single PR. No config flag, no migration. Behavior change is purely
visual.
