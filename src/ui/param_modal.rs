use crate::app::types::Mode;
use crate::app::App;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let Mode::ParamInput {
        recipe_idx,
        values,
        cursor,
    } = &app.mode
    else {
        return;
    };
    let jf = match app.active_justfile() {
        Some(j) => j,
        None => return,
    };
    let recipe = match jf.recipes.get(*recipe_idx) {
        Some(r) => r,
        None => return,
    };

    f.render_widget(Clear, area);
    let mut lines = Vec::new();
    lines.push(Line::from(format!("run {}:", recipe.name)));
    lines.push(Line::from(""));
    for (i, p) in recipe.params.iter().enumerate() {
        let val = values.get(i).cloned().unwrap_or_default();
        let marker = if i == *cursor { ">" } else { " " };
        lines.push(Line::from(format!("{marker} {}: {}", p.name, val)));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(ratatui::text::Span::styled(
        "Tab next, Enter run, Esc cancel",
        Style::default().fg(Color::Gray),
    )));
    let block = Block::default().borders(Borders::ALL).title("params");
    f.render_widget(Paragraph::new(lines).block(block), area);
}
