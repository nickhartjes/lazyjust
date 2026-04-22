use crate::app::App;
use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub struct Panes {
    pub top_bar: Rect,
    pub list: Rect,
    pub right: Rect,
    pub status: Rect,
}

pub fn compute(area: Rect, app: &App) -> Panes {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(3),
            Constraint::Length(1),
        ])
        .split(area);

    let left_pct = (app.split_ratio * 100.0).round() as u16;
    let right_pct = 100 - left_pct;
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(left_pct),
            Constraint::Percentage(right_pct),
        ])
        .split(vertical[1]);

    Panes {
        top_bar: vertical[0],
        list: horizontal[0],
        right: horizontal[1],
        status: vertical[2],
    }
}
