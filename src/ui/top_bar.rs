use crate::app::App;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

pub fn render(f: &mut Frame, area: Rect, app: &App, theme: &crate::theme::Theme) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(1), Constraint::Length(badge_width(app))])
        .split(area);

    let jf = app.active_justfile();
    let path = jf
        .map(|j| j.path.display().to_string())
        .unwrap_or_else(|| "<no justfile>".into());
    let count = jf.map(|j| j.recipes.len()).unwrap_or(0);

    let mut spans: Vec<Span> = vec![
        Span::styled("▌", Style::default().fg(theme.accent)),
        Span::raw(" "),
        Span::styled(
            "lazyjust",
            Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
        ),
        Span::styled("  · ", Style::default().fg(theme.dim)),
        Span::styled(path, Style::default().fg(theme.dim)),
        Span::styled("  · ", Style::default().fg(theme.dim)),
        Span::styled(format!("{count} recipes"), Style::default().fg(theme.dim)),
    ];
    if !app.startup_errors.is_empty() {
        spans.push(Span::raw("   "));
        spans.push(Span::styled(
            format!(" {} load errors ", app.startup_errors.len()),
            Style::default()
                .fg(theme.error)
                .bg(theme.bg)
                .add_modifier(Modifier::BOLD),
        ));
    }
    f.render_widget(Paragraph::new(Line::from(spans)), cols[0]);

    if let Some(j) = jf {
        let parent = j
            .path
            .parent()
            .and_then(|p| p.file_name())
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        if !parent.is_empty() {
            let pill = Line::from(Span::styled(
                format!(" {parent} "),
                Style::default().fg(theme.badge_fg).bg(theme.badge_bg),
            ));
            let right = Paragraph::new(pill).alignment(Alignment::Right);
            f.render_widget(right, cols[1]);
        }
    }
}

fn badge_width(app: &App) -> u16 {
    app.active_justfile()
        .and_then(|j| j.path.parent())
        .and_then(|p| p.file_name())
        .map(|s| s.to_string_lossy().chars().count() as u16 + 2)
        .unwrap_or(0)
}
