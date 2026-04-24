//! Color themes. Each theme is a set of 14 semantic color slots used
//! across the UI. Themes load from TOML — either built-in (compiled in
//! via `include_str!`) or user files under `<config_dir>/lazyrust/themes/`.

pub mod builtin;
mod parse;
pub mod registry;

pub use parse::{parse_theme, ThemeError};

use ratatui::style::Color;

pub const DEFAULT_THEME_NAME: &str = "tokyo-night";

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub bg: Color,
    pub fg: Color,
    pub dim: Color,
    pub accent: Color,
    pub highlight: Color,
    pub selected_fg: Color,
    pub success: Color,
    pub warn: Color,
    pub error: Color,
    pub running: Color,
    pub info: Color,
    pub badge_bg: Color,
    pub badge_fg: Color,
}
