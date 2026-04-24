# UI Redesign — Milestone 1: Config File Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the hardcoded `Config::load` with a TOML-backed config loader that reads `<config_dir>/lazyjust/config.toml` (via `dirs::config_dir()`), merges user overrides onto defaults, falls back safely on malformed input, and exposes `lazyjust config path` and `lazyjust config init` subcommands.

**Architecture:** Keep `src/config.rs` as the module root, add `src/config/` submodules (`defaults`, `paths`, `file`, `merge`). `ConfigFile` is a serde struct of all-optional sections; `merge(file, defaults) -> Config` fills missing keys. `Config::load()` resolves the platform path, reads if present, parses, merges, and returns; any IO or parse error logs a warning and falls back to `defaults()`. Two new CLI subcommands are added via a `Commands` enum on `Cli`. No UI change.

**Tech Stack:** Rust, existing `lazyjust` crate, adds `toml = "0.8"`. `toml_edit` is not needed here — `config init` writes a static template via `include_str!`, and no live-edit writes exist yet (those land in M2). Tests use existing `insta`, `tempfile`, `rstest`, `tracing-subscriber`.

**Spec:** `docs/superpowers/specs/2026-04-24-ui-redesign-design.md` (Milestone 1 section).

---

## File Structure

| File | Action | Responsibility |
|---|---|---|
| `Cargo.toml` | Modify | Add `toml = "0.8"` dependency. |
| `src/config.rs` | Modify | Module root. Declares submodules. Re-exports `Config`. Hosts `Config::load()` (now reads the file). |
| `src/config/defaults.rs` | Create | `pub fn defaults() -> Config` — moves today's hardcoded values. |
| `src/config/paths.rs` | Create | `pub fn config_file_path() -> PathBuf`, `pub fn user_themes_dir() -> PathBuf`. Resolves via `dirs::config_dir()`. Testable via `LAZYJUST_CONFIG_DIR` env override. |
| `src/config/file.rs` | Create | Serde struct `ConfigFile` with `Option<PathsSection>`, `Option<LoggingSection>`, `Option<EngineSection>`. `fn read(&Path) -> Result<Option<ConfigFile>, ConfigError>`. |
| `src/config/merge.rs` | Create | `pub fn merge(file: ConfigFile, base: Config) -> Config`. |
| `src/config/template.rs` | Create | `pub const CONFIG_TEMPLATE: &str = include_str!("../../assets/config-template.toml");` |
| `assets/config-template.toml` | Create | Commented example config, referenced by `config init`. |
| `src/cli.rs` | Modify | Add `Commands` enum with `Config { action }` variant; `Cli` gets optional `#[command(subcommand)]`. |
| `src/lib.rs` | Modify | Branch on `cli.command` before starting the UI. `config path` and `config init` are handled and the process exits. |
| `tests/config_loader.rs` | Create | Integration tests for default / partial / malformed load, `config init`, `config path`. |

---

## Task 1: Add `toml` dependency

**Files:**
- Modify: `Cargo.toml`

- [ ] **Step 1: Add `toml` to dependencies**

In `Cargo.toml`, under `[dependencies]` (alphabetical order; insert after `tokio`):

```toml
toml = "0.8"
```

Final `[dependencies]` block should contain the line `toml = "0.8"` between `tokio = ...` and `tracing = "0.1"`.

- [ ] **Step 2: Verify build**

Run: `cargo build`
Expected: builds successfully, `toml 0.8.x` appears in `Cargo.lock`.

- [ ] **Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock
git commit -m "build: add toml 0.8 dependency for config loader"
```

---

## Task 2: Extract `defaults()` and set up `src/config/` submodule structure

**Files:**
- Modify: `src/config.rs`
- Create: `src/config/defaults.rs`

- [ ] **Step 1: Create `src/config/defaults.rs`**

```rust
use super::Config;
use std::path::PathBuf;
use std::time::Duration;

