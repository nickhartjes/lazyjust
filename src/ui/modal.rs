use crate::app::types::Mode;
use crate::app::App;
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{List, ListItem, ListState, Paragraph};
use ratatui::Frame;

pub fn render(f: &mut Frame, app: &App, theme: &crate::theme::Theme) {
    match &app.mode {
        Mode::Dropdown { filter, cursor } => render_dropdown(f, app, filter, *cursor, theme),
        Mode::Help { .. } => {
            let h = f.area().height.saturating_sub(4).min(30);
            let area = crate::ui::modal_base::centered(f.area(), 72, h);
            crate::ui::modal_base::clear(f, area);
            super::help::render(f, app, area, theme);
        }
        Mode::Confirm { prompt, .. } => render_confirm(f, prompt, theme),
        Mode::ParamInput { .. } => {
            let area = crate::ui::modal_base::centered(f.area(), 60, 12);
            super::param_modal::render(f, app, area, theme);
        }
        Mode::ErrorsList => render_errors(f, app, theme),
        _ => {}
    }
}

fn render_dropdown(
    f: &mut Frame,
    app: &App,
    filter: &str,
    cursor: usize,
    theme: &crate::theme::Theme,
) {
    let frame_w = f.area().width;
    let modal_w = frame_w.saturating_sub(4).clamp(40, 100);
    let area = crate::ui::modal_base::centered(f.area(), modal_w, 14);
    crate::ui::modal_base::clear(f, area);
    // Inside the modal: 2 cols of border + 2 cols of left/right padding.
    let row_max = (modal_w as usize).saturating_sub(4);
    let indices = crate::app::reducer::filtered_justfile_indices(app, filter);
    let items: Vec<ListItem> = indices
        .iter()
        .map(|&i| {
            ListItem::new(crate::ui::path_display::shorten(
                &app.justfiles[i].path,
                row_max,
            ))
        })
        .collect();
    let mut state = ListState::default();
    state.select(Some(cursor.min(items.len().saturating_sub(1))));
    let title = format!("justfile: /{filter}");
    let list = List::new(items)
        .block(crate::ui::modal_base::block(&title, theme))
        .highlight_style(
            Style::default()
                .bg(theme.highlight)
                .fg(theme.selected_fg)
                .add_modifier(Modifier::BOLD),
        );
    f.render_stateful_widget(list, area, &mut state);
}

fn render_confirm(f: &mut Frame, prompt: &str, theme: &crate::theme::Theme) {
    let area = crate::ui::modal_base::centered(f.area(), 52, 7);
    crate::ui::modal_base::clear(f, area);
    let p = Paragraph::new(format!("{prompt}\n\n[y]es     [n]o"))
        .block(crate::ui::modal_base::block("confirm", theme));
    f.render_widget(p, area);
}

fn render_errors(f: &mut Frame, app: &App, theme: &crate::theme::Theme) {
    use ratatui::text::Span;
    use ratatui::widgets::Wrap;
    let area = crate::ui::modal_base::centered(f.area(), 80, 20);
    crate::ui::modal_base::clear(f, area);
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
        .block(crate::ui::modal_base::block("load errors", theme))
        .wrap(Wrap { trim: false });
    f.render_widget(p, area);
}
