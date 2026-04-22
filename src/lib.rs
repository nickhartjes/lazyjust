pub mod app;
pub mod cli;
pub mod config;
pub mod discovery;
pub mod error;
pub mod input;
pub mod logging;

pub use error::{Error, Result};

use clap::Parser;
use cli::Cli;
use config::Config;

pub fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let cfg = Config::load();
    let _log_guard = logging::init(&cfg, &cli.log_level)?;

    tracing::info!(?cli.path, "starting lazyjust");

    let result = discovery::discover(&cli.path)?;
    println!("discovered {} justfiles", result.justfiles.len());
    for j in &result.justfiles {
        println!("  {} ({} recipes)", j.path.display(), j.recipes.len());
    }
    if !result.errors.is_empty() {
        eprintln!("warnings:");
        for (p, e) in &result.errors {
            eprintln!("  {}: {}", p.display(), e);
        }
    }
    Ok(())
}
