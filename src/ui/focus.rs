use crate::app::types::Focus;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders};

pub fn pane_block<'a>(title: &'a str, active: bool, theme: &crate::theme::Theme) -> Block<'a> {
    let border_color = if active { theme.accent } else { theme.dim };
    let title_style = if active {
        Style::default()
            .fg(theme.bg)
            .bg(theme.accent)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.dim)
    };
    Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .title(ratatui::text::Span::styled(
            format!(" {title} "),
            title_style,
        ))
}

pub fn is_list_active(focus: Focus) -> bool {
    matches!(focus, Focus::List)
}

pub fn is_right_active(focus: Focus) -> bool {
    matches!(focus, Focus::Session)
}
