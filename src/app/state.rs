use super::types::{Focus, Justfile, ListMode, Mode, Recipe, SessionId, SessionMeta};
use super::view::ListView;
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
    pub icon_style: crate::ui::icon_style::IconStyle,
    pub collapsed_groups: std::collections::HashSet<String>,
    pub startup_errors: Vec<(PathBuf, String)>,
    pub next_session_id: SessionId,
    pub status_message: Option<String>,
    pub list_mode: ListMode,
    pub discovery_root: PathBuf,
    pub view: ListView,
}

impl App {
    // constructed once from Config + CLI args; builder pattern not warranted at this call count.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        justfiles: Vec<Justfile>,
        startup_errors: Vec<(PathBuf, String)>,
        split_ratio: f32,
        theme: crate::theme::Theme,
        theme_name: String,
        icon_style: crate::ui::icon_style::IconStyle,
        list_mode: ListMode,
        discovery_root: PathBuf,
    ) -> Self {
        let view = ListView::build(&justfiles, list_mode, 0);
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
            icon_style,
            collapsed_groups: Default::default(),
            startup_errors,
            next_session_id: 1,
            status_message: None,
            list_mode,
            discovery_root,
            view,
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
        let (jf_idx, recipe_idx) = self.view.recipe_at(self.list_cursor)?;
        self.justfiles.get(jf_idx)?.recipes.get(recipe_idx)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::types::{Justfile, ListMode, Recipe};
    use std::path::PathBuf;

    fn recipe(n: &str) -> Recipe {
        Recipe {
            name: n.into(),
            module_path: vec![],
            group: None,
            params: vec![],
            doc: None,
            command_preview: String::new(),
            runs: vec![],
            dependencies: vec![],
        }
    }

    #[test]
    fn new_initializes_view_for_active_mode_by_default() {
        let jf = Justfile {
            path: PathBuf::from("/x/justfile"),
            recipes: vec![recipe("a"), recipe("b")],
            groups: vec![],
        };
        let app = App::new(
            vec![jf],
            vec![],
            0.3,
            crate::theme::registry::resolve(crate::theme::DEFAULT_THEME_NAME),
            crate::theme::DEFAULT_THEME_NAME.to_string(),
            crate::ui::icon_style::IconStyle::Round,
            ListMode::Active,
            PathBuf::from("/x"),
        );
        assert_eq!(app.list_mode, ListMode::Active);
        assert_eq!(app.discovery_root, PathBuf::from("/x"));
        assert_eq!(app.view.recipe_count(), 2);
    }

    #[test]
    fn new_initializes_view_for_all_mode() {
        let a = Justfile {
            path: PathBuf::from("/r/a/justfile"),
            recipes: vec![recipe("ra1")],
            groups: vec![],
        };
        let b = Justfile {
            path: PathBuf::from("/r/b/justfile"),
            recipes: vec![recipe("rb1"), recipe("rb2")],
            groups: vec![],
        };
        let app = App::new(
            vec![a, b],
            vec![],
            0.3,
            crate::theme::registry::resolve(crate::theme::DEFAULT_THEME_NAME),
            crate::theme::DEFAULT_THEME_NAME.to_string(),
            crate::ui::icon_style::IconStyle::Round,
            ListMode::All,
            PathBuf::from("/r"),
        );
        assert_eq!(app.list_mode, ListMode::All);
        assert_eq!(app.view.recipe_count(), 3);
        // 2 headers + 3 recipes
        assert_eq!(app.view.rows.len(), 5);
    }
}
