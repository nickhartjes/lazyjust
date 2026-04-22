use clap::Parser;
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
}
