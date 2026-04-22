use std::io;
use std::path::PathBuf;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("`just` binary not found on PATH. Install: https://github.com/casey/just")]
    JustNotFound,

    #[error("failed to invoke `just` ({path}): {source}")]
    JustInvocation {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("`just --dump` failed for {path}: exit {code}\n{stderr}")]
    JustDump {
        path: PathBuf,
        code: i32,
        stderr: String,
    },

    #[error("failed to parse `just --dump` output for {path}: {source}")]
    JustDumpParse {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },

    #[error("filesystem walk error at {path}: {source}")]
    Walk {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("PTY spawn failed: {0}")]
    PtySpawn(String),

    #[error("terminal too small: minimum {min_cols}x{min_rows}, got {cols}x{rows}")]
    TerminalTooSmall {
        cols: u16,
        rows: u16,
        min_cols: u16,
        min_rows: u16,
    },

    #[error(transparent)]
    Io(#[from] io::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
