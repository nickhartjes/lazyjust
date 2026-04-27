use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;

pub fn render(
    buf: &mut Buffer,
    area: Rect,
    total_rows: usize,
    viewport_rows: usize,
    top_row: usize,
    theme: &crate::theme::Theme,
) {
    if total_rows <= viewport_rows || area.height == 0 {
        return;
    }
    let track_h = area.height as usize;
    let thumb_h = ((viewport_rows * track_h) / total_rows).max(1);
    let thumb_y = (top_row * track_h) / total_rows;
    for y in 0..track_h {
        let cell = &mut buf[(area.x, area.y + y as u16)];
        let is_thumb = y >= thumb_y && y < thumb_y + thumb_h;
        cell.set_symbol("│");
        cell.set_style(Style::default().fg(if is_thumb { theme.accent } else { theme.dim }));
    }
}
