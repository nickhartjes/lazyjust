use std::path::PathBuf;
use std::time::Instant;

pub type SessionId = u64;

#[derive(Debug, Clone)]
pub struct Justfile {
    pub path: PathBuf,
    pub recipes: Vec<Recipe>,
    pub groups: Vec<String>, // declaration-ordered list of group names
}

#[derive(Debug, Clone)]
pub struct Recipe {
    pub name: String,              // e.g. "build" or "api::serve"
    pub module_path: Vec<String>,  // e.g. ["api"] for modded recipes
    pub group: Option<String>,
    pub params: Vec<Param>,
    pub doc: Option<String>,
    pub command_preview: String,
    pub runs: Vec<SessionId>,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub default: Option<String>,
    pub kind: ParamKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamKind {
    Positional,
    Variadic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Running,
    ShellAfterExit { code: i32 },
    Exited { code: i32 },
    Broken,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    List,
    Preview,
    Session,
    Modal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Normal,
    FilterInput,
    ParamInput { recipe_idx: usize, values: Vec<String>, cursor: usize },
    Dropdown { filter: String, cursor: usize },
    Help,
    Confirm { prompt: String, on_accept: ConfirmAction },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfirmAction {
    QuitKillAll,
    KillSession(SessionId),
    CloseSession(SessionId),
}

#[derive(Debug)]
pub struct SessionMeta {
    pub id: SessionId,
    pub recipe_name: String,
    pub command_line: String,
    pub status: Status,
    pub unread: bool,
    pub started_at: Instant,
    pub log_path: PathBuf,
}
