# Discovery Always-Walks Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace `discovery::discover` / `discover_explicit` with a single entry point that always walks, optionally pins a justfile as the active one, and unions the walk roots when both `[PATH]` and `--justfile` are passed.

**Architecture:** Collapse the three current discovery entry points into one `discover(opts: DiscoverOptions)` call. Compute walk roots from the option struct, run the existing walker on each, dedup, force-include the pin, parse, and return a `DiscoveryResult` that now carries an `active_index`. The CLI's `path` becomes `Option<PathBuf>` so the caller can tell apart "user omitted PATH" from an explicit `.`.

**Tech Stack:** Rust 1.79+, clap, `ignore::WalkBuilder`, existing `walk::walk_justfiles` helper.

**Spec:** `docs/superpowers/specs/2026-04-29-discovery-always-walk-design.md`

---

## File Structure

- Modify: `src/cli.rs` — `path` becomes `Option<PathBuf>`.
- Modify: `src/discovery/mod.rs` — collapse three entry points into one. Add `DiscoverOptions`, `walk_roots`, pin-merge, `active_index`.
- Modify: `src/lib.rs::async_main` — call new API, post-assign `active_justfile`.
- Modify: `tests/discovery_integration.rs` — rewrite tests against new API + add four new behavior-matrix tests. The fixture at `tests/fixtures/tree/` already has a root `justfile`, a `sub/justfile`, a `node_modules/justfile` (skip-list), and an `ignored_by_gitignore/justfile` (gitignored), which is enough for every new case.
- Modify: `README.md` — flip the `--justfile` line in Usage to say it pins + still walks for siblings.

No new files.

---

### Task 1: Make CLI `path` an Option

**Files:**
- Modify: `src/cli.rs:7-9`

- [ ] **Step 1: Update the `Cli::path` field**

In `src/cli.rs`, replace:

```rust
    /// Project root to scan (defaults to current directory).
    #[arg(value_name = "PATH", default_value = ".")]
    pub path: PathBuf,
```

with:

```rust
    /// Project root to scan. Defaults to the current directory when omitted.
    #[arg(value_name = "PATH")]
    pub path: Option<PathBuf>,
```

- [ ] **Step 2: Verify the project still compiles (it won't)**

Run: `cargo check`
Expected: a single error in `src/lib.rs` complaining that `&cli.path` doesn't match `&Path` (it's now `&Option<PathBuf>`). That's expected — Task 4 patches the call site. Leave the breakage in place; we'll fix it after the discovery API is updated.

(Do not commit yet — code is broken, the rest of this plan resolves the break.)

---

### Task 2: Add `DiscoverOptions` and `walk_roots` (with unit test)

**Files:**
- Modify: `src/discovery/mod.rs`

- [ ] **Step 1: Add `DiscoverOptions` struct**

In `src/discovery/mod.rs`, replace the existing `pub struct DiscoveryResult { ... }` block (lines 9-13) with:

```rust
#[derive(Debug, Default, Clone, Copy)]
pub struct DiscoverOptions<'a> {
    pub path: Option<&'a Path>,
    pub justfile: Option<&'a Path>,
}

#[derive(Debug)]
pub struct DiscoveryResult {
    pub justfiles: Vec<Justfile>,
    pub errors: Vec<(PathBuf, String)>,
    /// Index into `justfiles` of the entry to pre-select on launch.
    /// 0 when no pin is set or the pin is not found in the result.
    pub active_index: usize,
}
```

- [ ] **Step 2: Add `walk_roots` helper at the end of the file**

Append at the bottom of `src/discovery/mod.rs` (above the `dump_and_parse` helper is fine — anywhere private to the module):

```rust
fn walk_roots(opts: &DiscoverOptions) -> Vec<PathBuf> {
    let mut roots: Vec<PathBuf> = Vec::new();

    if let Some(p) = opts.path {
        roots.push(p.to_path_buf());
    }

    if let Some(jf) = opts.justfile {
        let abs = walk::absolutize(jf.to_path_buf());
        let parent = abs
            .parent()
            .map(Path::to_path_buf)
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| PathBuf::from("."));
        roots.push(parent);
    }

    if roots.is_empty() {
        roots.push(PathBuf::from("."));
    }

    let mut seen: Vec<PathBuf> = Vec::new();
    let mut deduped: Vec<PathBuf> = Vec::new();
    for r in roots {
        let key = walk::absolutize(r.clone());
        if !seen.iter().any(|s| s == &key) {
            seen.push(key);
            deduped.push(r);
        }
    }
    deduped
}
```

