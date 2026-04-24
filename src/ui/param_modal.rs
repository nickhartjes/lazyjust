use crate::app::types::Mode;
use crate::app::App;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

pub fn render(f: &mut Frame, app: &App, area: Rect, theme: &crate::theme::Theme) {
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

    crate::ui::modal_base::clear(f, area);
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
        Style::default().fg(theme.dim),
    )));
    f.render_widget(
        Paragraph::new(lines).block(crate::ui::modal_base::block("params", theme)),
        area,
    );
}