pub fn defaults() -> Config {
    let state_dir = dirs::state_dir()
        .or_else(dirs::data_local_dir)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("lazyjust");
    let sessions_log_dir = state_dir.join("sessions");

    Config {
        state_dir,
        sessions_log_dir,
        session_log_size_cap: 10 * 1024 * 1024,
        session_log_retention: Duration::from_secs(7 * 24 * 3600),
        default_split_ratio: 0.30,
        min_left_cols: 20,
        min_right_cols: 40,
        terminal_floor_cols: 40,
        terminal_floor_rows: 10,
        render_throttle: Duration::from_millis(16),
        tick_interval: Duration::from_millis(250),
    }
}
```

- [ ] **Step 2: Replace `src/config.rs` body**

Replace the current `src/config.rs` with:

```rust
use std::path::PathBuf;
use std::time::Duration;

mod defaults;

#[derive(Debug, Clone)]
pub struct Config {
    /// Root directory for persistent state (logs, session output).
    /// On macOS: `~/Library/Application Support/lazyjust/`. On Linux: `$XDG_STATE_HOME/lazyjust/`.
    /// The daily-rotated app log lives here as `lazyjust.log.YYYY-MM-DD`.
    pub state_dir: PathBuf,
    /// Directory for per-session raw PTY logs. Populated in T18.
    pub sessions_log_dir: PathBuf,
    /// Max bytes written per session log file before rotation. Default 10 MiB.
    pub session_log_size_cap: u64,
    /// Retention for per-session log directories. Directories older than this are pruned on startup (T25).
    pub session_log_retention: Duration,
    /// Default left-pane width as a fraction of total width.
    pub default_split_ratio: f32,
    pub min_left_cols: u16,
    pub min_right_cols: u16,
    /// Below this size, the UI renders a "Terminal too small" screen (T26).
    pub terminal_floor_cols: u16,
    pub terminal_floor_rows: u16,
    /// Minimum wall time between render frames in the event loop.
    pub render_throttle: Duration,
    /// Interval for the event loop tick that polls child exit status.
    pub tick_interval: Duration,
}

impl Config {
    pub fn load() -> Self {
        defaults::defaults()
    }
}
```

- [ ] **Step 3: Run tests**

Run: `cargo test --all`
Expected: all existing tests pass. No behavior change.

- [ ] **Step 4: Commit**

```bash
git add src/config.rs src/config/defaults.rs
git commit -m "refactor(config): extract defaults() into submodule"
```

---

## Task 3: Platform path resolver

**Files:**
- Create: `src/config/paths.rs`
- Modify: `src/config.rs` (declare `mod paths`)

- [ ] **Step 1: Write failing test in `src/config/paths.rs`**

```rust
use std::path::PathBuf;

/// Override for tests and advanced users.
pub const OVERRIDE_ENV: &str = "LAZYJUST_CONFIG_DIR";

pub fn config_file_path() -> PathBuf {
    config_root().join("config.toml")
}

pub fn user_themes_dir() -> PathBuf {
    config_root().join("themes")
}

