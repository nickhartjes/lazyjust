# Shorten Path Display Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace full absolute justfile paths in the dropdown picker and top bar with a `~`-anchored, segment-aware shortened form, so long paths stay readable inside the available width.

**Architecture:** Add a single pure helper `crate::ui::path_display::shorten(&Path, max_width: usize) -> String` that handles `$HOME` → `~` substitution and segment-aware middle truncation. The dropdown modal becomes adaptive-width (clamped 40..=100 cols). The top bar computes its path budget from the surrounding span widths. No new dependencies.

**Tech Stack:** Rust 1.79+, ratatui 0.30, insta snapshots, std-only (no `unicode-width` direct dep).

**Spec:** `docs/superpowers/specs/2026-04-28-shorten-path-display-design.md`

**Repo conventions:**
- Run all `cargo` and `git` commands from the repo root (`/Users/nick.hartjes/projects/nh/lazyjust-rs`).
- Tests run with `--test-threads=1` because of a pre-existing env-mutex flake in `config::paths`.
- Snapshot tests live under `tests/snapshots/`. Review diffs with `cargo insta review`; do not blindly run `cargo insta accept`.
- Default branch is `main`; PR target is `main`; conventional commit prefixes (`feat:`, `refactor:`, `test:`, `docs:`, `chore:`).

**Conventions used in this plan:**
- Order: write test → run test → implement → run test → commit.
- Code blocks show the entire block to write or replace, not diffs.
- "Expected" lines describe the substring you should see in tool output.

---

## File Structure

| Path | Change | Responsibility |
|---|---|---|
| `src/ui/path_display.rs` | Create | Pure `shorten(path, max_width)` helper + unit tests. |
| `src/ui/mod.rs` | Modify | Register the new submodule with `pub mod path_display;`. |
| `src/ui/modal.rs` | Modify | Adaptive dropdown width; render rows via `shorten`. |
| `src/ui/top_bar.rs` | Modify | Compute available path width; render path span via `shorten`. |
| `tests/snapshots/` | Possibly regenerate | Existing snapshots use the relative path `./justfile`, which is shorter than every modal/top-bar budget, so they should not change. Verified in Task 5. |

No new files outside `src/ui/`. No new dependencies in `Cargo.toml`.

---

## Task 0: Branch and baseline

Cut a clean branch and confirm the test suite is green so any later regression is unambiguous.

**Files:** none modified.

- [ ] **Step 1: Confirm clean working tree on main**

Run:

```bash
git status
git checkout main
git pull --ff-only origin main
```

Expected: `working tree clean` and the branch is up to date with `origin/main`.

- [ ] **Step 2: Cut the working branch**

Run:

```bash
git checkout -b feat/shorten-path-display
```

Expected: `Switched to a new branch 'feat/shorten-path-display'`.

- [ ] **Step 3: Baseline test run**

Run:

```bash
cargo test -- --test-threads=1
```

Expected: all tests pass (`test result: ok.`). Note the count for sanity-checking later runs.

- [ ] **Step 4: Baseline build with warnings**

Run:

```bash
cargo build
```

Expected: compiles cleanly. No new warnings introduced by later steps.

---

## Task 1: `shorten` helper — TDD

Build the pure helper with tests first, before any caller exists. Each behavior gets its own test, written and verified failing before the implementation arrives.

**Files:**
- Create: `src/ui/path_display.rs`
- Modify: `src/ui/mod.rs` (one line)

### Step 1: Register the module

- [ ] **Step 1.1: Add the module declaration to `src/ui/mod.rs`**

Replace the top-of-file `pub mod` block in `src/ui/mod.rs` (currently lines 1–15) with this exact block (alphabetical, mirroring the existing style):

```rust
pub mod focus;
pub mod help;
pub mod icon_style;
pub mod layout;
pub mod list;
pub mod modal;
pub mod modal_base;
pub mod param_modal;
pub mod path_display;
pub mod preview;
pub mod scrollbar;
pub mod session_header;
pub mod session_pane;
pub mod status_bar;
pub mod theme_picker;
pub mod top_bar;
```

