use crate::app::App;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let justfile = app
        .active_justfile()
        .map(|j| j.path.display().to_string())
        .unwrap_or_else(|| "<no justfile>".into());

    let spans = vec![
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

    let p = Paragraph::new(Line::from(spans));
    f.render_widget(p, area);
}
