pub mod focus;
pub mod help;
pub mod icon_style;
pub mod layout;
pub mod list;
pub mod modal;
pub mod modal_base;
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
        let msg_text = format!("Terminal too small — need at least {cfg_cols}×{cfg_rows}.",);
        let msg = ratatui::text::Line::from(ratatui::text::Span::styled(
            msg_text,
            ratatui::style::Style::default().fg(theme.dim).bg(theme.bg),
        ));
        let y = size.height / 2;
        let area = ratatui::layout::Rect {
            x: size.x,
            y: size.y + y,
            width: size.width,
            height: 1,
        };
        let para =
            ratatui::widgets::Paragraph::new(msg).alignment(ratatui::layout::Alignment::Center);
        f.render_widget(para, area);
        return;
    }
    if app.justfiles.is_empty() {
        let msg = ratatui::widgets::Paragraph::new("No justfiles found.\nPress q to quit.")
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(msg, size);
        return;
    }

    let bg_fill = ratatui::widgets::Paragraph::new("")
        .style(ratatui::style::Style::default().bg(app.theme.bg));
    f.render_widget(bg_fill, size);

    let panes = layout::compute(size, app);
    let list_active = focus::is_list_active(app.focus);
    let right_active = focus::is_right_active(app.focus);
    top_bar::render(f, panes.top_bar, app, &app.theme);

    let list_block = focus::pane_block(Some("recipes"), list_active, &app.theme);
    let list_body = list_block.inner(panes.list);
    f.render_widget(list_block, panes.list);
    list::render(f, list_body, app, &app.theme);

    let display_sid = if matches!(app.focus, crate::app::types::Focus::Session) {
        app.active_session
    } else {
        app.recipe_at_cursor().and_then(|r| r.runs.last().copied())
    };
    let right_title = if display_sid.is_some() { "session" } else { "preview" };
    let right_block = focus::pane_block(Some(right_title), right_active, &app.theme);
    let right_body = right_block.inner(panes.right);
    f.render_widget(right_block, panes.right);
    if let Some(id) = display_sid {
        if let (Some(screen), Some(meta)) = (screens.get(&id), app.session(id)) {
            session_pane::render(f, right_body, screen, meta, right_active, &app.theme);
        } else {
            preview::render(f, right_body, app, &app.theme);
        }
    } else {
        preview::render(f, right_body, app, &app.theme);
    }
    status_bar::render(f, panes.status, app, &app.theme);
    modal::render(f, app, &app.theme);
    if matches!(&app.mode, crate::app::types::Mode::ThemePicker { .. }) {
        theme_picker::render(f, size, app);
    }
}
