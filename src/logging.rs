use crate::config::Config;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, EnvFilter};

pub fn init(cfg: &Config, level: &str) -> anyhow::Result<WorkerGuard> {
    create_private_dir_all(&cfg.state_dir)?;
    let file_appender = tracing_appender::rolling::daily(&cfg.state_dir, "lazyjust.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level));

    fmt()
        .with_env_filter(filter)
        .with_writer(non_blocking)
        .with_ansi(false)
        .init();

    Ok(guard)
}

use crate::app::types::SessionId;
use crate::error::Result;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn session_log_path(cfg: &Config, id: SessionId, recipe: &str) -> Result<PathBuf> {
    let today = format_date_today();
    let dir = cfg.sessions_log_dir.join(&today);
    create_private_dir_all(&dir)?;
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let safe_recipe: String = recipe
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    Ok(dir.join(format!("{safe_recipe}_{ts}_{id}.log")))
}

/// Session logs are raw recipe output (env dumps, tokens), so they are
/// readable by the owner only. Directories created for them are likewise
/// 0700; pre-existing directories keep their mode.
pub fn open_session_log(path: &Path) -> std::io::Result<std::fs::File> {
    let mut opts = std::fs::OpenOptions::new();
    opts.create(true).append(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut opts, 0o600);
    opts.open(path)
}

fn create_private_dir_all(path: &Path) -> std::io::Result<()> {
    let mut builder = std::fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    std::os::unix::fs::DirBuilderExt::mode(&mut builder, 0o700);
    builder.create(path)
}

/// Name of the per-day directory `session_log_path` creates. Retention
/// only ever prunes directories whose name has this shape.
pub fn is_session_day_dir(name: &str) -> bool {
    let b = name.as_bytes();
    b.len() == 10
        && b[4] == b'-'
        && b[7] == b'-'
        && b.iter()
            .enumerate()
            .all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit())
}

fn format_date_today() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let days = secs / 86_400;
    let (y, m, d) = days_to_ymd(days);
    format!("{y:04}-{m:02}-{d:02}")
}

fn days_to_ymd(mut days: i64) -> (i32, u32, u32) {
    let mut year = 1970;
    loop {
        let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
        let year_days = if leap { 366 } else { 365 };
        if days < year_days {
            break;
        }
        days -= year_days;
        year += 1;
    }
    let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
    let month_days = [
        31u32,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut m = 0;
    let mut remaining = days as u32;
    while remaining >= month_days[m] {
        remaining -= month_days[m];
        m += 1;
    }
    (year, m as u32 + 1, remaining + 1)
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    fn mode(p: &std::path::Path) -> u32 {
        std::fs::metadata(p).unwrap().permissions().mode() & 0o777
    }

    #[test]
    fn session_logs_are_owner_only() {
        let tmp = tempfile::tempdir().unwrap();
        let mut cfg = crate::config::Config::load();
        cfg.sessions_log_dir = tmp.path().join("sessions");

        let path = session_log_path(&cfg, 1, "build").unwrap();
        open_session_log(&path).unwrap();

        assert_eq!(mode(&cfg.sessions_log_dir), 0o700);
        assert_eq!(mode(path.parent().unwrap()), 0o700);
        assert_eq!(mode(&path), 0o600);
    }
}
