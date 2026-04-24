pub mod focus;
pub mod help;
pub mod icon_style;
pub mod layout;
pub mod list;
pub mod modal;
pub mod param_modal;
pub mod preview;
pub mod scrollbar;
pub mod session_header;
pub mod session_pane;
pub mod status_bar;
pub mod theme_picker;
pub mod top_bar;

use crate::app::App;
use ratatui::Frame;

pub type SessionScreens = std::collections::HashMap<crate::app::types::SessionId, vt100::Parser>;

pub fn render(f: &mut Frame, app: &App, screens: &SessionScreens) {
    let size = f.size();
    let cfg_cols: u16 = 40;
    let cfg_rows: u16 = 10;
    if size.width < cfg_cols || size.height < cfg_rows {
        let theme = &app.theme;
        let filled = ratatui::widgets::Paragraph::new("")
            .style(ratatui::style::Style::default().bg(theme.bg));
        f.render_widget(filled, size);
        let msg_text = format!(
            "Terminal too small — need at least {cfg_cols}×{cfg_rows}.",
        );
        let msg = ratatui::text::Line::from(ratatui::text::Span::styled(
            msg_text,
            ratatui::style::Style::default().fg(theme.dim).bg(theme.bg),
        ));
        let y = size.height / 2;
        let area = ratatui::layout::Rect { x: size.x, y: size.y + y, width: size.width, height: 1 };
        let para = ratatui::widgets::Paragraph::new(msg)
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(para, area);
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
        if let (Some(screen), Some(meta)) = (screens.get(&id), app.session(id)) {
            session_pane::render(f, panes.right, screen, meta, right_active, &app.theme);
        } else {
            preview::render(f, panes.right, app, &app.theme);
        }
    } else {
        preview::render(f, panes.right, app, &app.theme);
    }
    status_bar::render(f, panes.status, app);
    modal::render(f, app, &app.theme);
    if matches!(&app.mode, crate::app::types::Mode::ThemePicker { .. }) {
        theme_picker::render(f, size, app);
    }
}