- [ ] **Step 1.2: Create an empty `src/ui/path_display.rs` so the build still compiles**

Create `src/ui/path_display.rs` with this content (compiles, no public API yet — keeps the next step's `cargo test` failure focused on the missing function, not on a missing file):

```rust
// Helper for rendering filesystem paths inside the TUI.
//
// See docs/superpowers/specs/2026-04-28-shorten-path-display-design.md.
```

- [ ] **Step 1.3: Verify the project still builds**

Run:

```bash
cargo build
```

Expected: build succeeds.

- [ ] **Step 1.4: Commit the module skeleton**

Run:

```bash
git add src/ui/mod.rs src/ui/path_display.rs
git commit -m "chore(ui): add empty path_display module"
```

### Step 2: First failing test — paths under width are returned unchanged

- [ ] **Step 2.1: Add the first test**

Append to `src/ui/path_display.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn returns_unchanged_when_within_width() {
        let p = PathBuf::from("/tmp/justfile");
        assert_eq!(shorten(&p, 80), "/tmp/justfile");
    }
}
```

- [ ] **Step 2.2: Run the test — expect a compile error**

Run:

```bash
cargo test --lib path_display -- --test-threads=1
```

Expected: compilation fails with something like `cannot find function 'shorten' in this scope`. That is the failure we want before writing the implementation.

- [ ] **Step 2.3: Implement just enough to pass**

Replace the contents of `src/ui/path_display.rs` with:

```rust
// Helper for rendering filesystem paths inside the TUI.
//
// See docs/superpowers/specs/2026-04-28-shorten-path-display-design.md.

use std::path::Path;

/// Render `path` for display in the TUI, fitting inside `max_width` columns
/// when possible.
///
/// Behaviour:
/// 1. The leading `$HOME` segment is replaced with `~`.
/// 2. If the resulting display width is `<= max_width`, the string is
///    returned unchanged.
/// 3. Otherwise a segment-aware middle truncation is applied, preserving
///    the root anchor and the filename, with `…` between them. See the
///    design doc for the full algorithm.
pub fn shorten(path: &Path, max_width: usize) -> String {
    let s = render_with_home_tilde(path);
    if display_width(&s) <= max_width {
        return s;
    }
    middle_truncate(&s, max_width)
}

fn render_with_home_tilde(path: &Path) -> String {
    let raw = path.display().to_string();
    let home = match std::env::var_os("HOME") {
        Some(h) if !h.is_empty() => h,
        _ => return raw,
    };
    let home = home.to_string_lossy();
    if raw == *home {
        return "~".to_string();
    }
    let prefix = format!("{home}/");
    if let Some(rest) = raw.strip_prefix(&prefix) {
        format!("~/{rest}")
    } else {
        raw
    }
}

fn display_width(s: &str) -> usize {
    // ASCII-dominant in practice; chars() is good enough today and avoids
    // pulling unicode-width as a direct dependency. See spec for the
    // rationale and upgrade path.
    s.chars().count()
}

fn middle_truncate(s: &str, max_width: usize) -> String {
    // Implementation arrives in a later step. Returning `s` here would make
    // the next test pass for the wrong reason, so return the minimal
    // safe placeholder: the input itself is never wider than the input.
    s.to_string()
}
```

- [ ] **Step 2.4: Run the test — expect pass**

Run:

```bash
cargo test --lib path_display -- --test-threads=1
```

Expected: `test result: ok. 1 passed`.

- [ ] **Step 2.5: Commit**

Run:

```bash
git add src/ui/path_display.rs
git commit -m "feat(ui): add shorten() pass-through for short paths"
```

### Step 3: Failing test — `$HOME` is replaced with `~`

- [ ] **Step 3.1: Add the test**

Inside the existing `tests` module in `src/ui/path_display.rs`, add:

```rust
#[test]
fn replaces_home_with_tilde() {
    // Pin HOME so the test is deterministic across machines.
    std::env::set_var("HOME", "/Users/nick");
    let p = PathBuf::from("/Users/nick/projects/foo/justfile");
    assert_eq!(shorten(&p, 80), "~/projects/foo/justfile");
}

#[test]
fn home_only_path_renders_as_tilde() {
    std::env::set_var("HOME", "/Users/nick");
    let p = PathBuf::from("/Users/nick");
    assert_eq!(shorten(&p, 80), "~");
}

#[test]
fn unrelated_path_is_unaffected_by_home() {
    std::env::set_var("HOME", "/Users/nick");
    let p = PathBuf::from("/var/log/justfile");
    assert_eq!(shorten(&p, 80), "/var/log/justfile");
}
```

Note on the env mutation: these tests share process state with the rest of the suite, so the project already runs `cargo test -- --test-threads=1`. Setting `HOME` here is safe under that constraint. If the wider suite ever moves off single-threaded execution, gate these tests with the same `Mutex<()>` pattern used in `tests/config_loader.rs` (search for `ENV_LOCK`).

- [ ] **Step 3.2: Run the tests — expect pass for the home-replacement cases (already implemented in Step 2.3)**

Run:

```bash
cargo test --lib path_display -- --test-threads=1
```

Expected: `test result: ok. 4 passed`. If anything fails, the implementation in Step 2.3 has a bug — fix it before continuing.

- [ ] **Step 3.3: Commit**

Run:

```bash
git add src/ui/path_display.rs
git commit -m "test(ui): cover HOME→~ substitution in shorten()"
```

### Step 4: Failing test — long paths get middle-truncated

- [ ] **Step 4.1: Add the failing test**

Inside the same `tests` module:

```rust
#[test]
fn middle_truncates_long_absolute_path() {
    std::env::set_var("HOME", "/Users/nick");
    let p =
        PathBuf::from("/Users/nick/projects/entrnce/trader/services/api/justfile");
    let out = shorten(&p, 28);
    assert!(out.starts_with("~/"), "expected leading ~/, got {out:?}");
    assert!(out.contains('…'), "expected ellipsis, got {out:?}");
    assert!(out.ends_with("/justfile"), "expected /justfile tail, got {out:?}");
    assert!(out.chars().count() <= 28, "expected ≤28 cols, got {} ({out:?})", out.chars().count());
}

#[test]
fn middle_truncates_non_home_path() {
    std::env::remove_var("HOME");
    let p = PathBuf::from("/var/very/deeply/nested/repo/sub/dir/justfile");
    let out = shorten(&p, 24);
    assert!(out.starts_with("/…/"), "expected /…/ root anchor, got {out:?}");
    assert!(out.ends_with("/justfile"), "expected /justfile tail, got {out:?}");
    assert!(out.chars().count() <= 24, "got {} ({out:?})", out.chars().count());
}

#[test]
fn very_tight_budget_returns_root_ellipsis_filename_even_if_over_budget() {
    std::env::remove_var("HOME");
    // /…/justfile is 11 chars; max_width 5 cannot fit it. Spec says we still
    // return that form rather than truncating inside a segment.
    let p = PathBuf::from("/var/x/y/z/justfile");
    let out = shorten(&p, 5);
    assert_eq!(out, "/…/justfile");
}

#[test]
fn root_only_path_unchanged() {
    std::env::remove_var("HOME");
    let p = PathBuf::from("/justfile");
    assert_eq!(shorten(&p, 5), "/justfile");
}

#[test]
fn relative_path_with_no_root_segment() {
    std::env::remove_var("HOME");
    let p = PathBuf::from("a/b/c/d/e/justfile");
    let out = shorten(&p, 14);
    // Root anchor for a relative path is its first segment.
    assert!(out.starts_with("a/"), "got {out:?}");
    assert!(out.contains('…'));
    assert!(out.ends_with("/justfile"));
    assert!(out.chars().count() <= 14, "got {} ({out:?})", out.chars().count());
}
```

- [ ] **Step 4.2: Run the tests — expect failures**

Run:

```bash
cargo test --lib path_display -- --test-threads=1
```

Expected: the four new long-path tests fail because `middle_truncate` is still a stub that returns the input unchanged. The earlier 4 tests still pass.

- [ ] **Step 4.3: Implement `middle_truncate`**

Replace the existing `middle_truncate` placeholder in `src/ui/path_display.rs` with:

```rust
fn middle_truncate(s: &str, max_width: usize) -> String {
    // Split into segments, preserving the leading "" produced by an
    // absolute path. For an absolute path the first element is "" so
    // `segments[0]` is the empty root and `segments[1..]` are real names.
    let segments: Vec<&str> = s.split('/').collect();

    // Need at least: <root>/<filename>. Anything shorter has nothing to do.
    if segments.len() < 2 {
        return s.to_string();
    }

    let root = segments[0]; // "" for /abs paths, "~" for tilde paths,
                            //  first dir for relative paths.
    let filename = segments[segments.len() - 1];
    let middle: &[&str] = &segments[1..segments.len() - 1];

    // Form the minimum candidate "<root>/…/<filename>".
    let minimum = format!("{root}/…/{filename}");

    // Greedily prepend middle segments back in from the right (closest to
    // the filename first), keeping the form "<root>/…/<seg>/.../<filename>".
    let mut kept_from_right: usize = 0;
    let mut current = minimum.clone();
    while kept_from_right < middle.len() {
        let take = kept_from_right + 1;
        let tail_segments = &middle[middle.len() - take..];
        let candidate = format!("{root}/…/{}/{filename}", tail_segments.join("/"));
        if display_width(&candidate) > max_width {
            break;
        }
        current = candidate;
        kept_from_right = take;
    }

    // If we never kept any tail segment AND the minimum itself fits,
    // emit the minimum. If the minimum doesn't fit either, the spec says
    // emit it anyway and let ratatui clip — never split a segment.
    if kept_from_right == 0 {
        return minimum;
    }

    // If we kept *all* middle segments we never actually elided anything;
    // that case can't happen here because the caller only invokes
    // middle_truncate when the un-truncated string was over budget.
    current
}
```

- [ ] **Step 4.4: Run the tests — expect pass**

Run:

```bash
cargo test --lib path_display -- --test-threads=1
```

Expected: `test result: ok. 9 passed`.

- [ ] **Step 4.5: Run clippy on the new module**

Run:

```bash
cargo clippy --all-targets -- -D warnings
```

Expected: no warnings.

- [ ] **Step 4.6: Commit**

Run:

```bash
git add src/ui/path_display.rs
git commit -m "feat(ui): segment-aware middle truncation in shorten()"
```

---

## Task 2: Wire the dropdown to the new helper

**Files:**
- Modify: `src/ui/modal.rs`

- [ ] **Step 1: Open `src/ui/modal.rs` and locate `render_dropdown`**

Confirm the current implementation matches:

```rust
fn render_dropdown(
    f: &mut Frame,
    app: &App,
    filter: &str,
    cursor: usize,
    theme: &crate::theme::Theme,
) {
    let area = crate::ui::modal_base::centered(f.area(), 60, 14);
    crate::ui::modal_base::clear(f, area);
    let indices = crate::app::reducer::filtered_justfile_indices(app, filter);
    let items: Vec<ListItem> = indices
        .iter()
        .map(|&i| ListItem::new(app.justfiles[i].path.display().to_string()))
        .collect();
    ...
}
```

If the surrounding code has drifted, stop and reconcile before continuing.

- [ ] **Step 2: Replace `render_dropdown` with the adaptive-width version**

Replace the entire `render_dropdown` function in `src/ui/modal.rs` with:

```rust
fn render_dropdown(
    f: &mut Frame,
    app: &App,
    filter: &str,
    cursor: usize,
    theme: &crate::theme::Theme,
) {
    let frame_w = f.area().width;
    let modal_w = frame_w.saturating_sub(4).clamp(40, 100);
    let area = crate::ui::modal_base::centered(f.area(), modal_w, 14);
    crate::ui::modal_base::clear(f, area);
    // Inside the modal: 2 cols of border + 2 cols of left/right padding.
    let row_max = (modal_w as usize).saturating_sub(4);
    let indices = crate::app::reducer::filtered_justfile_indices(app, filter);
    let items: Vec<ListItem> = indices
        .iter()
        .map(|&i| {
            ListItem::new(crate::ui::path_display::shorten(
                &app.justfiles[i].path,
                row_max,
            ))
        })
        .collect();
    let mut state = ListState::default();
    state.select(Some(cursor.min(items.len().saturating_sub(1))));
    let title = format!("justfile: /{filter}");
    let list = List::new(items)
        .block(crate::ui::modal_base::block(&title, theme))
        .highlight_style(
            Style::default()
                .bg(theme.highlight)
                .fg(theme.selected_fg)
                .add_modifier(Modifier::BOLD),
        );
    f.render_stateful_widget(list, area, &mut state);
}
```

- [ ] **Step 3: Build and run the test suite**

Run:

```bash
cargo build
cargo test -- --test-threads=1
```

Expected: build succeeds; all tests pass. The existing snapshot fixtures use the path `./justfile`, which is shorter than every possible budget and contains no `$HOME` prefix, so `shorten` returns it unchanged — snapshots should be unaffected. If insta reports a diff, stop and inspect with `cargo insta review`.

- [ ] **Step 4: Commit**

Run:

```bash
git add src/ui/modal.rs
git commit -m "feat(ui): adaptive dropdown width and shortened path rows"
```

---

## Task 3: Wire the top bar to the new helper

The top bar lays out a single line in `cols[0]` followed by a fixed-width badge in `cols[1]`. The path span sits between two `"  · "` separators. To compute a budget for `shorten` we need to subtract every other span's width from `cols[0].width`.

**Files:**
- Modify: `src/ui/top_bar.rs`

- [ ] **Step 1: Confirm the current shape of `render`**

Open `src/ui/top_bar.rs` and confirm `render` matches the version recorded in the spec (the path is rendered as `Span::styled(path, …)` where `path = jf.map(|j| j.path.display().to_string()).unwrap_or_else(|| "<no justfile>".into())`).

- [ ] **Step 2: Replace `render` with a budget-aware version**

Replace the existing `render` function in `src/ui/top_bar.rs` with:

```rust
pub fn render(f: &mut Frame, area: Rect, app: &App, theme: &crate::theme::Theme) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(1), Constraint::Length(badge_width(app))])
        .split(area);

    let jf = app.active_justfile();
    let count = jf.map(|j| j.recipes.len()).unwrap_or(0);

    // Build every span *except* the path first, so we know exactly how
    // many columns the path itself may consume.
    let leading: Vec<Span> = vec![
        Span::styled("▌", Style::default().fg(theme.accent)),
        Span::raw(" "),
        Span::styled(
            "lazyjust",
            Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
        ),
        Span::styled("  · ", Style::default().fg(theme.dim)),
    ];
    let recipes_text = format!("{count} recipes");
    let trailing: Vec<Span> = vec![
        Span::styled("  · ", Style::default().fg(theme.dim)),
        Span::styled(recipes_text.clone(), Style::default().fg(theme.dim)),
    ];
    let errors_span: Option<Span> = (!app.startup_errors.is_empty()).then(|| {
        Span::styled(
            format!(" {} load errors ", app.startup_errors.len()),
            Style::default()
                .fg(theme.error)
                .bg(theme.bg)
                .add_modifier(Modifier::BOLD),
        )
    });

    let chrome_width: usize = leading
        .iter()
        .chain(trailing.iter())
        .chain(errors_span.iter())
        .map(|s| s.content.chars().count())
        .sum::<usize>()
        + if errors_span.is_some() { 3 } else { 0 }; // "   " separator before errors

    let path_budget: usize = (cols[0].width as usize)
        .saturating_sub(chrome_width)
        .max(16); // never collapse below "<root>/…/<filename>"-ish space

    let path = match jf {
        Some(j) => crate::ui::path_display::shorten(&j.path, path_budget),
        None => "<no justfile>".to_string(),
    };

    let mut spans: Vec<Span> = Vec::with_capacity(8);
    spans.extend(leading);
    spans.push(Span::styled(path, Style::default().fg(theme.dim)));
    spans.extend(trailing);
    if let Some(err_span) = errors_span {
        spans.push(Span::raw("   "));
        spans.push(err_span);
    }
    f.render_widget(Paragraph::new(Line::from(spans)), cols[0]);

    if let Some(j) = jf {
        let parent = j
            .path
            .parent()
            .and_then(|p| p.file_name())
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        if !parent.is_empty() {
            let pill = Line::from(Span::styled(
                format!(" {parent} "),
                Style::default().fg(theme.badge_fg).bg(theme.badge_bg),
            ));
            let right = Paragraph::new(pill).alignment(Alignment::Right);
            f.render_widget(right, cols[1]);
        }
    }
}
```

The `badge_width` helper below the function stays as-is.

- [ ] **Step 3: Build, test, clippy**

Run each in turn:

```bash
cargo build
cargo test -- --test-threads=1
cargo clippy --all-targets -- -D warnings
```

Expected: build clean, all tests pass, no clippy warnings. If insta reports a diff, inspect with `cargo insta review` — fixtures use `./justfile` which is short, so any diff is a regression and must be investigated rather than accepted.

- [ ] **Step 4: Commit**

Run:

```bash
git add src/ui/top_bar.rs
git commit -m "feat(ui): shorten justfile path in top bar with budget-aware width"
```

---

## Task 4: Manual smoke test

The TUI render paths can hide width-math bugs that the snapshot suite (which uses `./justfile`) won't catch. Verify the dropdown and top bar by hand on real long paths.

**Files:** none modified.

- [ ] **Step 1: Build a debug binary**

Run:

```bash
cargo build
```

Expected: clean build.

- [ ] **Step 2: Construct a fixture with a deeply nested justfile**

Run:

```bash
mkdir -p /tmp/lazyjust-pathtest/projects/entrnce/trader/services/api
printf 'default:\n\techo hi\n' > /tmp/lazyjust-pathtest/projects/entrnce/trader/services/api/justfile
mkdir -p /tmp/lazyjust-pathtest/projects/entrnce/trader/services/billing
printf 'default:\n\techo hi\n' > /tmp/lazyjust-pathtest/projects/entrnce/trader/services/billing/justfile
mkdir -p /tmp/lazyjust-pathtest/projects/short
printf 'default:\n\techo hi\n' > /tmp/lazyjust-pathtest/projects/short/justfile
```

- [ ] **Step 3: Launch lazyjust against that root**

Run:

```bash
HOME=/tmp/lazyjust-pathtest target/debug/lazyjust /tmp/lazyjust-pathtest
```

(`HOME` is overridden so the test fixture's paths get the `~` substitution and we can see the helper at work without polluting the real `$HOME`.)

- [ ] **Step 4: Open the dropdown picker and confirm three things**

Press the dropdown shortcut (`/` or whatever the help modal lists for "open justfile-switcher dropdown" — see `src/ui/help.rs:52`).

Confirm:
1. The modal is wider than the previous fixed 60 cols on a wide terminal.
2. Each row begins with `~/projects/...` rather than `/tmp/lazyjust-pathtest/projects/...`.
3. Long rows show a `…` somewhere in the middle, never a hard cut at the right border.

If any of these fails, file the issue against the relevant task above and fix before moving on.

- [ ] **Step 5: Inspect the top bar**

Pick the `services/api/justfile` row and confirm the top bar shows the same `~/...` form. Resize the terminal narrower and confirm the path collapses further (more elision) without breaking the layout or pushing the badge off-screen.

- [ ] **Step 6: Tear down the fixture**

Run:

```bash
rm -rf /tmp/lazyjust-pathtest
```

- [ ] **Step 7: Commit nothing (smoke test only)**

No commit. If anything required a code change, commit that under the appropriate task above and re-run the smoke test before continuing.

---

## Task 5: Snapshot review and PR

**Files:** none modified directly. May regenerate `tests/snapshots/*.snap` if a deliberate change is required.

- [ ] **Step 1: Run the full suite one more time**

Run:

```bash
cargo test -- --test-threads=1
```

Expected: all tests pass, no insta diffs reported.

- [ ] **Step 2: If insta did report a diff, decide before accepting**

Existing fixtures use `./justfile` and run inside an 80×24 `TestBackend`. With the new code:
- The dropdown's `modal_w` becomes `clamp(80 - 4, 40, 100) = 76` — wider than the old 60. Any *dropdown-mode* snapshot will legitimately change because the modal is wider.
- The top bar still calls `shorten` on `./justfile`, which fits in any budget and contains no `$HOME` prefix — so non-dropdown snapshots should be byte-identical.

If diffs are limited to dropdown-mode snapshots, that's expected behavior change. Review with:

```bash
cargo insta review
```

Accept the dropdown-mode diffs and reject anything else; investigate any non-dropdown diff before accepting.

- [ ] **Step 3: Update the spec if any algorithm detail changed during implementation**

Open `docs/superpowers/specs/2026-04-28-shorten-path-display-design.md`. If your implementation diverged from the spec (e.g. you swapped to a different width metric, or added a config knob), update the spec inline and stage it for the same PR. If it didn't, skip this step.

- [ ] **Step 4: Push and open the PR**

Run:

```bash
git push -u origin feat/shorten-path-display
gh pr create --title "feat(ui): shorten justfile path display" --body "$(cat <<'EOF'
## Summary
- Replace full absolute justfile paths in the dropdown picker and top bar with a `~`-anchored, segment-aware shortened form (`~/.../parent/justfile`).
- Make the dropdown modal width adaptive (clamped 40..=100 cols) so we use available space before truncating.
- Add a focused `crate::ui::path_display::shorten` helper with unit tests covering home substitution, middle truncation, and tight-budget edge cases.

## Test plan
- [ ] `cargo test -- --test-threads=1` is green
- [ ] `cargo clippy --all-targets -- -D warnings` is clean
- [ ] Smoke test against `/tmp/lazyjust-pathtest` (see plan, Task 4) shows shortened paths in dropdown and top bar
- [ ] Resizing the terminal narrower collapses the path further without breaking layout

Spec: `docs/superpowers/specs/2026-04-28-shorten-path-display-design.md`
EOF
)"
```

Expected: PR URL printed.

---

## Self-Review Notes (for the plan author, not the executor)

Spec coverage check:
- Module + signature: Task 1.
- Home → `~`: Task 1, Step 3.
- Width measurement (`chars().count()`): Task 1, Step 2.3.
- Segment-aware middle truncation: Task 1, Step 4.
- Tight-budget fallback: Task 1, Step 4.1 (`very_tight_budget_returns_root_ellipsis_filename_even_if_over_budget`).
- Dropdown adaptive width: Task 2.
- Dropdown row shortening: Task 2.
- Top bar budget calc + shortening: Task 3.
- No new dependencies: enforced by not editing `Cargo.toml`; verifiable via `git diff main -- Cargo.toml Cargo.lock`.
- Status bar out of scope: matches spec (status bar shows the filter prompt, not a path).

Type/name consistency:
- `shorten(&Path, usize) -> String` — used identically in Tasks 1, 2, 3.
- Module path `crate::ui::path_display::shorten` — same in Tasks 2, 3.
- `row_max` (Task 2) and `path_budget` (Task 3) are local names; no cross-task references.
