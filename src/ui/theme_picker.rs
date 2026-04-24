use crate::app::state::App;
use crate::app::types::Mode;
use crate::theme::Theme;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::Span,
    widgets::{List, ListItem, ListState},
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
    crate::ui::modal_base::clear(f, outer);

    let block = crate::ui::modal_base::block("theme", theme);
    let inner = block.inner(outer);
    f.render_widget(block, outer);

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
    f.render_stateful_widget(list, inner, &mut state);
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
