use super::Theme;
use ratatui::style::Color;
use serde::Deserialize;

#[derive(Debug, thiserror::Error)]
pub enum ThemeError {
    #[error("theme parse error: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("invalid color {0:?}: expected #rrggbb, a named ratatui color, or a 0-255 index")]
    Color(String),
}

#[derive(Debug, Deserialize)]
struct ThemeFile {
    name: String,
    bg: String,
    fg: String,
    dim: String,
    accent: String,
    highlight: String,
    selected_fg: String,
    success: String,
    warn: String,
    error: String,
    running: String,
    info: String,
    badge_bg: String,
    badge_fg: String,
}

pub fn parse_theme(input: &str) -> Result<Theme, ThemeError> {
    let f: ThemeFile = toml::from_str(input)?;
    Ok(Theme {
        name: f.name,
        bg: parse_color(&f.bg)?,
        fg: parse_color(&f.fg)?,
        dim: parse_color(&f.dim)?,
        accent: parse_color(&f.accent)?,
        highlight: parse_color(&f.highlight)?,
        selected_fg: parse_color(&f.selected_fg)?,
        success: parse_color(&f.success)?,
        warn: parse_color(&f.warn)?,
        error: parse_color(&f.error)?,
        running: parse_color(&f.running)?,
        info: parse_color(&f.info)?,
        badge_bg: parse_color(&f.badge_bg)?,
        badge_fg: parse_color(&f.badge_fg)?,
    })
}

fn parse_color(s: &str) -> Result<Color, ThemeError> {
    let t = s.trim();
    if let Ok(n) = t.parse::<u8>() {
        return Ok(Color::Indexed(n));
    }
    if let Some(hex) = t.strip_prefix('#') {
        if hex.len() == 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| ThemeError::Color(s.into()))?;
            let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| ThemeError::Color(s.into()))?;
            let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| ThemeError::Color(s.into()))?;
            return Ok(Color::Rgb(r, g, b));
        }
        return Err(ThemeError::Color(s.into()));
    }
    Ok(match t.to_ascii_lowercase().as_str() {
        "black" => Color::Black,
        "red" => Color::Red,
        "green" => Color::Green,
        "yellow" => Color::Yellow,
        "blue" => Color::Blue,
        "magenta" => Color::Magenta,
        "cyan" => Color::Cyan,
        "gray" | "grey" => Color::Gray,
        "dark_gray" | "darkgray" | "dark_grey" | "darkgrey" => Color::DarkGray,
        "light_red" | "lightred" | "bright_red" | "brightred" => Color::LightRed,
        "light_green" | "lightgreen" | "bright_green" | "brightgreen" => Color::LightGreen,
        "light_yellow" | "lightyellow" | "bright_yellow" | "brightyellow" => Color::LightYellow,
        "light_blue" | "lightblue" | "bright_blue" | "brightblue" => Color::LightBlue,
        "light_magenta" | "lightmagenta" | "bright_magenta" | "brightmagenta" => {
            Color::LightMagenta
        }
        "light_cyan" | "lightcyan" | "bright_cyan" | "brightcyan" => Color::LightCyan,
        "white" => Color::White,
        _ => return Err(ThemeError::Color(s.into())),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_parses() {
        assert_eq!(parse_color("#89b4fa").unwrap(), Color::Rgb(137, 180, 250));
    }

    #[test]
    fn named_parses_case_insensitive() {
        assert_eq!(parse_color("Blue").unwrap(), Color::Blue);
        assert_eq!(parse_color("bright_red").unwrap(), Color::LightRed);
    }

    #[test]
    fn palette_index_parses() {
        assert_eq!(parse_color("21").unwrap(), Color::Indexed(21));
    }

    #[test]
    fn empty_string_is_error() {
        assert!(matches!(parse_color(""), Err(ThemeError::Color(_))));
    }

    #[test]
    fn short_hex_is_error() {
        assert!(matches!(parse_color("#abc"), Err(ThemeError::Color(_))));
    }

    #[test]
    fn unknown_name_is_error() {
        assert!(matches!(parse_color("mauve"), Err(ThemeError::Color(_))));
    }

    #[test]
    fn theme_round_trips() {
        let toml = "name = \"test\"\nbg = \"#000000\"\nfg = \"#ffffff\"\ndim = \"gray\"\naccent = \"cyan\"\nhighlight = \"#202020\"\nselected_fg = \"white\"\nsuccess = \"green\"\nwarn = \"yellow\"\nerror = \"red\"\nrunning = \"blue\"\ninfo = \"cyan\"\nbadge_bg = \"dark_gray\"\nbadge_fg = \"white\"";
        let t = parse_theme(toml).unwrap();
        assert_eq!(t.name, "test");
        assert_eq!(t.bg, Color::Rgb(0, 0, 0));
        assert_eq!(t.dim, Color::Gray);
        assert_eq!(t.accent, Color::Cyan);
    }

    #[test]
    fn missing_slot_fails() {
        let toml = "name = \"test\"\nbg = \"#000000\"\nfg = \"#ffffff\"\ndim = \"gray\"\naccent = \"cyan\"\nhighlight = \"#202020\"\nselected_fg = \"white\"\nsuccess = \"green\"\nwarn = \"yellow\"\nerror = \"red\"\nrunning = \"blue\"\ninfo = \"cyan\"\nbadge_bg = \"dark_gray\"";
        assert!(matches!(parse_theme(toml), Err(ThemeError::Parse(_))));
    }
}
