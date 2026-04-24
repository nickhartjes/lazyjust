use crate::app::types::{Recipe, Status};
use crate::app::App;
use crate::ui::focus::{is_list_active, pane_block};
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{List, ListItem, ListState};
use ratatui::Frame;

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let active = is_list_active(app.focus);
    let Some(jf) = app.active_justfile() else {
        f.render_widget(pane_block("recipes", active), area);
        return;
    };

    let items = build_items(jf.recipes.as_slice(), &app.filter, app);

    let mut state = ListState::default();
    state.select(Some(app.list_cursor.min(items.len().saturating_sub(1))));

    let list = List::new(items)
        .block(pane_block("recipes", active))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">");
    f.render_stateful_widget(list, area, &mut state);
}

fn build_items<'a>(recipes: &'a [Recipe], filter: &str, app: &App) -> Vec<ListItem<'a>> {
    let names: Vec<&str> = recipes.iter().map(|r| r.name.as_str()).collect();
    let scored = crate::app::filter::fuzzy_match(&names, filter);

    let mut items = Vec::new();
    let mut current_group: Option<&str> = None;

    for (idx, _score) in scored {
        let r = &recipes[idx];
        let group_name = r.group.as_deref();
        if group_name != current_group {
            if let Some(g) = group_name {
                items.push(ListItem::new(Line::from(Span::styled(
                    format!("GROUP: {g}"),
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                ))));
            } else {
                items.push(ListItem::new(Line::from(Span::styled(
                    "GROUP: (ungrouped)",
                    Style::default().fg(Color::DarkGray),
                ))));
            }
            current_group = group_name;
        }
        let indicators = session_indicators_for(r, app);
        let mut spans = vec![Span::raw("  "), Span::raw(r.name.clone())];
        if !indicators.is_empty() {
            spans.push(Span::raw("   "));
            spans.extend(indicators);
        }
        items.push(ListItem::new(Line::from(spans)));
    }
    items
}

fn session_indicators_for<'a>(r: &'a Recipe, app: &App) -> Vec<Span<'a>> {
    let mut out = Vec::new();
    let mut emitted = 0usize;
    for &sid in r.runs.iter().rev() {
        if emitted >= 3 {
            out.push(Span::raw(format!(" (+{} more)", r.runs.len() - 3)));
            break;
        }
        if let Some(s) = app.session(sid) {
            out.push(Span::raw(" "));
            out.push(status_span(s.status, s.unread));
            emitted += 1;
        }
    }
    out
}

fn status_span(status: Status, unread: bool) -> Span<'static> {
    let (icon, color) = match status {
        Status::Running => ("●", Color::Blue),
        Status::ShellAfterExit { code } | Status::Exited { code } if code == 0 => (
            "✓",
            if unread {
                Color::Green
            } else {
                Color::DarkGray
            },
        ),
        Status::ShellAfterExit { .. } | Status::Exited { .. } => {
            ("✗", if unread { Color::Red } else { Color::DarkGray })
        }
        Status::Broken => ("!", Color::Yellow),
    };
    Span::styled(icon, Style::default().fg(color))
}
