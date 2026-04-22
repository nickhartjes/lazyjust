pub mod layout;
pub mod list;
pub mod preview;
pub mod status_bar;
pub mod top_bar;

use crate::app::App;
use ratatui::Frame;

pub fn render(f: &mut Frame, app: &App) {
    let panes = layout::compute(f.size(), app);
    top_bar::render(f, panes.top_bar, app);
    list::render(f, panes.list, app);
    preview::render(f, panes.right, app);
    status_bar::render(f, panes.status, app);
}
