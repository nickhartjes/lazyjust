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
    pub theme: crate::theme::Theme,
    pub theme_name: String,
    pub collapsed_groups: std::collections::HashSet<String>,
    pub startup_errors: Vec<(PathBuf, String)>,
    pub next_session_id: SessionId,
    pub status_message: Option<String>,
    pub icon_style: crate::ui::icon_style::IconStyle,
}

impl App {
    pub fn new(
        justfiles: Vec<Justfile>,
        startup_errors: Vec<(PathBuf, String)>,
        split_ratio: f32,
        theme: crate::theme::Theme,
        theme_name: String,
        icon_style: crate::ui::icon_style::IconStyle,
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
            theme,
            theme_name,
            collapsed_groups: Default::default(),
            startup_errors,
            next_session_id: 1,
            status_message: None,
            icon_style,
        }
    }

    pub fn active_justfile(&self) -> Option<&Justfile> {
        self.justfiles.get(self.active_justfile)
    }

    pub fn active_justfile_mut(&mut self) -> Option<&mut Justfile> {
        self.justfiles.get_mut(self.active_justfile)
    }

    /// Hand out the next `SessionId` and advance the counter. Call sites that
    /// need to allocate an id use `app.next_session_id()`; the bare field
    /// `app.next_session_id` still reads the next-to-be-handed-out value —
    /// this field/method name overlap is intentional.
    pub fn next_session_id(&mut self) -> SessionId {
        let id = self.next_session_id;
        self.next_session_id = self
            .next_session_id
            .checked_add(1)
            .expect("SessionId overflow: 2^64 sessions");
        id
    }

    pub fn recipe_at_cursor(&self) -> Option<&Recipe> {
        self.active_justfile()
            .and_then(|jf| jf.recipes.get(self.list_cursor))
    }

    pub fn session_mut(
        &mut self,
        id: crate::app::types::SessionId,
    ) -> Option<&mut crate::app::types::SessionMeta> {
        self.sessions.iter_mut().find(|s| s.id == id)
    }

    pub fn session(
        &self,
        id: crate::app::types::SessionId,
    ) -> Option<&crate::app::types::SessionMeta> {
        self.sessions.iter().find(|s| s.id == id)
    }
}