fn config_root() -> PathBuf {
    if let Ok(v) = std::env::var(OVERRIDE_ENV) {
        return PathBuf::from(v);
    }
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("lazyjust")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn env_override_wins() {
        let tmp = tempfile::tempdir().unwrap();
        env::set_var(OVERRIDE_ENV, tmp.path());
        assert_eq!(config_file_path(), tmp.path().join("config.toml"));
        assert_eq!(user_themes_dir(), tmp.path().join("themes"));
        env::remove_var(OVERRIDE_ENV);
    }

    #[test]
    fn falls_back_when_dirs_unavailable() {
        // dirs::config_dir() returns Some on every supported platform in CI,
        // so this test just exercises the default path ending in "lazyjust".
        env::remove_var(OVERRIDE_ENV);
        let p = config_file_path();
        assert_eq!(p.file_name().unwrap(), "config.toml");
        assert_eq!(p.parent().unwrap().file_name().unwrap(), "lazyjust");
    }
}
```

- [ ] **Step 2: Register `paths` submodule**

In `src/config.rs`, after the existing `mod defaults;` line, add:

```rust
pub mod paths;
```

- [ ] **Step 3: Run tests**

Run: `cargo test --lib config::paths::tests`
Expected: both tests pass.

- [ ] **Step 4: Commit**

```bash
git add src/config.rs src/config/paths.rs
git commit -m "feat(config): add platform path resolver with env override"
```

---

## Task 4: `ConfigFile` serde struct with optional sections

**Files:**
- Create: `src/config/file.rs`
- Modify: `src/config.rs` (declare `mod file`, add `ConfigError`)

- [ ] **Step 1: Add `ConfigError` type to `src/config.rs`**

In `src/config.rs`, after the `use` statements, add:

```rust
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("config file IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("config file parse error: {0}")]
    Parse(#[from] toml::de::Error),
}
```

- [ ] **Step 2: Create `src/config/file.rs`**

```rust
use super::ConfigError;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Default, Deserialize)]
pub struct ConfigFile {
    // Unknown top-level keys are silently ignored (serde default) so
    // users can keep `[ui]` / `[keys]` blocks now; those become active
    // in later milestones without breaking this loader.
    pub paths: Option<PathsSection>,
    pub logging: Option<LoggingSection>,
    pub engine: Option<EngineSection>,
}

