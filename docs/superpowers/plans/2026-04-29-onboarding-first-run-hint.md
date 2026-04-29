# Onboarding First-Run Hint Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the `?` help glyph in the status bar discoverable on first launch, and signpost the same path in the README before the full keybindings table.

**Architecture:** Two stateless changes. (1) Accent the `?` glyph in `hint_for`'s Normal-mode and Session-focus arms by introducing a third style helper that uses `theme.accent`. (2) Insert a new `## First run` section between `## Usage` and `## Configuration` in `README.md` listing the six keys a new user needs.

**Tech Stack:** Rust 1.79+, ratatui (`Span`/`Line`/`Style`), markdown.

**Spec:** `docs/superpowers/specs/2026-04-29-onboarding-first-run-hint-design.md`

---

## File Structure

- Modify: `src/ui/status_bar.rs` — add accent helper, switch the `?` glyph to it in two `Mode::Normal` arms.
- Modify: `README.md` — insert `## First run` section between `## Usage` and `## Configuration`.

No new files. No tests added (spec rules them out: visual one-color change, no existing render tests in `status_bar.rs`, harness cost not justified).

---

### Task 0: Pre-flight — land the existing Usage edit

The working tree at the start of this plan has an uncommitted edit to
`README.md` from an earlier brainstorming turn: the `## Usage` block was
expanded with `--log-level`, `--help`, `--version`, and a discovery-rules
paragraph. Land it as its own commit before Task 2 so the First-run
section lands cleanly afterwards.

**Files:**
- Modify (already-modified working tree): `README.md`

- [ ] **Step 1: Confirm the working-tree change**

Run: `git diff README.md`
Expected: a single hunk replacing the old four-line `## Usage` block with the new six-line table plus a discovery-rules paragraph that mentions `.gitignore`, the hardcoded skip list (`node_modules`, `target`, `dist`, `.git`), recognized filenames, and the `d` switcher. No other files dirty.

If the diff looks different — or if `git status` shows the README clean — STOP and resync with the user before continuing.

- [ ] **Step 2: Commit the Usage edit**

```bash
git add README.md
git commit -m "$(cat <<'EOF'
docs(readme): clarify Usage flags and discovery rules

Spell out that lazyjust [PATH] walks recursively, that --justfile FILE
skips the walk, document --log-level / --help / --version, and add a
short paragraph on the discovery walk's gitignore + skip-list behavior.
EOF
)"
```

---

### Task 1: Highlight `?` glyph in status bar

**Files:**
- Modify: `src/ui/status_bar.rs:30-76`

- [ ] **Step 1: Read the current `hint_for` function**

Read `src/ui/status_bar.rs` lines 30-76. Confirm two `?` glyphs exist:
- Line 52 in the `Mode::Normal if matches!(app.focus, Focus::Session)` arm.
- Line 69 in the plain `Mode::Normal` arm.

Both currently use `k("?")` (which paints with `theme.fg`).

- [ ] **Step 2: Add the accent helper**

In `src/ui/status_bar.rs`, find the existing helper block at lines 31-33:

```rust
let sep = Span::styled("  ·  ", Style::default().fg(theme.dim));
let k = |s: &str| Span::styled(s.to_string(), Style::default().fg(theme.fg));
let d = |s: &str| Span::styled(s.to_string(), Style::default().fg(theme.dim));
```

Add a third helper directly below, so the block becomes:

```rust
let sep = Span::styled("  ·  ", Style::default().fg(theme.dim));
let k = |s: &str| Span::styled(s.to_string(), Style::default().fg(theme.fg));
let d = |s: &str| Span::styled(s.to_string(), Style::default().fg(theme.dim));
let a = |s: &str| Span::styled(s.to_string(), Style::default().fg(theme.accent));
```

- [ ] **Step 3: Swap `k("?")` for `a("?")` in the Session-focus arm**

In the `Mode::Normal if matches!(app.focus, Focus::Session)` arm (lines 35-55), replace the `k("?")` at line 52 with `a("?")`. The surrounding `Span::raw(" ")` and `d("help")` stay unchanged. The arm should end:

```rust
            sep,
            a("?"),
            Span::raw(" "),
            d("help"),
        ]),
```

- [ ] **Step 4: Swap `k("?")` for `a("?")` in the plain Normal arm**

In the `Mode::Normal =>` arm (lines 56-76), replace the `k("?")` at line 69 with `a("?")`. The surrounding `Span::raw(" ")` and `d("help")` stay unchanged. That part of the arm should read:

```rust
            sep.clone(),
            a("?"),
            Span::raw(" "),
            d("help"),
            sep,
            k("q"),
            Span::raw(" "),
            d("quit"),
        ]),
```

Leave every other glyph (`⏎`, `/`, `t`, `q`, `F12 / Ctrl+g`, `PgUp/PgDn`, `K`, `x`) on `k(...)` — only `?` is accented.

- [ ] **Step 5: Verify the project compiles**

Run: `cargo check`
Expected: clean exit, no warnings about the new closure (it is used twice).

- [ ] **Step 6: Run the existing test suite**

Run: `cargo test`
Expected: all tests pass. The change is render-only and no test inspects `status_bar.rs`, so this should be a green run.

- [ ] **Step 7: Manual smoke test**

Run: `cargo run -- .`
Expected: status bar's `?` glyph renders in the theme's accent color (cyan-ish on `tokyo-night`, the default), while the rest of the hint stays in `theme.fg` / `theme.dim`. Press `Tab` to focus the session pane and confirm the `?` in that hint also accents. Press `q` to quit.

- [ ] **Step 8: Commit**

```bash
git add src/ui/status_bar.rs
git commit -m "$(cat <<'EOF'
feat(ui): accent ? glyph in status-bar hint

Paint the help-modal glyph with theme.accent in Normal and Session-focus
hints so new users notice the entry point. Other glyphs stay on theme.fg.
EOF
)"
```

---

### Task 2: Add "First run" section to README

**Files:**
- Modify: `README.md` — insert new section between `## Usage` and `## Configuration`.

- [ ] **Step 1: Locate the insertion point**

Open `README.md` and find the `## Usage` section. After Task 0 it ends with the paragraph describing discovery rules (gitignore, hardcoded skip list, recognized filenames, `d` switcher). The next heading is `## Configuration`. The new `## First run` section goes between them.

- [ ] **Step 2: Insert the new section**

Add the following block immediately before `## Configuration`:

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

Keep one blank line before `## Configuration`.

- [ ] **Step 3: Verify the README still renders cleanly**

Run: `git diff README.md`
Expected: a single contiguous addition between `## Usage` (and its trailing
paragraph) and `## Configuration`. No other lines moved.

If the repository has a markdown linter wired in (`just lint` or similar),
run it. Otherwise skip.

- [ ] **Step 4: Commit**

```bash
git add README.md
git commit -m "$(cat <<'EOF'
docs(readme): add First run section

Surface the six essential keys (?, /, Enter, t, d, q) plus a
troubleshooting hint right after Usage so new users find the help modal
and the startup-errors modal without scanning the full keybindings table.
EOF
)"
```

---

## Verification

After both tasks land:

- `cargo build --release` — succeeds.
- `cargo test` — green.
- `cargo run -- .` — `?` is visibly accented in the status bar, in both list and session focus.
- `README.md` — `## First run` section sits between `## Usage` and `## Configuration` with a six-row key table and a troubleshooting line.

## Out of scope

- First-launch detection / persisted "seen tour" flag.
- Help-modal restyle.
- README screenshot / animated GIF (separate doc task).
- Splitting README into per-topic files.
