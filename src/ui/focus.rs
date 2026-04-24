use crate::app::types::Focus;
use ratatui::style::Style;
use ratatui::text::Span;

pub fn is_list_active(focus: Focus) -> bool {
    matches!(focus, Focus::List)
}

pub fn is_right_active(focus: Focus) -> bool {
    matches!(focus, Focus::Session)
}

pub fn focus_bar(active: bool, theme: &crate::theme::Theme) -> Span<'static> {
    let color = if active { theme.accent } else { theme.dim };
    Span::styled("▍", Style::default().fg(color))
}
