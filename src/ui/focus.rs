use crate::app::types::Focus;
use ratatui::style::{Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, BorderType, Borders};

pub fn is_list_active(focus: Focus) -> bool {
    matches!(focus, Focus::List)
}

pub fn is_right_active(focus: Focus) -> bool {
    matches!(focus, Focus::Session)
}

/// Rounded-border block for a pane. Border in `theme.accent` when focused,
/// `theme.dim` when not. Title rendered bold in `theme.fg` when provided.
pub fn pane_block<'a>(
    title: Option<&'a str>,
    active: bool,
    theme: &crate::theme::Theme,
) -> Block<'a> {
    let border_color = if active { theme.accent } else { theme.dim };
    let mut b = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().fg(theme.fg).bg(theme.bg));
    if let Some(t) = title {
        b = b.title(Span::styled(
            format!(" {t} "),
            Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
        ));
    }
    b
}
