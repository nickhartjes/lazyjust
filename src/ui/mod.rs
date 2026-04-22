pub mod layout;
pub mod status_bar;
pub mod top_bar;

use crate::app::App;
use ratatui::Frame;

pub fn render(f: &mut Frame, app: &App) {
    let panes = layout::compute(f.size(), app);
    top_bar::render(f, panes.top_bar, app);
    status_bar::render(f, panes.status, app);
    // list + right pane added in subsequent tasks; render a placeholder box to show layout.
    use ratatui::widgets::{Block, Borders};
    f.render_widget(
        Block::default().borders(Borders::ALL).title("recipes"),
        panes.list,
    );
    f.render_widget(
        Block::default().borders(Borders::ALL).title("preview"),
        panes.right,
    );
}
