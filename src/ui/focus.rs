use crate::app::types::Focus;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders};

pub fn pane_block<'a>(title: &'a str, active: bool) -> Block<'a> {
    let border_color = if active { Color::Cyan } else { Color::DarkGray };
    let title_style = if active {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
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
