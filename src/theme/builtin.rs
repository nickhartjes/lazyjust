//! Built-in themes. Each entry is (name, raw TOML string) embedded at
//! compile time via `include_str!`. Loaded through the registry in
//! `super::registry`.

pub const BUILTIN_THEMES: &[(&str, &str)] = &[
    (
        "catppuccin-latte",
        include_str!("../../assets/themes/catppuccin-latte.toml"),
    ),
    (
        "catppuccin-frappe",
        include_str!("../../assets/themes/catppuccin-frappe.toml"),
    ),
    (
        "catppuccin-macchiato",
        include_str!("../../assets/themes/catppuccin-macchiato.toml"),
    ),
    (
        "catppuccin-mocha",
        include_str!("../../assets/themes/catppuccin-mocha.toml"),
    ),
    (
        "tokyo-night",
        include_str!("../../assets/themes/tokyo-night.toml"),
    ),
    (
        "gruvbox-dark",
        include_str!("../../assets/themes/gruvbox-dark.toml"),
    ),
    ("dracula", include_str!("../../assets/themes/dracula.toml")),
    ("nord", include_str!("../../assets/themes/nord.toml")),
    (
        "solarized-dark",
        include_str!("../../assets/themes/solarized-dark.toml"),
    ),
    (
        "one-dark",
        include_str!("../../assets/themes/one-dark.toml"),
    ),
    (
        "mono-amber",
        include_str!("../../assets/themes/mono-amber.toml"),
    ),
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::parse_theme;

    #[test]
    fn every_builtin_parses() {
        for (name, toml) in BUILTIN_THEMES {
            parse_theme(toml)
                .unwrap_or_else(|e| panic!("built-in {name:?} failed to parse: {e}"));
        }
    }

    #[test]
    fn default_name_is_registered() {
        assert!(BUILTIN_THEMES
            .iter()
            .any(|(n, _)| *n == super::super::DEFAULT_THEME_NAME));
    }

    #[test]
    fn all_names_unique() {
        let mut seen = std::collections::HashSet::new();
        for (n, _) in BUILTIN_THEMES {
            assert!(seen.insert(*n), "duplicate theme name: {n}");
        }
    }
}
