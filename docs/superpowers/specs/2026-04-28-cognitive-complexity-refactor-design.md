# Cognitive Complexity Refactor — Design

## Goal

Drop SonarCloud's three remaining `rust:S3776` (Cognitive Complexity) findings
below the project threshold of **18**. The refactor is mechanical and changes
no observable behavior.

## Scope

| File | Function | Current | Target | Style |
|---|---|---:|---:|---|
| `src/app/reducer.rs` | `reduce` | 137 | ≤18 | Free functions, same file |
| `src/ui/session_pane.rs` | `render` | 32 | ≤18 | Extract `cell_to_span` helper |
| `src/config/merge.rs` | `merge` | 24 | ≤18 | Per-section helpers |

Three borderline findings (`ui/list.rs`, `session/reader.rs`, `theme/registry.rs`)
were dropped from scope when the SonarCloud project profile threshold was
raised from 15 to 18.

## Constraints

- **No behavior change.** Every existing test, including `tests/snapshots`,
  passes without `cargo insta accept`. An unexpected snapshot diff means a
  real behavior change — stop and investigate.
- **No clippy / fmt regressions.** `cargo clippy --all-targets -- -D warnings`
  and `cargo fmt --all -- --check` continue to succeed.
- **No new dependencies.** Pure source reorganization.
- **Single PR.** Three commits, one per file, each independently green.

## Non-goals

- Architectural restructuring beyond extracting functions in the same file.
- Reaching the original Sonar default of 15 (the project threshold is 18).
- Refactoring the borderline files at complexity 16-18.
- Adding tests for previously uncovered behavior. Existing coverage is
  considered sufficient for a strict-no-change refactor.

## Plan A: `src/app/reducer.rs`

`reduce` is one giant `match` on 53 `Action` variants. Each non-trivial arm
becomes a free function in the same file. The `match` becomes pure dispatch:
one branch per variant, one or two lines each.

```rust
pub fn reduce(app: &mut App, action: Action) {
    match action {
        Action::NoOp => {}
        Action::CursorDown => cursor_down(app),
        Action::CursorUp => cursor_up(app),
        Action::EnterFilter => app.mode = Mode::FilterInput,
        Action::FilterChar(c) => filter_push(app, c),
        Action::FilterBackspace => filter_pop(app),
        // … remaining 47 arms, each one or two lines …
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
```

Rules:

- Each non-trivial arm body extracts to a free `fn` named after the action's
  verb (`cursor_down`, `filter_push`, `session_kill_request`, …).
- Trivial arms (single assignment like `app.mode = X`, or `NoOp`) stay inline.
- Helper signatures are `fn handler(app: &mut App, …payload)`. The payload
  arguments mirror the `Action` variant's fields by name and order.
- No reordering of mutations within an arm body.
- Existing constants (`SPLIT_MIN`, `SPLIT_MAX`, `SPLIT_STEP`, `SPLIT_DEFAULT`)
  and the existing `cycle_history` / `filtered_justfile_indices` helpers stay
  put.
- Drop the `#[allow(clippy::collapsible_if, clippy::collapsible_match)]`
  attribute on `reduce` if clippy stops needing it after extraction. Keep it
  if any helper still triggers either lint.

Resulting complexity: `reduce` becomes ≈ # arms ≤ 18 because S3776 charges
each `match` arm a fixed weight (1) at the same nesting level. Each helper
inherits the cognitive weight of one original arm, all individually under 18.

Test guard: the 15 tests in `tests/reducer_tests.rs` plus all snapshot tests.

## Plan B: `src/ui/session_pane.rs`

The hot spot is the nested `for r in 0..rows_count { for c in 0..cols { … } }`
loop in `render` (lines 46–74). Each iteration walks a vt100 cell and
accumulates a `Style` from up to five conditional modifiers. That body
extracts to a free function:

```rust
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
    if cell.bold()      { style = style.add_modifier(Modifier::BOLD); }
    if cell.italic()    { style = style.add_modifier(Modifier::ITALIC); }
    if cell.underline() { style = style.add_modifier(Modifier::UNDERLINED); }
    let ch = cell.contents();
    let text: String = if ch.is_empty() { " ".into() } else { ch.into() };
    Span::styled(text, style)
}
```

Inner loop becomes:

```rust
for r in 0..rows_count {
    let mut spans = Vec::with_capacity(cols);
    for c in 0..cols {
        spans.push(cell_to_span(grid, r as u16, c as u16));
    }
    lines.push(Line::from(spans));
}
```

`render` complexity drops to roughly the layout + scrollback math (~10).
`cell_to_span` complexity is ≤15.

