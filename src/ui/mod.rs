pub mod layout;
pub mod list;
pub mod modal;
pub mod preview;
pub mod session_pane;
pub mod status_bar;
pub mod top_bar;

use crate::app::App;
use ratatui::Frame;

pub type SessionScreens = std::collections::HashMap<crate::app::types::SessionId, vt100::Parser>;

pub fn render(f: &mut Frame, app: &App, screens: &SessionScreens) {
    let panes = layout::compute(f.size(), app);
    top_bar::render(f, panes.top_bar, app);
    list::render(f, panes.list, app);
    if let Some(id) = app.active_session {
        if let Some(screen) = screens.get(&id) {
            session_pane::render(f, panes.right, screen);
        } else {
            preview::render(f, panes.right, app);
        }
    } else {
        preview::render(f, panes.right, app);
    }
    status_bar::render(f, panes.status, app);
    modal::render(f, app);
}
