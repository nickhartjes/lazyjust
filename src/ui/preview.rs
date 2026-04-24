use crate::app::App;
use crate::ui::focus::{focus_bar, is_right_active};
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};
use ratatui::Frame;

pub fn render(f: &mut Frame, area: Rect, app: &App, theme: &crate::theme::Theme) {
    let active = is_right_active(app.focus);
    let Some(r) = app.recipe_at_cursor() else {
        let line = Line::from(focus_bar(active, theme));
        f.render_widget(Paragraph::new(line), area);
        return;
    };

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(vec![
        focus_bar(active, theme),
        Span::raw(" "),
        Span::styled(
            r.name.clone(),
            Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
        ),
    ]));
    if let Some(doc) = &r.doc {
        lines.push(Line::from(Span::styled(
            format!("  {doc}"),
            Style::default().fg(theme.dim),
        )));
    }

    if r.has_deps() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  depends on",
            Style::default().fg(theme.dim),
        )));
        for dep in r.dep_names() {
            lines.push(Line::from(vec![
                Span::raw("    "),
                Span::styled("▸ ", Style::default().fg(theme.success)),
                Span::styled(dep.to_string(), Style::default().fg(theme.fg)),
            ]));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  command",
        Style::default().fg(theme.dim),
    )));
    for (i, cmd_line) in r.command_preview.lines().enumerate() {
        let prefix = if i == 0 {
            vec![
                Span::raw("    "),
                Span::styled("$ ", Style::default().fg(theme.info)),
            ]
        } else {
            vec![Span::raw("      ")]
        };
        let mut spans = prefix;
        spans.push(Span::styled(cmd_line.to_string(), Style::default().fg(theme.fg)));
        lines.push(Line::from(spans));
    }

    if !r.params.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  params",
            Style::default().fg(theme.dim),
        )));
        for p in &r.params {
            let mut spans = vec![
                Span::raw("    "),
                Span::styled(p.name.clone(), Style::default().fg(theme.fg)),
            ];
            if let Some(d) = &p.default {
                spans.push(Span::styled(
                    format!("  (default: {d})"),
                    Style::default().fg(theme.dim),
                ));
            }
            lines.push(Line::from(spans));
        }
    }

    let p = Paragraph::new(lines).wrap(Wrap { trim: false });
    f.render_widget(p, area);
}
