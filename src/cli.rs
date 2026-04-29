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

    /// Override the recipe-list mode for this run. Values: active, all.
    #[arg(long = "list-mode", value_enum, value_name = "MODE")]
    pub list_mode: Option<ListModeArg>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(clap::ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListModeArg {
    Active,
    All,
}

impl From<ListModeArg> for crate::app::types::ListMode {
    fn from(v: ListModeArg) -> Self {
        match v {
            ListModeArg::Active => crate::app::types::ListMode::Active,
            ListModeArg::All => crate::app::types::ListMode::All,
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn list_mode_flag_parses_active_and_all() {
        let cli = Cli::try_parse_from(["lazyjust", "--list-mode", "active"]).unwrap();
        assert_eq!(cli.list_mode, Some(ListModeArg::Active));
        let cli = Cli::try_parse_from(["lazyjust", "--list-mode", "all"]).unwrap();
        assert_eq!(cli.list_mode, Some(ListModeArg::All));
    }

    #[test]
    fn list_mode_flag_rejects_invalid_value() {
        let err = Cli::try_parse_from(["lazyjust", "--list-mode", "weird"]).unwrap_err();
        assert!(err.to_string().to_lowercase().contains("invalid"));
    }

    #[test]
    fn list_mode_flag_optional() {
        let cli = Cli::try_parse_from(["lazyjust"]).unwrap();
        assert!(cli.list_mode.is_none());
    }
}
