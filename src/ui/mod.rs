pub mod focus;
pub mod help;
pub mod layout;
pub mod list;
pub mod modal;
pub mod param_modal;
pub mod preview;
pub mod session_pane;
pub mod status_bar;
pub mod top_bar;

use crate::app::App;
use ratatui::Frame;

pub type SessionScreens = std::collections::HashMap<crate::app::types::SessionId, vt100::Parser>;

pub fn render(f: &mut Frame, app: &App, screens: &SessionScreens) {
    let size = f.size();
    if size.width < 40 || size.height < 10 {
        let msg = ratatui::widgets::Paragraph::new("Terminal too small")
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(msg, size);
        return;
    }
    if app.justfiles.is_empty() {
        let msg = ratatui::widgets::Paragraph::new("No justfiles found.\nPress q to quit.")
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(msg, size);
        return;
    }

    let panes = layout::compute(size, app);
    top_bar::render(f, panes.top_bar, app, &app.theme);
    list::render(f, panes.list, app, &app.theme);
    let right_active = focus::is_right_active(app.focus);
    if let Some(id) = app.active_session {
        if let Some(screen) = screens.get(&id) {
            session_pane::render(f, panes.right, screen, right_active, &app.theme);
        } else {
            preview::render(f, panes.right, app, &app.theme);
        }
    } else {
        preview::render(f, panes.right, app, &app.theme);
    }
    status_bar::render(f, panes.status, app);
    modal::render(f, app, &app.theme);
}
