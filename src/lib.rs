pub mod app;
pub mod cli;
pub mod config;
pub mod discovery;
pub mod error;
pub mod input;
pub mod logging;
pub mod session;
pub mod ui;

pub use error::{Error, Result};

use clap::Parser;
use cli::Cli;
use config::Config;

pub fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let cfg = Config::load();
    let _log_guard = logging::init(&cfg, &cli.log_level)?;

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    rt.block_on(async move { async_main(cli, cfg).await })
}

async fn async_main(cli: Cli, cfg: Config) -> anyhow::Result<()> {
    let disc = discovery::discover(&cli.path)?;
    let _ =
        crate::session::retention::prune_sessions(&cfg.sessions_log_dir, cfg.session_log_retention);
    let app = app::App::new(disc.justfiles, disc.errors, cfg.default_split_ratio);
    app::event_loop::run(app, cfg).await
}