Fallback: if `cell_to_span` itself comes in over 18 on Sonar, collapse the
three `Modifier` `if`s into a single bitmask:

```rust
let mut modifiers = Modifier::empty();
if cell.bold()      { modifiers |= Modifier::BOLD; }
if cell.italic()    { modifiers |= Modifier::ITALIC; }
if cell.underline() { modifiers |= Modifier::UNDERLINED; }
if !modifiers.is_empty() {
    style = style.add_modifier(modifiers);
}
```

Skip this bonus unless the first cut is still over threshold.

Test guard: 20 snapshots in `tests/snapshots/` cover initial layout, sessions
running, modals, multiple themes and icon styles. Any pixel-level change
fails them.

## Plan C: `src/config/merge.rs`

`merge` walks 4 optional sections (`ui`, `paths`, `logging`, `engine`). Each
section gets its own helper. The top-level becomes:

```rust
pub fn merge(file: ConfigFile, base: Config) -> Config {
    let mut out = base;
    if let Some(u) = file.ui      { merge_ui(&mut out, u); }
    if let Some(p) = file.paths   { merge_paths(&mut out, p); }
    if let Some(l) = file.logging { merge_logging(&mut out, l); }
    if let Some(e) = file.engine  { merge_engine(&mut out, e); }
    out
}
```

Each helper is private (`fn` with no `pub`) in the same module and accepts
its concrete section type from `src/config/file.rs`: `UiSection`,
`PathsSection`, `LoggingSection`, `EngineSection`.

`merge_ui` keeps the icon-style-parse-or-warn logic intact:

```rust
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
```

`merge_paths`, `merge_logging`, `merge_engine` are mechanical — the same
`if let Some(x) = … { out.field = transform(x); }` shape that lives in
`merge` today, just lifted into named functions. The transforms (`PathBuf::from`,
`Duration::from_millis`, `Duration::from_secs(days * 24 * 3600)`,
`mb * 1024 * 1024`) stay byte-for-byte identical.

Test guard: 8 inline tests in `src/config/merge.rs::tests`.

## Verification

Run the same checks before each commit and once more after the third:

```bash
cargo build
cargo clippy --all-targets -- -D warnings
cargo fmt --all -- --check
cargo test --all-targets -- --test-threads=1
```

Snapshot tests must pass without acceptance. The
`--test-threads=1` mirrors what the Nix sandbox uses to avoid the
pre-existing env-mutex flake in `config::paths`.

## PR shape

- **Branch:** `refactor/sonarcloud-cognitive-complexity`
- **Title:** `refactor: split high-complexity functions per SonarCloud findings`
- **Commits (one per file, in this order):**
  1. `refactor(config): split merge() into per-section helpers`
  2. `refactor(ui): extract cell_to_span from session_pane::render`
  3. `refactor(reducer): dispatch each Action variant to a free fn`
- **Body:** links to the three SonarCloud findings, before/after complexity
  table, the no-behavior-change guarantee, and the verification commands.

The order (smallest first) lets the smallest, simplest change land its
diff format and the pattern before the larger ones build on top.

## Post-merge

- Push to `main` triggers the SonarQube workflow.
- The next scan should report all three findings as `RESOLVED`.
- If any function still scores over 18, apply the bonus tactic noted in
  Plan B (`Modifier` bitmask) or extract one more helper from the offender.

## Risks

- **Subtle ordering changes inside a `match` arm body.** Mitigation: don't
  reorder mutations during extraction; lift each arm's body verbatim.
- **`#[allow]` attributes lifted off `reduce` could re-trigger lints inside
  helpers.** Mitigation: run clippy after each commit; reapply locally if
  needed.
- **Snapshot test flake.** Snapshots are stable on this codebase, but a
  whitespace-only `Span` change could shift one. Investigate any diff
  before considering acceptance.
- **`reduce` dispatch arms count alone could still exceed 18 under Sonar's
  weighting.** Mitigation: the rule charges `match` arms with weight 1 each,
  not nested, so 53 arms in a flat `match` come out around 53. If S3776
  still flags the dispatcher, group arms by domain into trivial dispatchers
  (e.g. `match action { _ if cursor_action(action) => …, _ if filter_action(action) => …}`)
  as a follow-up. Verify on Sonar after the first scan; do not over-engineer
  pre-emptively.

## Out of scope

- Reorganizing `src/app/reducer.rs` into submodules.
- Adding new test coverage for the 38 `Action` variants currently lacking
  dedicated tests.
- Touching the three borderline files (`ui/list.rs`, `session/reader.rs`,
  `theme/registry.rs`) now under the 18 threshold.
