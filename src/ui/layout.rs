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

    let total = vertical[1].width;
    let mut left = ((app.split_ratio * total as f32).round() as u16).max(28);
    if total.saturating_sub(left) < 48 {
        left = total.saturating_sub(48);
    }
    let right = total.saturating_sub(left);
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(left), Constraint::Length(right)])
        .split(vertical[1]);

    Panes {
        top_bar: vertical[0],
        list: horizontal[0],
        right: horizontal[1],
        status: vertical[2],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::layout::Rect;

    fn rect(w: u16, h: u16) -> Rect { Rect { x: 0, y: 0, width: w, height: h } }

    #[test]
    fn list_pane_respects_min_left_cols() {
        let app = fake_app_with_split(0.01);
        let panes = compute(rect(120, 30), &app);
        assert!(panes.list.width >= 28, "list {} < 28", panes.list.width);
    }

    #[test]
    fn right_pane_respects_min_right_cols() {
        let app = fake_app_with_split(0.99);
        let panes = compute(rect(120, 30), &app);
        assert!(panes.right.width >= 48, "right {} < 48", panes.right.width);
    }

    fn fake_app_with_split(ratio: f32) -> crate::app::App {
        crate::app::App::new(
            vec![],
            vec![],
            ratio,
            crate::theme::registry::resolve(crate::theme::DEFAULT_THEME_NAME),
            crate::theme::DEFAULT_THEME_NAME.into(),
        )
    }
}
