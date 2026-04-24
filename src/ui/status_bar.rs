use crate::app::types::Mode;
use crate::app::App;
use ratatui::layout::Rect;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let text = match &app.mode {
        Mode::Normal => "↑↓ j/k move   / filter   Enter run   Shift+Enter/r new   K kill   x close   ? help   q quit".to_string(),
        Mode::FilterInput => format!("Filter: {}_   (Enter commit, Esc clear)", app.filter),
        Mode::Help { .. } => "Help — Esc / q to close".into(),
        Mode::Confirm { prompt, .. } => format!("{}  [y]es / [n]o", prompt),
        Mode::Dropdown { filter, .. } => format!("Justfile: {}_   (Enter pick, Esc cancel)", filter),
        Mode::ParamInput { values, cursor, .. } => {
            format!("Param {}/{} = {}", cursor + 1, values.len().max(1), values.get(*cursor).cloned().unwrap_or_default())
        }
        Mode::ErrorsList => "Load errors — Esc / q / e to close".into(),
        Mode::ThemePicker { highlighted, names, .. } => {
            let name = names.get(*highlighted).map(|s| s.as_str()).unwrap_or("");
            format!("Theme: {}   ↑↓ j/k move   Enter confirm   Esc cancel", name)
        }
    };
    let msg = app
        .status_message
        .as_deref()
        .map(|m| format!("  |  {}", m))
        .unwrap_or_default();
    f.render_widget(Paragraph::new(format!("{text}{msg}")), area);
}
