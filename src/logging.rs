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
