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