#[derive(Debug, Default, Deserialize)]
pub struct PathsSection {
    pub state_dir: Option<String>,
    pub sessions_log_dir: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct LoggingSection {
    pub session_log_size_cap_mb: Option<u64>,
    pub session_log_retention_days: Option<u64>,
}

#[derive(Debug, Default, Deserialize)]
pub struct EngineSection {
    pub render_throttle_ms: Option<u64>,
    pub tick_interval_ms: Option<u64>,
}

impl ConfigFile {
    /// Reads and parses the file. Returns `Ok(None)` if the file does not exist.
    pub fn read(path: &Path) -> Result<Option<ConfigFile>, ConfigError> {
        match std::fs::read_to_string(path) {
            Ok(s) => Ok(Some(toml::from_str(&s)?)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(ConfigError::Io(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(contents: &str) -> tempfile::NamedTempFile {
        let f = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(f.path(), contents).unwrap();
        f
    }

    #[test]
    fn missing_file_returns_none() {
        let p = std::path::Path::new("/definitely/does/not/exist/lazyjust.toml");
        assert!(matches!(ConfigFile::read(p), Ok(None)));
    }

    #[test]
    fn empty_file_parses() {
        let f = write("");
        let cf = ConfigFile::read(f.path()).unwrap().unwrap();
        assert!(cf.paths.is_none());
        assert!(cf.logging.is_none());
        assert!(cf.engine.is_none());
    }

    #[test]
    fn partial_file_parses() {
        let f = write(
            r#"
            [engine]
            render_throttle_ms = 8
            "#,
        );
        let cf = ConfigFile::read(f.path()).unwrap().unwrap();
        assert_eq!(cf.engine.unwrap().render_throttle_ms, Some(8));
        assert!(cf.paths.is_none());
        assert!(cf.logging.is_none());
    }

    #[test]
    fn unknown_sections_ignored() {
        let f = write(
            r#"
            [ui]
            theme = "tokyo-night"

            [keys]
            quit = "q"
            "#,
        );
        // ui/keys are not yet defined in ConfigFile; serde silently ignores
        // unknown keys by default.
        let cf = ConfigFile::read(f.path()).unwrap().unwrap();
        assert!(cf.paths.is_none());
    }

    #[test]
    fn malformed_file_returns_parse_error() {
        let f = write("this is = = not valid toml [[");
        let err = ConfigFile::read(f.path()).unwrap_err();
        assert!(matches!(err, ConfigError::Parse(_)));
    }
}
```

- [ ] **Step 3: Register submodule in `src/config.rs`**

After `pub mod paths;`, add:

```rust
mod file;
```

- [ ] **Step 4: Run tests**

Run: `cargo test --lib config::file::tests`
Expected: all five tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/config.rs src/config/file.rs
git commit -m "feat(config): add ConfigFile serde struct with optional sections"
```

---

## Task 5: Merge logic

**Files:**
- Create: `src/config/merge.rs`
- Modify: `src/config.rs` (declare `mod merge`)

- [ ] **Step 1: Create `src/config/merge.rs`**

```rust
use super::file::ConfigFile;
use super::Config;
use std::path::PathBuf;
use std::time::Duration;

/// Overlay a parsed file onto a base `Config`, filling missing values
/// from the base. The returned `Config` is ready for use.
pub fn merge(file: ConfigFile, base: Config) -> Config {
    let mut out = base;

    if let Some(p) = file.paths {
        if let Some(d) = p.state_dir {
            out.state_dir = PathBuf::from(d);
        }
        if let Some(d) = p.sessions_log_dir {
            out.sessions_log_dir = PathBuf::from(d);
        }
    }

    if let Some(l) = file.logging {
        if let Some(mb) = l.session_log_size_cap_mb {
            out.session_log_size_cap = mb.saturating_mul(1024 * 1024);
        }
        if let Some(days) = l.session_log_retention_days {
            out.session_log_retention = Duration::from_secs(days.saturating_mul(24 * 3600));
        }
    }

    if let Some(e) = file.engine {
        if let Some(ms) = e.render_throttle_ms {
            out.render_throttle = Duration::from_millis(ms);
        }
        if let Some(ms) = e.tick_interval_ms {
            out.tick_interval = Duration::from_millis(ms);
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::defaults::defaults;
    use crate::config::file::{ConfigFile, EngineSection, LoggingSection, PathsSection};

    #[test]
    fn empty_file_yields_defaults() {
        let d = defaults();
        let expected_throttle = d.render_throttle;
        let merged = merge(ConfigFile::default(), d);
        assert_eq!(merged.render_throttle, expected_throttle);
    }

    #[test]
    fn engine_overrides_apply() {
        let file = ConfigFile {
            engine: Some(EngineSection {
                render_throttle_ms: Some(8),
                tick_interval_ms: Some(500),
            }),
            ..Default::default()
        };
        let merged = merge(file, defaults());
        assert_eq!(merged.render_throttle, Duration::from_millis(8));
        assert_eq!(merged.tick_interval, Duration::from_millis(500));
    }

    #[test]
    fn logging_mb_converts_to_bytes() {
        let file = ConfigFile {
            logging: Some(LoggingSection {
                session_log_size_cap_mb: Some(5),
                session_log_retention_days: Some(2),
            }),
            ..Default::default()
        };
        let merged = merge(file, defaults());
        assert_eq!(merged.session_log_size_cap, 5 * 1024 * 1024);
        assert_eq!(merged.session_log_retention, Duration::from_secs(2 * 24 * 3600));
    }

    #[test]
    fn paths_override_apply() {
        let file = ConfigFile {
            paths: Some(PathsSection {
                state_dir: Some("/tmp/lj-test".into()),
                sessions_log_dir: None,
            }),
            ..Default::default()
        };
        let merged = merge(file, defaults());
        assert_eq!(merged.state_dir, PathBuf::from("/tmp/lj-test"));
        // sessions_log_dir untouched when not specified
        assert_ne!(merged.sessions_log_dir, PathBuf::from("/tmp/lj-test"));
    }
}
```

- [ ] **Step 2: Register submodule and mark `file` + `defaults` visible to `merge`**

In `src/config.rs`, adjust module declarations so they look like:

```rust
pub mod paths;

mod defaults;
mod file;
mod merge;
```

In `src/config/defaults.rs`, change the fn signature from `pub fn` to stay `pub fn` (already correct). No other change.

In `src/config/file.rs`, no change — items are already `pub` within the module.

- [ ] **Step 3: Run tests**

Run: `cargo test --lib config::merge::tests`
Expected: all four tests pass.

- [ ] **Step 4: Commit**

```bash
git add src/config.rs src/config/merge.rs
git commit -m "feat(config): merge ConfigFile overrides onto defaults"
```

---

## Task 6: Wire `Config::load()` to read, parse, merge

**Files:**
- Modify: `src/config.rs`
- Create: `tests/config_loader.rs`

- [ ] **Step 1: Replace `Config::load` body in `src/config.rs`**

Replace the existing `impl Config { ... }` block with:

```rust
impl Config {
    /// Load config from the platform path, merged with defaults.
    /// Missing file → all defaults. Malformed file → warning logged, all defaults.
    pub fn load() -> Self {
        let base = defaults::defaults();
        let path = paths::config_file_path();
        match file::ConfigFile::read(&path) {
            Ok(Some(cf)) => merge::merge(cf, base),
            Ok(None) => base,
            Err(e) => {
                tracing::warn!(
                    target: "lazyjust::config",
                    path = %path.display(),
                    error = %e,
                    "failed to load config, using defaults",
                );
                base
            }
        }
    }
}
```

- [ ] **Step 2: Write integration test**

Create `tests/config_loader.rs`:

```rust
use lazyjust::config::Config;
use std::time::Duration;

// Serialize these tests: they all mutate the same env var.
// Not fancy enough to warrant serial_test crate; each test sets+resets.

fn with_config_dir<T>(contents: Option<&str>, body: impl FnOnce() -> T) -> T {
    let tmp = tempfile::tempdir().unwrap();
    if let Some(c) = contents {
        std::fs::write(tmp.path().join("config.toml"), c).unwrap();
    }
    std::env::set_var("LAZYJUST_CONFIG_DIR", tmp.path());
    let out = body();
    std::env::remove_var("LAZYJUST_CONFIG_DIR");
    out
}

#[test]
fn no_file_returns_defaults() {
    let cfg = with_config_dir(None, Config::load);
    assert_eq!(cfg.render_throttle, Duration::from_millis(16));
    assert_eq!(cfg.tick_interval, Duration::from_millis(250));
}

#[test]
fn partial_file_overrides_only_specified_keys() {
    let cfg = with_config_dir(
        Some("[engine]\nrender_throttle_ms = 8\n"),
        Config::load,
    );
    assert_eq!(cfg.render_throttle, Duration::from_millis(8));
    // tick_interval stayed at default
    assert_eq!(cfg.tick_interval, Duration::from_millis(250));
}

#[test]
fn malformed_file_falls_back_to_defaults() {
    let cfg = with_config_dir(Some("this = = = not valid toml"), Config::load);
    assert_eq!(cfg.render_throttle, Duration::from_millis(16));
    // Note: the warning is emitted via tracing; we don't capture it here.
    // tests/logging_integration.rs (existing) covers tracing capture patterns
    // if future tightening is needed.
}
```

- [ ] **Step 3: Run tests**

Run: `cargo test --test config_loader`
Expected: three tests pass.

Run: `cargo test --all`
Expected: everything still green.

- [ ] **Step 4: Commit**

```bash
git add src/config.rs tests/config_loader.rs
git commit -m "feat(config): load + merge TOML config with malformed fallback"
```

---

## Task 7: CLI `Commands` subcommand skeleton

**Files:**
- Modify: `src/cli.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: Rewrite `src/cli.rs`**

Replace the current `src/cli.rs` with:

```rust
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "lazyjust", about = "Terminal UI for just", version)]
pub struct Cli {
    /// Project root to scan (defaults to current directory).
    #[arg(value_name = "PATH", default_value = ".")]
    pub path: PathBuf,

    /// Specific justfile to use as root (overrides depth-0 auto-pick).
    #[arg(long = "justfile", value_name = "FILE")]
    pub justfile: Option<PathBuf>,

    /// Log verbosity.
    #[arg(long = "log-level", default_value = "warn")]
    pub log_level: String,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Inspect or initialize the config file.
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand, Debug)]
pub enum ConfigAction {
    /// Print the path to the config file.
    Path,
    /// Write a commented example config to the config path.
    /// Refuses to overwrite an existing file.
    Init,
}
```

- [ ] **Step 2: Branch in `src/lib.rs`**

In `src/lib.rs`, find the `run()` function and the line `let cli = Cli::parse();` (around line 18). After `Cli::parse()`, handle the subcommand before logging init:

```rust
let cli = Cli::parse();

if let Some(cmd) = cli.command.as_ref() {
    return handle_subcommand(cmd);
}

let cfg = Config::load();
// ...rest unchanged
```

Then add the `handle_subcommand` function below `run()`:

```rust
fn handle_subcommand(cmd: &cli::Commands) -> anyhow::Result<()> {
    match cmd {
        cli::Commands::Config { action } => match action {
            cli::ConfigAction::Path => {
                println!("{}", config::paths::config_file_path().display());
                Ok(())
            }
            cli::ConfigAction::Init => {
                // Implemented in Task 9.
                anyhow::bail!("not yet implemented");
            }
        },
    }
}
```

- [ ] **Step 3: Run `--help` smoke check**

Run: `cargo run -- --help`
Expected: help text includes `Commands: config ...`.

Run: `cargo run -- config --help`
Expected: shows `path` and `init` subcommands.

- [ ] **Step 4: Commit**

```bash
git add src/cli.rs src/lib.rs
git commit -m "feat(cli): add 'config' subcommand with path/init actions"
```

---

## Task 8: `config path` subcommand end-to-end

**Files:**
- Create: `tests/cli_config_path.rs`

- [ ] **Step 1: Write integration test**

```rust
use std::process::Command;

fn cargo_bin() -> String {
    env!("CARGO_BIN_EXE_lazyjust").to_string()
}

#[test]
fn prints_config_file_path_with_env_override() {
    let tmp = tempfile::tempdir().unwrap();
    let out = Command::new(cargo_bin())
        .env("LAZYJUST_CONFIG_DIR", tmp.path())
        .args(["config", "path"])
        .output()
        .unwrap();
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8(out.stdout).unwrap();
    let expected = tmp.path().join("config.toml");
    assert_eq!(stdout.trim(), expected.to_string_lossy());
}
```

- [ ] **Step 2: Run**

Run: `cargo test --test cli_config_path`
Expected: test passes.

- [ ] **Step 3: Commit**

```bash
git add tests/cli_config_path.rs
git commit -m "test(cli): 'config path' prints resolved path with env override"
```

---

## Task 9: `config init` writes commented template

**Files:**
- Create: `assets/config-template.toml`
- Create: `src/config/template.rs`
- Modify: `src/config.rs` (declare `mod template`)
- Modify: `src/lib.rs` (implement the `Init` branch)
- Create: `tests/cli_config_init.rs`

- [ ] **Step 1: Create the template file**

Create `assets/config-template.toml`:

```toml
# lazyjust config file
#
# Every key is optional. Delete any section or key to use the built-in
# default. Unknown keys are ignored so you can keep blocks for features
# that land in later releases.

[ui]
# Built-in theme name or a filename (without .toml) under
# <config_dir>/lazyjust/themes/. Applied in M2 of the UI redesign.
theme = "tokyo-night"
# Fraction of terminal width given to the recipe list.
split_ratio = 0.30
# Render "→ dep1 · dep2" under recipes that have dependencies.
show_inline_deps = true
# Glyph set for list indicators. "round" | "ascii" | "none".
icon_style = "round"

[keys]
# Single chars, named keys ("enter", "esc", "tab", "pgup", "pgdn",
# "home", "end", "up", "down", "left", "right", "space"), or chords
# ("ctrl+c", "alt+t"). Case-insensitive.
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

[paths]
# Override the state directory (logs, session output). Default:
# platform-specific via dirs::state_dir().
# state_dir = "/absolute/path"
# sessions_log_dir = "/absolute/path"

[logging]
# Per-session log file size cap in MiB before rotation.
session_log_size_cap_mb = 10
# Days to keep per-session log directories on startup.
session_log_retention_days = 7

[engine]
# Minimum wall time between render frames (ms).
render_throttle_ms = 16
# Event-loop tick interval for polling child exit status (ms).
tick_interval_ms = 250
```

- [ ] **Step 2: Create `src/config/template.rs`**

```rust
pub const CONFIG_TEMPLATE: &str = include_str!("../../assets/config-template.toml");
```

- [ ] **Step 3: Register submodule and implement the `Init` branch**

In `src/config.rs`, extend the submodule declarations so they include:

```rust
pub mod paths;
pub mod template;

mod defaults;
mod file;
mod merge;
```

In `src/lib.rs`, replace the `ConfigAction::Init` arm body with:

```rust
cli::ConfigAction::Init => {
    let path = config::paths::config_file_path();
    if path.exists() {
        anyhow::bail!(
            "config file already exists at {}; refusing to overwrite",
            path.display()
        );
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, config::template::CONFIG_TEMPLATE)?;
    println!("wrote {}", path.display());
    Ok(())
}
```

- [ ] **Step 4: Write integration test**

Create `tests/cli_config_init.rs`:

```rust
use std::process::Command;

fn cargo_bin() -> String {
    env!("CARGO_BIN_EXE_lazyjust").to_string()
}

#[test]
fn init_writes_template_when_missing() {
    let tmp = tempfile::tempdir().unwrap();
    let out = Command::new(cargo_bin())
        .env("LAZYJUST_CONFIG_DIR", tmp.path())
        .args(["config", "init"])
        .output()
        .unwrap();
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));

