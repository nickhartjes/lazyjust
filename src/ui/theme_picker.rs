use crate::app::state::App;
use crate::app::types::Mode;
use crate::theme::Theme;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let (names, highlighted) = match &app.mode {
        Mode::ThemePicker {
            names, highlighted, ..
        } => (names, *highlighted),
        _ => return,
    };

    let theme: &Theme = &app.theme;

    let outer = centered(area, 40, 60);
    f.render_widget(Clear, outer);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.accent))
        .title(Span::styled(
            " Theme ",
            Style::default()
                .fg(theme.fg)
                .add_modifier(Modifier::BOLD),
        ))
        .title_alignment(Alignment::Center)
        .style(Style::default().bg(theme.bg).fg(theme.fg));
    let inner = block.inner(outer);
    f.render_widget(block, outer);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(inner);
    let list_area = chunks[0];
    let hint_area = chunks[1];

    let items: Vec<ListItem> = names
        .iter()
        .map(|n| ListItem::new(Span::raw(n.clone())))
        .collect();

    let list = List::new(items)
        .highlight_style(
            Style::default()
                .bg(theme.highlight)
                .fg(theme.selected_fg)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▸ ");

    let mut state = ListState::default();
    state.select(Some(highlighted));
    f.render_stateful_widget(list, list_area, &mut state);

    let hint = Paragraph::new(Span::styled(
        "j/k select · Enter apply & save · Esc revert",
        Style::default().fg(theme.dim),
    ))
    .alignment(Alignment::Center);
    f.render_widget(hint, hint_area);
}

fn centered(area: Rect, min_w: u16, min_h: u16) -> Rect {
    let w = min_w.min(area.width.saturating_sub(4));
    let h = min_h.min(area.height.saturating_sub(4));
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    Rect {
        x,
        y,
        width: w,
        height: h,
    }
}
