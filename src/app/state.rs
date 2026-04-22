use super::types::*;
use std::path::PathBuf;

#[derive(Debug)]
pub struct App {
    pub justfiles: Vec<Justfile>,
    pub active_justfile: usize,
    pub filter: String,
    pub list_cursor: usize,
    pub sessions: Vec<SessionMeta>,
    pub active_session: Option<SessionId>,
    pub focus: Focus,
    pub mode: Mode,
    pub split_ratio: f32,
    pub collapsed_groups: std::collections::HashSet<String>,
    pub startup_errors: Vec<(PathBuf, String)>,
    pub next_session_id: SessionId,
    pub status_message: Option<String>,
}

impl App {
    pub fn new(
        justfiles: Vec<Justfile>,
        startup_errors: Vec<(PathBuf, String)>,
        split_ratio: f32,
    ) -> Self {
        Self {
            justfiles,
            active_justfile: 0,
            filter: String::new(),
            list_cursor: 0,
            sessions: Vec::new(),
            active_session: None,
            focus: Focus::List,
            mode: Mode::Normal,
            split_ratio,
            collapsed_groups: Default::default(),
            startup_errors,
            next_session_id: 1,
            status_message: None,
        }
    }

    pub fn active_justfile(&self) -> Option<&Justfile> {
        self.justfiles.get(self.active_justfile)
    }

    pub fn active_justfile_mut(&mut self) -> Option<&mut Justfile> {
        self.justfiles.get_mut(self.active_justfile)
    }

    pub fn next_session_id(&mut self) -> SessionId {
        let id = self.next_session_id;
        self.next_session_id += 1;
        id
    }

    pub fn recipe_at_cursor(&self) -> Option<&Recipe> {
        self.active_justfile()
            .and_then(|jf| jf.recipes.get(self.list_cursor))
    }
}
