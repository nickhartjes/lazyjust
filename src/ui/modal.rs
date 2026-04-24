use crate::app::types::Mode;
use crate::app::App;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph};
use ratatui::Frame;

pub fn render(f: &mut Frame, app: &App, theme: &crate::theme::Theme) {
    match &app.mode {
        Mode::Dropdown { filter, cursor } => render_dropdown(f, app, filter, *cursor, theme),
        Mode::Help { .. } => {
            let h = f.size().height.saturating_sub(4).min(30);
            let area = centered(f.size(), 72, h);
            f.render_widget(Clear, area);
            super::help::render(f, app, area, theme);
        }
        Mode::Confirm { prompt, .. } => render_confirm(f, prompt),
        Mode::ParamInput { .. } => {
            let area = centered(f.size(), 60, 12);
            super::param_modal::render(f, app, area, theme);
        }
        Mode::ErrorsList => render_errors(f, app, theme),
        _ => {}
    }
}

fn centered(parent: Rect, w: u16, h: u16) -> Rect {
    let v = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(parent.height.saturating_sub(h) / 2),
            Constraint::Length(h),
            Constraint::Min(0),
        ])
        .split(parent);
    let hslices = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(parent.width.saturating_sub(w) / 2),
            Constraint::Length(w),
            Constraint::Min(0),
        ])
        .split(v[1]);
    hslices[1]
}

fn render_dropdown(
    f: &mut Frame,
    app: &App,
    filter: &str,
    cursor: usize,
    theme: &crate::theme::Theme,
) {
    let area = centered(f.size(), 60, 14);
    f.render_widget(Clear, area);
    let indices = crate::app::reducer::filtered_justfile_indices(app, filter);
    let items: Vec<ListItem> = indices
        .iter()
        .map(|&i| ListItem::new(app.justfiles[i].path.display().to_string()))
        .collect();
    let mut state = ListState::default();
    state.select(Some(cursor.min(items.len().saturating_sub(1))));
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("justfile: /{filter}")),
        )
        .highlight_style(
            Style::default()
                .bg(theme.highlight)
                .fg(theme.selected_fg)
                .add_modifier(Modifier::BOLD),
        );
    f.render_stateful_widget(list, area, &mut state);
}

fn render_confirm(f: &mut Frame, prompt: &str) {
    let area = centered(f.size(), 48, 5);
    f.render_widget(Clear, area);
    let p = Paragraph::new(format!("{prompt}\n  [y]es     [n]o"))
        .block(Block::default().borders(Borders::ALL).title("confirm"));
    f.render_widget(p, area);
}

fn render_errors(f: &mut Frame, app: &App, theme: &crate::theme::Theme) {
    use ratatui::widgets::Wrap;
    let area = centered(f.size(), 80, 20);
    f.render_widget(Clear, area);
    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(format!(
        "{} justfile(s) failed to load:",
        app.startup_errors.len()
    )));
    lines.push(Line::from(""));
    for (path, msg) in &app.startup_errors {
        lines.push(Line::from(Span::styled(
            path.display().to_string(),
            Style::default().fg(theme.accent),
        )));
        for l in msg.lines() {
            lines.push(Line::from(format!("  {l}")));
        }
        lines.push(Line::from(""));
    }
    lines.push(Line::from(Span::styled(
        "Esc / q / e to close",
        Style::default().fg(theme.dim),
    )));
    let p = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title("load errors"))
        .wrap(Wrap { trim: false });
    f.render_widget(p, area);
}