- [ ] **Step 3: Add a unit-test module covering the four behavior-matrix rows**

Append to `src/discovery/mod.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn walk_roots_default_is_cwd() {
        let roots = walk_roots(&DiscoverOptions::default());
        assert_eq!(roots, vec![PathBuf::from(".")]);
    }

    #[test]
    fn walk_roots_path_only() {
        let p = PathBuf::from("some/path");
        let roots = walk_roots(&DiscoverOptions {
            path: Some(&p),
            justfile: None,
        });
        assert_eq!(roots, vec![PathBuf::from("some/path")]);
    }

    #[test]
    fn walk_roots_justfile_only_uses_parent() {
        let jf = PathBuf::from("some/path/justfile");
        let roots = walk_roots(&DiscoverOptions {
            path: None,
            justfile: Some(&jf),
        });
        assert_eq!(roots.len(), 1);
        assert!(
            roots[0].ends_with("some/path"),
            "expected parent of justfile, got {:?}",
            roots[0]
        );
    }

    #[test]
    fn walk_roots_path_plus_justfile_unions() {
        let p = PathBuf::from("a");
        let jf = PathBuf::from("b/justfile");
        let roots = walk_roots(&DiscoverOptions {
            path: Some(&p),
            justfile: Some(&jf),
        });
        assert_eq!(roots.len(), 2);
        assert_eq!(roots[0], PathBuf::from("a"));
        assert!(roots[1].ends_with("b"));
    }

    #[test]
    fn walk_roots_path_equal_to_justfile_parent_dedups() {
        let p = PathBuf::from("a");
        let jf = PathBuf::from("a/justfile");
        let roots = walk_roots(&DiscoverOptions {
            path: Some(&p),
            justfile: Some(&jf),
        });
        assert_eq!(roots.len(), 1);
    }
}
```

- [ ] **Step 4: Run the new unit tests**

Run: `cargo test --lib discovery::tests::walk_roots`
Expected: 5 passed; 0 failed.

