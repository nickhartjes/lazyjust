use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "lazyjust", about = "Terminal UI for just", version)]
pub struct Cli {
    /// Project root to scan. Defaults to the current directory when omitted.
    #[arg(value_name = "PATH")]
    pub path: Option<PathBuf>,

    /// Pin FILE as the active justfile; walk its parent for siblings.
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
