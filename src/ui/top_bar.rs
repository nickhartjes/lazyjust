use crate::app::App;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

pub fn render(f: &mut Frame, area: Rect, app: &App, theme: &crate::theme::Theme) {
    let justfile = app
        .active_justfile()
        .map(|j| j.path.display().to_string())
        .unwrap_or_else(|| "<no justfile>".into());

    let mut spans = vec![
        Span::styled("lazyjust", Style::default().fg(theme.accent)),
        Span::raw("  —  justfile: "),
        Span::styled(justfile, Style::default().fg(theme.info)),
        Span::raw(" ▾"),
        Span::raw("        "),
        Span::styled("?", Style::default().fg(theme.dim)),
        Span::raw(" help  "),
        Span::styled("q", Style::default().fg(theme.dim)),
        Span::raw(" quit"),
    ];

    if !app.startup_errors.is_empty() {
        spans.push(Span::raw("  |  "));
        spans.push(Span::styled(
            format!("{} load errors — press e", app.startup_errors.len()),
            Style::default()
                .fg(theme.error)
                .add_modifier(Modifier::BOLD),
        ));
    }

    f.render_widget(Paragraph::new(Line::from(spans)), area);
}