(Discovery doesn't currently have a `tests` submodule — the new module compiles standalone and will not collide with anything.)

The crate-level build still has the broken `lib.rs` call site from Task 1; that resolves in Task 4. The `--lib` test invocation above succeeds because the `tests` submodule sits inside `discovery` and doesn't import `lib.rs`'s broken match arm.

If `cargo test --lib` fails to compile because of the `lib.rs` break: skip running this step right now and verify the unit tests at the end of Task 4 instead.

- [ ] **Step 5: Commit**

```bash
git add src/discovery/mod.rs
git commit -m "$(cat <<'EOF'
refactor(discovery): introduce DiscoverOptions + walk_roots helper

Add the new DiscoverOptions struct, the active_index field on
DiscoveryResult, and the walk_roots helper that computes the union of
the positional [PATH] and the parent of --justfile. The existing
discover / discover_explicit entry points are still in place; the next
commit collapses them.
EOF
)"
```

---

### Task 3: Replace the three discovery entry points with one

**Files:**
- Modify: `src/discovery/mod.rs`

- [ ] **Step 1: Replace the entry-point block**

In `src/discovery/mod.rs`, delete the existing block:

```rust
pub fn discover(root: &Path) -> Result<DiscoveryResult> {
    discover_inner(root, None)
}

/// Like `discover`, but pin the discovery to a single explicit justfile,
/// bypassing the walk. The path is lexically absolutized so the spawned
/// `just --justfile` resolves it from any PTY CWD.
pub fn discover_explicit(justfile: &Path) -> Result<DiscoveryResult> {
    discover_inner(Path::new(""), Some(justfile))
}

fn discover_inner(root: &Path, explicit: Option<&Path>) -> Result<DiscoveryResult> {
    ensure_just_on_path()?;
    let paths = match explicit {
        Some(p) => vec![walk::absolutize(p.to_path_buf())],
        None => walk::walk_justfiles(root)?,
    };

    let mut justfiles = Vec::new();
    let mut errors = Vec::new();
    for path in paths {
        match dump_and_parse(&path) {
            Ok(jf) => justfiles.push(jf),
            Err(e) => errors.push((path, e.to_string())),
        }
    }

    justfiles.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(DiscoveryResult { justfiles, errors })
}
```

and replace it with:

```rust
pub fn discover(opts: DiscoverOptions) -> Result<DiscoveryResult> {
    ensure_just_on_path()?;

    let roots = walk_roots(&opts);

    let mut paths: Vec<PathBuf> = Vec::new();
    for root in &roots {
        let walked = walk::walk_justfiles(root)?;
        for p in walked {
            if !paths.iter().any(|existing| existing == &p) {
                paths.push(p);
            }
        }
    }

    let pinned = opts
        .justfile
        .map(|p| walk::absolutize(p.to_path_buf()));
    if let Some(pin) = &pinned {
        if !paths.iter().any(|p| p == pin) {
            paths.push(pin.clone());
        }
    }

    paths.sort();

    let mut justfiles = Vec::new();
    let mut errors = Vec::new();
    for path in paths {
        match dump_and_parse(&path) {
            Ok(jf) => justfiles.push(jf),
            Err(e) => errors.push((path, e.to_string())),
        }
    }

    let active_index = match &pinned {
        Some(pin) => justfiles
            .iter()
            .position(|j| &j.path == pin)
            .unwrap_or(0),
        None => 0,
    };

    Ok(DiscoveryResult {
        justfiles,
        errors,
        active_index,
    })
}
```

- [ ] **Step 2: Compile**

Run: `cargo check`
Expected: still one error — the `lib.rs` call site is now doubly broken (wrong arg shape and uses removed names). Fix lands in Task 4.

(Do not commit — Task 4 produces the next commit so the tree is buildable again.)

---

### Task 4: Update `lib.rs` to call the new discovery API

**Files:**
- Modify: `src/lib.rs:60-83`

- [ ] **Step 1: Rewrite the discovery + app construction block**

In `src/lib.rs`, replace the block:

```rust
async fn async_main(cli: Cli, cfg: Config) -> anyhow::Result<()> {
    let disc = match cli.justfile.as_deref() {
        Some(jf) => discovery::discover_explicit(jf),
        None => discovery::discover(&cli.path),
    };
    let disc = match disc {
        Ok(d) => d,
        Err(e @ crate::Error::JustNotFound) => {
            eprintln!("{e}");
            std::process::exit(2);
        }
        Err(e) => return Err(e.into()),
    };
    let _ =
        crate::session::retention::prune_sessions(&cfg.sessions_log_dir, cfg.session_log_retention);
    let theme = theme::registry::resolve(&cfg.theme_name);
    let app = app::App::new(
        disc.justfiles,
        disc.errors,
        cfg.default_split_ratio,
        theme,
        cfg.theme_name.clone(),
        cfg.icon_style,
    );
    app::event_loop::run(app, cfg).await
}
```

with:

```rust
async fn async_main(cli: Cli, cfg: Config) -> anyhow::Result<()> {
    let disc = discovery::discover(discovery::DiscoverOptions {
        path: cli.path.as_deref(),
        justfile: cli.justfile.as_deref(),
    });
    let disc = match disc {
        Ok(d) => d,
        Err(e @ crate::Error::JustNotFound) => {
            eprintln!("{e}");
            std::process::exit(2);
        }
        Err(e) => return Err(e.into()),
    };
    let _ =
        crate::session::retention::prune_sessions(&cfg.sessions_log_dir, cfg.session_log_retention);
    let theme = theme::registry::resolve(&cfg.theme_name);
    let mut app = app::App::new(
        disc.justfiles,
        disc.errors,
        cfg.default_split_ratio,
        theme,
        cfg.theme_name.clone(),
        cfg.icon_style,
    );
    app.active_justfile = disc.active_index;
    app::event_loop::run(app, cfg).await
}
```

- [ ] **Step 2: Build and run the lib unit tests**

Run: `cargo build`
Expected: clean build.

Run: `cargo test --lib`
Expected: every existing lib test still passes, plus the 5 new `discovery::tests::walk_roots_*` tests pass.

- [ ] **Step 3: Commit**

```bash
git add src/cli.rs src/discovery/mod.rs src/lib.rs
git commit -m "$(cat <<'EOF'
feat(discovery): always walk, with optional --justfile pin

Replace discover / discover_explicit with a single discover(opts) that
always walks. When --justfile FILE is set, the parent of FILE is added
to the walk roots, FILE itself is force-included in the result (in case
it's gitignored or outside any walked tree), and DiscoveryResult exposes
an active_index pointing at FILE.

CLI: --path is now Option<PathBuf>; lib.rs distinguishes "user omitted
PATH" from an explicit ".".
EOF
)"
```

---

### Task 5: Update integration tests against the new API

**Files:**
- Modify: `tests/discovery_integration.rs`

- [ ] **Step 1: Replace the file contents**

Overwrite `tests/discovery_integration.rs` with the following. The existing two `discover` tests are migrated; the `discover_explicit` test is replaced with a richer set of cases for the new behavior.

```rust
use lazyjust::discovery::{discover, DiscoverOptions};
use std::path::PathBuf;

fn fixture_root() -> PathBuf {
    PathBuf::from("tests/fixtures/tree")
}

#[test]
fn discover_tree_returns_two_justfiles_with_recipes() {
    let root = fixture_root();
    let result = discover(DiscoverOptions {
        path: Some(&root),
        justfile: None,
    })
    .unwrap();
    assert_eq!(result.justfiles.len(), 2);
    assert!(result.errors.is_empty());

    let root_abs = std::path::absolute(&root).unwrap();
    let root_jf = result
        .justfiles
        .iter()
        .find(|j| j.path == root_abs.join("justfile"))
        .unwrap();
    assert!(root_jf.recipes.iter().any(|r| r.name == "build"));
    assert_eq!(result.active_index, 0);
}

#[test]
fn discover_returns_absolute_paths_even_for_relative_root() {
    let root = fixture_root();
    let result = discover(DiscoverOptions {
        path: Some(&root),
        justfile: None,
    })
    .unwrap();

    for jf in &result.justfiles {
        assert!(
            jf.path.is_absolute(),
            "expected absolute path, got: {}",
            jf.path.display()
        );
    }
}

#[test]
fn justfile_only_walks_parent_and_pins_active() {
    let pin = fixture_root().join("sub").join("justfile");
    let result = discover(DiscoverOptions {
        path: None,
        justfile: Some(&pin),
    })
    .unwrap();

    let pin_abs = std::path::absolute(&pin).unwrap();

    // The walk root is `tests/fixtures/tree/sub`, which only contains the
    // sub/justfile — no siblings. So the result is exactly one entry.
    assert_eq!(result.justfiles.len(), 1, "got: {:?}", result.justfiles);
    assert_eq!(result.justfiles[0].path, pin_abs);
    assert_eq!(result.active_index, 0);
}

#[test]
fn path_plus_justfile_unions_walks_and_pins_active() {
    let path = fixture_root();
    let pin = fixture_root().join("sub").join("justfile");
    let result = discover(DiscoverOptions {
        path: Some(&path),
        justfile: Some(&pin),
    })
    .unwrap();

    // Walking `tree` finds `tree/justfile` and `tree/sub/justfile` (the
    // node_modules/* and ignored_by_gitignore/* are excluded by the
    // hardcoded skip list and `.gitignore` respectively). Walking
    // `tree/sub` finds the same `tree/sub/justfile`. After dedup we
    // expect exactly the same two justfiles.
    assert_eq!(result.justfiles.len(), 2);
    assert!(result.errors.is_empty());

    let pin_abs = std::path::absolute(&pin).unwrap();
    assert_eq!(
        result.justfiles[result.active_index].path,
        pin_abs,
        "active_index should point at the pinned justfile"
    );
}

#[test]
fn justfile_outside_walked_tree_is_force_included() {
    // The fixture's `ignored_by_gitignore/justfile` would be skipped by
    // a normal walk of `tests/fixtures/tree`. Pinning it must still
    // surface it in the result and pin it active.
    let path = fixture_root();
    let pin = fixture_root()
        .join("ignored_by_gitignore")
        .join("justfile");

    let result = discover(DiscoverOptions {
        path: Some(&path),
        justfile: Some(&pin),
    })
    .unwrap();

    let pin_abs = std::path::absolute(&pin).unwrap();
    assert!(
        result.justfiles.iter().any(|j| j.path == pin_abs),
        "expected gitignored pin to appear in result"
    );
    assert_eq!(
        result.justfiles[result.active_index].path,
        pin_abs,
        "active_index should point at the gitignored pin"
    );
}

#[test]
fn no_args_defaults_to_cwd() {
    let result = discover(DiscoverOptions::default()).unwrap();
    // We can't assert what's in cwd — just that the call succeeds and
    // active_index is in range.
    assert!(
        result.active_index <= result.justfiles.len(),
        "active_index out of range",
    );
}
```

- [ ] **Step 2: Run the integration tests**

Run: `cargo test --test discovery_integration`
Expected: 6 passed; 0 failed.

If `justfile_outside_walked_tree_is_force_included` fails because the gitignored fixture file does not parse via `just`, check:

```bash
cat tests/fixtures/tree/ignored_by_gitignore/justfile
just --justfile tests/fixtures/tree/ignored_by_gitignore/justfile --dump --dump-format=json
```

If `just` rejects the fixture, fix the fixture content (it should be a trivial recipe like `hello:\n\techo hi`) before re-running.

- [ ] **Step 3: Run the full suite**

Run: `cargo test`
Expected: clean run. The snapshot tests are unaffected by discovery changes.

- [ ] **Step 4: Commit**

```bash
git add tests/discovery_integration.rs
git commit -m "$(cat <<'EOF'
test(discovery): cover always-walk + pin behavior matrix

Migrate the two discover() tests to the new DiscoverOptions API and
add four new cases:
- justfile only walks parent + pins active
- path + justfile union the walks and pin active
- gitignored pin is force-included and pinned active
- default options walk cwd
EOF
)"
```

---

### Task 6: Update the README `--justfile` line

**Files:**
- Modify: `README.md` — the Usage code block.

- [ ] **Step 1: Open the file and find the Usage block**

The block was last edited in commit `9f544e4`. It currently reads (around lines 116–123):

```bash
lazyjust                    # recursively scan current directory for justfiles
lazyjust [path]             # recursively scan PATH for justfiles
lazyjust --justfile FILE    # load only FILE; skip subdirectory walk
lazyjust --log-level LEVEL  # log verbosity (default: warn)
lazyjust --help             # full flag reference
lazyjust --version          # print version
```

- [ ] **Step 2: Replace the `--justfile` line**

In `README.md`, change the line:

```
lazyjust --justfile FILE    # load only FILE; skip subdirectory walk
```

to:

```
lazyjust --justfile FILE    # walk FILE's directory, pin FILE as active
```

The discovery-rules paragraph that follows the code block stays as-is.

- [ ] **Step 3: Verify the diff is just one line**

Run: `git diff README.md`
Expected: a single-line change to the `--justfile` row inside the Usage code block.

- [ ] **Step 4: Commit**

```bash
git add README.md
git commit -m "$(cat <<'EOF'
docs(readme): --justfile now pins, no longer skips the walk

Reflect the discovery change: --justfile FILE walks FILE's directory
and pins FILE as the active justfile, instead of disabling the walk.
EOF
)"
```

---

## Verification

After Tasks 1–6 land:

- `cargo build --release` — succeeds.
- `cargo test` — full suite green (existing 68 + 5 new walk_roots unit tests + 4 new integration tests).
- `cargo run -- --justfile tests/fixtures/tree/sub/justfile` against the fixture — opens with `tree/sub/justfile` active and the dropdown (`d`) lists every justfile under `tree/sub`. `cargo run -- tests/fixtures/tree --justfile tests/fixtures/tree/sub/justfile` lists both `tree/justfile` and `tree/sub/justfile` with the latter pre-selected.
- `lazyjust --help` shows the flag descriptions in line with the README Usage block.

## Out of scope follow-ups

- Multi-pin (`--justfile F1 --justfile F2`).
- Async / progressive discovery.
- Per-justfile env overrides.
- Additional CLI flags (e.g. `--no-walk` to opt back into the old single-file behavior).
