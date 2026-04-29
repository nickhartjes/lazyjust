use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("config file IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("config file parse error: {0}")]
    Parse(#[from] toml::de::Error),
}

mod defaults;
pub(crate) mod file;
mod merge;
pub mod paths;
pub mod template;
pub mod writer;

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
    /// Resolved theme name — matches a built-in or user theme file.
    pub theme_name: String,
    /// Glyph set for list indicators. Values: `round` (default), `ascii`, or `none`.
    pub icon_style: crate::ui::icon_style::IconStyle,
}

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
                    path = %path.display(),
                    error = %e,
                    "failed to load config, using defaults",
                );
                base
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[tracing_test::traced_test]
    fn malformed_file_falls_back_to_defaults_with_warning() {
        let _lock = paths::env_lock().lock().unwrap_or_else(|e| e.into_inner());

        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("config.toml"), "this = = = not valid toml").unwrap();
        std::env::set_var("LAZYJUST_CONFIG_DIR", tmp.path());

        let cfg = Config::load();

        std::env::remove_var("LAZYJUST_CONFIG_DIR");

        assert_eq!(cfg.render_throttle, Duration::from_millis(16));
        assert!(
            logs_contain("failed to load config, using defaults"),
            "expected tracing::warn! about config load failure",
        );
    }
}
