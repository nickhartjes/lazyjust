// Filled in by Task 4.
use super::{Theme, DEFAULT_THEME_NAME};

pub fn resolve(_name: &str) -> Theme {
    super::parse_theme(
        r#"
        name = "placeholder"
        bg = "black"
        fg = "white"
        dim = "gray"
        accent = "cyan"
        highlight = "dark_gray"
        selected_fg = "white"
        success = "green"
        warn = "yellow"
        error = "red"
        running = "blue"
        info = "cyan"
        badge_bg = "dark_gray"
        badge_fg = "white"
    "#,
    )
    .expect("placeholder theme must parse")
}

pub fn list() -> Vec<String> {
    vec![DEFAULT_THEME_NAME.to_string()]
}
