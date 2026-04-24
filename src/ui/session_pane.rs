use crate::ui::focus::pane_block;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

pub fn render(f: &mut Frame, area: Rect, screen: &vt100::Parser, active: bool, theme: &crate::theme::Theme) {
    let block = pane_block("session", active, theme);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let grid = screen.screen();
    let rows = inner.height as usize;
    let cols = inner.width as usize;

    let mut lines = Vec::with_capacity(rows);
    for r in 0..rows {
        let mut spans = Vec::with_capacity(cols);
        for c in 0..cols {
            if let Some(cell) = grid.cell(r as u16, c as u16) {
                let mut style = Style::default();
                let fg = cell.fgcolor();
                let bg = cell.bgcolor();
                if let Some(color) = convert_color(fg) {
                    style = style.fg(color);
                }
                if let Some(color) = convert_color(bg) {
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
    f.render_widget(p, inner);
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
