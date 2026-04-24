use crate::app::types::Focus;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::Span;
use ratatui::Frame;

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

/// Paint a solid 1-row bar across the full width of `area`'s topmost row.
/// Accent bg when active; dim bg when not.
pub fn paint_focus_bar(f: &mut Frame, area: Rect, active: bool, theme: &crate::theme::Theme) {
    if area.height == 0 || area.width == 0 {
        return;
    }
    let bg = if active { theme.accent } else { theme.dim };
    let style = Style::default().bg(bg);
    let buf: &mut Buffer = f.buffer_mut();
    for x in 0..area.width {
        let cell = buf.get_mut(area.x + x, area.y);
        cell.set_symbol(" ");
        cell.set_style(style);
    }
}
