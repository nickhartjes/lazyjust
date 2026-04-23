use crate::app::App;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let justfile = app
        .active_justfile()
        .map(|j| j.path.display().to_string())
        .unwrap_or_else(|| "<no justfile>".into());

    let mut spans = vec![
        Span::styled("lazyjust", Style::default().fg(Color::Cyan)),
        Span::raw("  —  justfile: "),
        Span::styled(justfile, Style::default().fg(Color::Yellow)),
        Span::raw(" ▾"),
        Span::raw("        "),
        Span::styled("?", Style::default().fg(Color::Gray)),
        Span::raw(" help  "),
        Span::styled("q", Style::default().fg(Color::Gray)),
        Span::raw(" quit"),
    ];

    if !app.startup_errors.is_empty() {
        spans.push(Span::raw("  |  "));
        spans.push(Span::styled(
            format!("{} load errors — press e", app.startup_errors.len()),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ));
    }

    f.render_widget(Paragraph::new(Line::from(spans)), area);
}
