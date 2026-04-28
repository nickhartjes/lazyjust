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
    let count = jf.map(|j| j.recipes.len()).unwrap_or(0);

    // Build every span *except* the path first, so we know exactly how
    // many columns the path itself may consume.
    let leading: Vec<Span> = vec![
        Span::styled("▌", Style::default().fg(theme.accent)),
        Span::raw(" "),
        Span::styled(
            "lazyjust",
            Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
        ),
        Span::styled("  · ", Style::default().fg(theme.dim)),
    ];
    let recipes_text = format!("{count} recipes");
    let trailing: Vec<Span> = vec![
        Span::styled("  · ", Style::default().fg(theme.dim)),
        Span::styled(recipes_text.clone(), Style::default().fg(theme.dim)),
    ];
    let errors_span: Option<Span> = (!app.startup_errors.is_empty()).then(|| {
        Span::styled(
            format!(" {} load errors ", app.startup_errors.len()),
            Style::default()
                .fg(theme.error)
                .bg(theme.bg)
                .add_modifier(Modifier::BOLD),
        )
    });

    let chrome_width: usize = leading
        .iter()
        .chain(trailing.iter())
        .chain(errors_span.iter())
        .map(|s| s.content.chars().count())
        .sum::<usize>()
        + if errors_span.is_some() { 3 } else { 0 }; // "   " separator before errors

    let path_budget: usize = (cols[0].width as usize)
        .saturating_sub(chrome_width)
        .max(16); // never collapse below "<root>/…/<filename>"-ish space

    let path = match jf {
        Some(j) => crate::ui::path_display::shorten(&j.path, path_budget),
        None => "<no justfile>".to_string(),
    };

    let mut spans: Vec<Span> = Vec::with_capacity(8);
    spans.extend(leading);
    spans.push(Span::styled(path, Style::default().fg(theme.dim)));
    spans.extend(trailing);
    if let Some(err_span) = errors_span {
        spans.push(Span::raw("   "));
        spans.push(err_span);
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
