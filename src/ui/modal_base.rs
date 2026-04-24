use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, BorderType, Borders, Clear};
use ratatui::Frame;

pub fn centered(parent: Rect, w: u16, h: u16) -> Rect {
    let v = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(parent.height.saturating_sub(h) / 2),
            Constraint::Length(h),
            Constraint::Min(0),
        ])
        .split(parent);
    let h_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(parent.width.saturating_sub(w) / 2),
            Constraint::Length(w),
            Constraint::Min(0),
        ])
        .split(v[1]);
    h_cols[1]
}

pub fn block<'a>(title: &'a str, theme: &crate::theme::Theme) -> Block<'a> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.accent))
        .title(Span::styled(
            format!(" {title} "),
            Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().fg(theme.fg).bg(theme.bg))
}

pub fn clear(f: &mut Frame, area: Rect) {
    f.render_widget(Clear, area);
}
