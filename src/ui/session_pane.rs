use crate::app::types::SessionMeta;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

pub fn render(
    f: &mut Frame,
    area: Rect,
    screen: &vt100::Parser,
    meta: &SessionMeta,
    active: bool,
    theme: &crate::theme::Theme,
) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(1),
        ])
        .split(area);
    let header_area = rows[0];
    let body_area = rows[2];

    crate::ui::session_header::render(f, header_area, meta, active, theme);

    // reserve last column of body for the scroll thumb
    let grid_area = Rect {
        width: body_area.width.saturating_sub(1),
        ..body_area
    };
    let scroll_area = Rect {
        x: body_area.x + body_area.width.saturating_sub(1),
        y: body_area.y,
        width: 1,
        height: body_area.height,
    };

    let grid = screen.screen();
    let rows_count = grid_area.height as usize;
    let cols = grid_area.width as usize;

    let mut lines = Vec::with_capacity(rows_count);
    for r in 0..rows_count {
        let mut spans = Vec::with_capacity(cols);
        for c in 0..cols {
            if let Some(cell) = grid.cell(r as u16, c as u16) {
                let mut style = Style::default();
                if let Some(color) = convert_color(cell.fgcolor()) {
                    style = style.fg(color);
                }
                if let Some(color) = convert_color(cell.bgcolor()) {
                    style = style.bg(color);
                }
                if cell.bold() {
                    style = style.add_modifier(Modifier::BOLD);
                }
                if cell.italic() {
                    style = style.add_modifier(Modifier::ITALIC);
                }
                if cell.underline() {
                    style = style.add_modifier(Modifier::UNDERLINED);
                }
                let ch = cell.contents();
                spans.push(Span::styled(
                    if ch.is_empty() { " ".into() } else { ch },
                    style,
                ));
            } else {
                spans.push(Span::raw(" "));
            }
        }
        lines.push(Line::from(spans));
    }

    let p = Paragraph::new(lines);
    f.render_widget(p, grid_area);

    // scroll thumb from vt100 scrollback
    let (total, top) = scrollback_dims(screen, rows_count);
    let buf = f.buffer_mut();
    crate::ui::scrollbar::render(buf, scroll_area, total, rows_count, top, theme);
}

fn scrollback_dims(screen: &vt100::Parser, viewport: usize) -> (usize, usize) {
    let grid = screen.screen();
    let total = viewport + grid.scrollback();
    let top = grid.scrollback();
    (total, top)
}

fn convert_color(c: vt100::Color) -> Option<Color> {
    use vt100::Color as V;
    match c {
        V::Default => None,
        V::Idx(0) => Some(Color::Black),
        V::Idx(1) => Some(Color::Red),
        V::Idx(2) => Some(Color::Green),
        V::Idx(3) => Some(Color::Yellow),
        V::Idx(4) => Some(Color::Blue),
        V::Idx(5) => Some(Color::Magenta),
        V::Idx(6) => Some(Color::Cyan),
        V::Idx(7) => Some(Color::Gray),
        V::Idx(8) => Some(Color::DarkGray),
        V::Idx(9) => Some(Color::LightRed),
        V::Idx(10) => Some(Color::LightGreen),
        V::Idx(11) => Some(Color::LightYellow),
        V::Idx(12) => Some(Color::LightBlue),
        V::Idx(13) => Some(Color::LightMagenta),
        V::Idx(14) => Some(Color::LightCyan),
        V::Idx(15) => Some(Color::White),
        V::Idx(n) => Some(Color::Indexed(n)),
        V::Rgb(r, g, b) => Some(Color::Rgb(r, g, b)),
    }
}
