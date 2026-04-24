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
