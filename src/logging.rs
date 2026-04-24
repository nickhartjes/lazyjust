use crate::config::Config;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, EnvFilter};

pub fn init(cfg: &Config, level: &str) -> anyhow::Result<WorkerGuard> {
    std::fs::create_dir_all(&cfg.state_dir)?;
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
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn session_log_path(cfg: &Config, id: SessionId, recipe: &str) -> Result<PathBuf> {
    let today = format_date_today();
    let dir = cfg.sessions_log_dir.join(&today);
    std::fs::create_dir_all(&dir)?;
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
