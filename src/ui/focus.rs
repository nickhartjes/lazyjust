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

/// Paint a 1-col `▍` down the full height of `area`'s leftmost column.
/// Accent when active; dim when not. Call BEFORE rendering pane content
/// that occupies col 0 — content overwrites the bar where they overlap.
pub fn paint_focus_bar(f: &mut Frame, area: Rect, active: bool, theme: &crate::theme::Theme) {
    if area.width == 0 {
        return;
    }
    let color = if active { theme.accent } else { theme.dim };
    let style = Style::default().fg(color).bg(theme.bg);
    let buf: &mut Buffer = f.buffer_mut();
    for y in 0..area.height {
        let cell = buf.get_mut(area.x, area.y + y);
        cell.set_symbol("▍");
        cell.set_style(style);
    }
}
