use crate::app::App;
use crate::ui::focus::{is_right_active, pane_block};
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};
use ratatui::Frame;

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let block = pane_block("preview", is_right_active(app.focus));
    let Some(r) = app.recipe_at_cursor() else {
        f.render_widget(block, area);
        return;
    };

    let mut lines = Vec::new();
    lines.push(Line::from(Span::styled(
        format!("recipe: {}", r.name),
        Style::default().fg(Color::Yellow),
    )));
    if let Some(doc) = &r.doc {
        lines.push(Line::from(Span::styled(
            doc.clone(),
            Style::default().fg(Color::Gray),
        )));
    }
    if !r.params.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from("params:"));
        for p in &r.params {
            let default = p
                .default
                .as_ref()
                .map(|d| format!("={d}"))
                .unwrap_or_default();
            lines.push(Line::from(format!("  {}{}", p.name, default)));
        }
    }
    lines.push(Line::from(""));
    lines.push(Line::from("command:"));
    for cmd_line in r.command_preview.lines() {
        lines.push(Line::from(format!("  {cmd_line}")));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Enter to run",
        Style::default().fg(Color::Cyan),
    )));

    let p = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });
    f.render_widget(p, area);
}