    let path = tmp.path().join("config.toml");
    assert!(path.exists());
    let contents = std::fs::read_to_string(&path).unwrap();
    assert!(contents.contains("[ui]"));
    assert!(contents.contains("theme = \"tokyo-night\""));
}

#[test]
fn init_refuses_to_overwrite() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("config.toml"), "existing = true\n").unwrap();

    let out = Command::new(cargo_bin())
        .env("LAZYJUST_CONFIG_DIR", tmp.path())
        .args(["config", "init"])
        .output()
        .unwrap();
    assert!(!out.status.success());
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("refusing to overwrite"));

    // Existing contents preserved.
    let preserved = std::fs::read_to_string(tmp.path().join("config.toml")).unwrap();
    assert_eq!(preserved, "existing = true\n");
}

#[test]
fn init_then_load_round_trips() {
    let tmp = tempfile::tempdir().unwrap();
    let out = Command::new(cargo_bin())
        .env("LAZYJUST_CONFIG_DIR", tmp.path())
        .args(["config", "init"])
        .output()
        .unwrap();
    assert!(out.status.success());

    // Load from inside the test process using the same env override.
    std::env::set_var("LAZYJUST_CONFIG_DIR", tmp.path());
    let cfg = lazyjust::config::Config::load();
    std::env::remove_var("LAZYJUST_CONFIG_DIR");

    // Template values match defaults, so load succeeds and key values equal defaults.
    assert_eq!(cfg.render_throttle, std::time::Duration::from_millis(16));
    assert_eq!(cfg.tick_interval, std::time::Duration::from_millis(250));
    assert_eq!(cfg.session_log_size_cap, 10 * 1024 * 1024);
}
```

- [ ] **Step 5: Run**

Run: `cargo test --test cli_config_init`
Expected: all three tests pass.

Run: `cargo test --all`
Expected: full suite green.

- [ ] **Step 6: Commit**

```bash
git add assets/config-template.toml src/config.rs src/config/template.rs src/lib.rs tests/cli_config_init.rs
git commit -m "feat(cli): 'config init' writes commented template, refuses overwrite"
```

---

## Task 10: Template validates against loader

Confidence check — the committed template must parse cleanly through the real loader. Task 9's `init_then_load_round_trips` covers the happy path; this task adds a direct static test so a malformed template is caught at `cargo test` even if the binary stops shipping `config init`.

**Files:**
- Modify: `src/config/template.rs`

- [ ] **Step 1: Add self-check test to `src/config/template.rs`**

Append to `src/config/template.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::CONFIG_TEMPLATE;
    use crate::config::file::ConfigFile;

    #[test]
    fn template_parses_as_config_file() {
        let parsed: ConfigFile = toml::from_str(CONFIG_TEMPLATE)
            .expect("committed template must parse");
        // We specifically want [engine] to come through so the template stays
        // representative of real overrides.
        let engine = parsed.engine.expect("template should define [engine]");
        assert_eq!(engine.render_throttle_ms, Some(16));
        assert_eq!(engine.tick_interval_ms, Some(250));
    }
}
```

- [ ] **Step 2: Make `file` visible to `template` tests**

`file` is already declared as `mod file;` (private). The test above needs access via `crate::config::file::ConfigFile`. To allow this without making the whole `file` module public, change `src/config.rs`'s declaration to:

```rust
pub(crate) mod file;
```

(Keep `defaults` and `merge` private.)

- [ ] **Step 3: Run**

Run: `cargo test --lib config::template`
Expected: test passes.

Run: `cargo test --all`
Expected: full suite green.

- [ ] **Step 4: Commit**

```bash
git add src/config.rs src/config/template.rs
git commit -m "test(config): template round-trips through the real loader"
```

---

## Task 11: Manual verification

Not a code task — exercise the binary to confirm behavior end-to-end before opening a PR.

- [ ] **Step 1: Delete any existing config, confirm defaults**

Run:

```bash
rm -f "$(cargo run --quiet -- config path)"
cargo run --quiet -- --help
```

Expected: no crash; `--help` text mentions the new `config` subcommand.

- [ ] **Step 2: `config path`**

Run: `cargo run --quiet -- config path`
Expected: prints a path ending in `/lazyjust/config.toml` under your platform's config dir.

- [ ] **Step 3: `config init`**

Run: `cargo run --quiet -- config init`
Expected: prints `wrote <path>`. Path now exists.

- [ ] **Step 4: Inspect + re-init**

```bash
cat "$(cargo run --quiet -- config path)" | head
cargo run --quiet -- config init
```

Expected: file contains the commented template. Second `init` exits non-zero with "refusing to overwrite".

- [ ] **Step 5: Edit + launch**

Edit `$(cargo run --quiet -- config path)` and set `render_throttle_ms = 8`. Launch the UI (`cargo run`) and confirm it still runs (no visible change — M1 doesn't change UI).

- [ ] **Step 6: Break the file, confirm fallback**

```bash
echo '{{{ broken' >> "$(cargo run --quiet -- config path)"
cargo run -- --log-level=warn
```

Expected: UI launches. Check the log file (`~/Library/Application Support/lazyjust/lazyjust.log.YYYY-MM-DD` on macOS) for a `failed to load config, using defaults` warning.

- [ ] **Step 7: Clean up**

Delete the test config file so it doesn't leak into future manual runs:

```bash
rm -f "$(cargo run --quiet -- config path)"
```

---

## Exit criteria for Milestone 1

All checkboxes above ticked.

- `cargo test --all` green.
- `cargo clippy --all-targets --all-features -- -D warnings` clean.
- `cargo fmt --all -- --check` clean.
- Binary ships `config path` and `config init` subcommands.
- `Config::load()` reads TOML, merges, falls back on error.
- No UI change. No theme system yet (M2).
