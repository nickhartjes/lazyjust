pub mod app;
pub mod cli;
pub mod config;
pub mod discovery;
pub mod error;
pub mod input;
pub mod logging;
pub mod session;
pub mod theme;
pub mod ui;

pub use error::{Error, Result};

use clap::Parser;
use cli::Cli;
use config::Config;

pub fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if let Some(cmd) = cli.command.as_ref() {
        return handle_subcommand(cmd);
    }

    let mut cfg = Config::load();
    if let Some(arg) = cli.list_mode {
        cfg.list_mode = arg.into();
    }
    let _log_guard = logging::init(&cfg, &cli.log_level)?;

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    rt.block_on(async move { async_main(cli, cfg).await })
}

fn handle_subcommand(cmd: &cli::Commands) -> anyhow::Result<()> {
    match cmd {
        cli::Commands::Config { action } => match action {
            cli::ConfigAction::Path => {
                println!("{}", config::paths::config_file_path().display());
                Ok(())
            }
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
        },
    }
}

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
