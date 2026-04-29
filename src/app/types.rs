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
    pub name: String,             // e.g. "build" or "api::serve"
    pub module_path: Vec<String>, // e.g. ["api"] for modded recipes
    pub group: Option<String>,
    pub params: Vec<Param>,
    pub dependencies: Vec<String>,
    pub doc: Option<String>,
    pub command_preview: String,
    pub runs: Vec<SessionId>,
}

impl Recipe {
    pub fn has_deps(&self) -> bool {
        !self.dependencies.is_empty()
    }

    pub fn dep_names(&self) -> Vec<&str> {
        self.dependencies.iter().map(String::as_str).collect()
    }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListMode {
    Active,
    All,
}

impl ListMode {
    /// Parse a config string. Returns `None` for unknown values so callers
    /// can warn-log and fall back to a default.
    pub fn parse(s: &str) -> Option<ListMode> {
        match s {
            "active" => Some(ListMode::Active),
            "all" => Some(ListMode::All),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Normal,
    FilterInput,
    ParamInput {
        recipe_idx: usize,
        values: Vec<String>,
        cursor: usize,
    },
    Dropdown {
        filter: String,
        cursor: usize,
    },
    Help {
        scroll: u16,
        origin: crate::app::help_section::SectionId,
    },
    Confirm {
        prompt: String,
        on_accept: ConfirmAction,
    },
    ErrorsList,
    ThemePicker {
        original_name: String,
        highlighted: usize,
        names: Vec<String>,
    },
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
    /// Human-readable recipe invocation (`just --justfile <p> <recipe> <args>`).
    /// Informational only — the actual PTY argv is `$SHELL -i` and the recipe
    /// is delivered via `session::shell::prime_line` on stdin.
    pub command_line: String,
    pub status: Status,
    pub unread: bool,
    pub started_at: Instant,
    pub log_path: PathBuf,
    /// OS process id of the shell child process, if the platform could provide one.
    pub pid: Option<u32>,
}

#[cfg(test)]
mod list_mode_tests {
    use super::*;

    #[test]
    fn list_mode_parse_known_values() {
        assert_eq!(ListMode::parse("active"), Some(ListMode::Active));
        assert_eq!(ListMode::parse("all"), Some(ListMode::All));
    }

    #[test]
    fn list_mode_parse_is_case_sensitive_by_design() {
        // Case-sensitive: TOML schema is lowercase-only and merge_ui
        // warn-logs on mismatch.
        assert_eq!(ListMode::parse("Active"), None);
        assert_eq!(ListMode::parse("ALL"), None);
        assert_eq!(ListMode::parse("weird"), None);
        assert_eq!(ListMode::parse(""), None);
    }
}
