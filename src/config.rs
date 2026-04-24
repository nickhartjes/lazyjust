use std::path::PathBuf;
use std::time::Duration;

mod defaults;
pub mod paths;

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
