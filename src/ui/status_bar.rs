use crate::app::types::{Focus, Mode};
use crate::app::App;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

pub fn render(f: &mut Frame, area: Rect, app: &App, theme: &crate::theme::Theme) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    let hint = hint_for(&app.mode, app, theme);
    f.render_widget(Paragraph::new(hint), cols[0]);

    if let Some(msg) = &app.status_message {
        let style = Style::default().fg(if msg.starts_with("err") || msg.starts_with("Error") {
            theme.error
        } else {
            theme.warn
        });
        let right = Paragraph::new(Line::from(Span::styled(msg.clone(), style)))
            .alignment(Alignment::Right);
        f.render_widget(right, cols[1]);
    }
}

fn hint_for<'a>(mode: &'a Mode, app: &'a App, theme: &crate::theme::Theme) -> Line<'a> {
    let sep = Span::styled("  ·  ", Style::default().fg(theme.dim));
    let k = |s: &str| Span::styled(s.to_string(), Style::default().fg(theme.fg));
    let d = |s: &str| Span::styled(s.to_string(), Style::default().fg(theme.dim));
    match mode {
        Mode::Normal if matches!(app.focus, Focus::Session) => Line::from(vec![
            k("PgUp/PgDn"), Span::raw(" "), d("scroll"), sep.clone(),
            k("F12"), Span::raw(" "), d("list"), sep.clone(),
            k("K"), Span::raw(" "), d("kill"), sep.clone(),
            k("x"), Span::raw(" "), d("close"), sep,
            k("?"), Span::raw(" "), d("help"),
        ]),
        Mode::Normal => Line::from(vec![
            k("⏎"), Span::raw(" "), d("run"), sep.clone(),
            k("/"), Span::raw(" "), d("filter"), sep.clone(),
            k("t"), Span::raw(" "), d("theme"), sep.clone(),
            k("?"), Span::raw(" "), d("help"), sep,
            k("q"), Span::raw(" "), d("quit"),
        ]),
        Mode::FilterInput => Line::from(vec![
            d("/"), k(&format!("{}_", app.filter)), sep.clone(),
            k("Esc"), Span::raw(" "), d("cancel"), sep,
            k("⏎"), Span::raw(" "), d("apply"),
        ]),
        Mode::ParamInput { values, cursor, recipe_idx } => {
            let jf = app.active_justfile();
            let name = jf
                .and_then(|j| j.recipes.get(*recipe_idx))
                .and_then(|r| r.params.get(*cursor))
                .map(|p| p.name.as_str())
                .unwrap_or("");
            let val = values.get(*cursor).cloned().unwrap_or_default();
            Line::from(vec![
                d(&format!("[{}/{}] ", cursor + 1, values.len().max(1))),
                k(&format!("{name} = {val}_")),
                sep.clone(),
                k("⏎"), Span::raw(" "), d("next"),
                sep,
                k("Esc"), Span::raw(" "), d("cancel"),
            ])
        }
        Mode::ThemePicker { .. } => Line::from(vec![
            k("j/k"), Span::raw(" "), d("select"), sep.clone(),
            k("⏎"), Span::raw(" "), d("apply & save"), sep,
            k("Esc"), Span::raw(" "), d("revert"),
        ]),
        Mode::Help { .. } => Line::from(vec![k("Esc / q"), Span::raw(" "), d("close")]),
        Mode::Confirm { prompt, .. } => Line::from(vec![
            d(prompt), Span::raw(" "),
            k("y"), Span::raw(" "), d("yes"), sep, k("n"), Span::raw(" "), d("no"),
        ]),
        Mode::Dropdown { filter, .. } => Line::from(vec![
            d("justfile: /"), k(&format!("{filter}_")), sep.clone(),
            k("⏎"), Span::raw(" "), d("pick"), sep,
            k("Esc"), Span::raw(" "), d("cancel"),
        ]),
        Mode::ErrorsList => Line::from(vec![k("Esc / q / e"), Span::raw(" "), d("close")]),
    }
}
