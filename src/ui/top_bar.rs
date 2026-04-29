use crate::app::App;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

pub fn render(f: &mut Frame, area: Rect, app: &App, theme: &crate::theme::Theme) {
    use crate::app::types::ListMode;
    if app.list_mode == ListMode::All {
        render_all_mode(f, area, app, theme);
    } else {
        render_active_mode(f, area, app, theme);
    }
}

fn render_all_mode(f: &mut Frame, area: Rect, app: &App, theme: &crate::theme::Theme) {
    let total_recipes: usize = app
        .justfiles
        .iter()
        .map(|j| j.recipes.len())
        .sum();
    let leading: Vec<Span> = vec![
        Span::styled("▌", Style::default().fg(theme.accent)),
        Span::raw(" "),
        Span::styled(
            "lazyjust",
            Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
        ),
        Span::styled("  · ", Style::default().fg(theme.dim)),
    ];

    let chrome_width: usize = leading
        .iter()
        .map(|s| s.content.chars().count())
        .sum::<usize>();
    let path_budget: usize = (area.width as usize).saturating_sub(chrome_width).max(8);

    let root_label =
        crate::ui::path_display::shorten(&app.discovery_root, path_budget);

    let trailing = format!(
        "  · {} justfiles, {} recipes",
        app.justfiles.len(),
        total_recipes
    );

    let errors_span: Option<Span> = (!app.startup_errors.is_empty()).then(|| {
        Span::styled(
            format!("   {} load errors ", app.startup_errors.len()),
            Style::default()
                .fg(theme.error)
                .bg(theme.bg)
                .add_modifier(Modifier::BOLD),
        )
    });

    let mut spans: Vec<Span> = Vec::with_capacity(8);
    spans.extend(leading);
    spans.push(Span::styled(root_label, Style::default().fg(theme.dim)));
    spans.push(Span::styled(trailing, Style::default().fg(theme.dim)));
    if let Some(err_span) = errors_span {
        spans.push(err_span);
    }
    f.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn render_active_mode(f: &mut Frame, area: Rect, app: &App, theme: &crate::theme::Theme) {
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
    let trailing: Vec<Span> = vec![
        Span::styled("  · ", Style::default().fg(theme.dim)),
        Span::styled(format!("{count} recipes"), Style::default().fg(theme.dim)),
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

#[cfg(test)]
mod tests {
    use crate::app::types::{Justfile, ListMode, Recipe};
    use crate::app::App;
    use crate::ui::icon_style::IconStyle;
    use ratatui::layout::Rect;
    use ratatui::Terminal;
    use std::path::PathBuf;

    fn r(n: &str) -> Recipe {
        Recipe {
            name: n.into(),
            module_path: vec![],
            group: None,
            params: vec![],
            doc: None,
            command_preview: String::new(),
            runs: vec![],
            dependencies: vec![],
        }
    }

    fn render_top_bar(app: &App, w: u16) -> String {
        let backend = ratatui::backend::TestBackend::new(w, 1);
        let mut term = Terminal::new(backend).unwrap();
        term.draw(|f| {
            super::render(f, Rect::new(0, 0, w, 1), app, &app.theme);
        })
        .unwrap();
        let buf = term.backend().buffer().clone();
        buf.content()
            .iter()
            .map(|c| c.symbol())
            .collect::<String>()
    }

    #[test]
    fn all_mode_top_bar_shows_total_counts_and_root() {
        let a = Justfile {
            path: PathBuf::from("/root/api/justfile"),
            recipes: vec![r("build"), r("test")],
            groups: vec![],
        };
        let b = Justfile {
            path: PathBuf::from("/root/web/justfile"),
            recipes: vec![r("dev")],
            groups: vec![],
        };
        let app = App::new(
            vec![a, b],
            vec![],
            0.3,
            crate::theme::registry::resolve(crate::theme::DEFAULT_THEME_NAME),
            crate::theme::DEFAULT_THEME_NAME.to_string(),
            IconStyle::Round,
            ListMode::All,
            PathBuf::from("/root"),
        );
        let s = render_top_bar(&app, 80);
        assert!(s.contains("2 justfiles"), "got {s:?}");
        assert!(s.contains("3 recipes"), "got {s:?}");
    }
}
