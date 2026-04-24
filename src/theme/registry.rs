//! Resolves a theme name to a `Theme`. User theme files under
//! `<config_dir>/lazyjust/themes/` shadow built-ins of the same name.
//! Missing / invalid name logs a warning and falls back to the default.

use super::{builtin::BUILTIN_THEMES, parse_theme, Theme, DEFAULT_THEME_NAME};
use std::collections::BTreeSet;
use std::path::PathBuf;

pub fn resolve(name: &str) -> Theme {
    // 1. User theme file, if present.
    if let Some(t) = load_user_theme(name) {
        return t;
    }
    // 2. Built-in with matching name.
    if let Some(t) = load_builtin(name) {
        return t;
    }
    // 3. Fallback to default.
    if name != DEFAULT_THEME_NAME {
        tracing::warn!(
            target: "lazyjust::theme",
            requested = %name,
            fallback = %DEFAULT_THEME_NAME,
            "theme not found, using default",
        );
    }
    load_builtin(DEFAULT_THEME_NAME).expect("default theme must be a built-in")
}

/// All theme names available — built-in + user — deduped and sorted.
pub fn list() -> Vec<String> {
    let mut out: BTreeSet<String> = BTreeSet::new();
    for (n, _) in BUILTIN_THEMES {
        out.insert((*n).to_string());
    }
    if let Some(dir) = user_themes_dir() {
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for e in entries.flatten() {
                let p = e.path();
                if p.extension().and_then(|s| s.to_str()) == Some("toml") {
                    if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
                        out.insert(stem.to_string());
                    }
                }
            }
        }
    }
    out.into_iter().collect()
}

fn user_themes_dir() -> Option<PathBuf> {
    Some(crate::config::paths::user_themes_dir())
}

fn load_user_theme(name: &str) -> Option<Theme> {
    let dir = user_themes_dir()?;
    let path = dir.join(format!("{name}.toml"));
    let contents = std::fs::read_to_string(&path).ok()?;
    match parse_theme(&contents) {
        Ok(t) => Some(t),
        Err(e) => {
            tracing::warn!(
                target: "lazyjust::theme",
                path = %path.display(),
                error = %e,
                "user theme failed to parse; falling through to built-in",
            );
            None
        }
    }
}

fn load_builtin(name: &str) -> Option<Theme> {
    BUILTIN_THEMES
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, raw)| parse_theme(raw).expect("built-in theme must parse"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::paths::OVERRIDE_ENV;
    use std::sync::{Mutex, OnceLock};

    fn guard() -> &'static Mutex<()> {
        static M: OnceLock<Mutex<()>> = OnceLock::new();
        M.get_or_init(|| Mutex::new(()))
    }

    fn with_themes_dir<T>(files: &[(&str, &str)], body: impl FnOnce() -> T) -> T {
        let _lock = guard().lock().unwrap_or_else(|e| e.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        let themes = tmp.path().join("themes");
        std::fs::create_dir_all(&themes).unwrap();
        for (name, contents) in files {
            std::fs::write(themes.join(format!("{name}.toml")), contents).unwrap();
        }
        std::env::set_var(OVERRIDE_ENV, tmp.path());
        let out = body();
        std::env::remove_var(OVERRIDE_ENV);
        out
    }

    #[test]
    fn resolves_builtin_by_name() {
        let t = resolve("tokyo-night");
        assert_eq!(t.name, "Tokyo Night");
    }

    #[test]
    fn unknown_name_falls_back_to_default() {
        let t = resolve("this-does-not-exist");
        assert_eq!(t.name, "Tokyo Night");
    }

    #[test]
    fn user_theme_shadows_builtin() {
        let custom = "name = \"Custom Nord\"\nbg = \"#000001\"\nfg = \"#ffffff\"\ndim = \"gray\"\naccent = \"cyan\"\nhighlight = \"dark_gray\"\nselected_fg = \"white\"\nsuccess = \"green\"\nwarn = \"yellow\"\nerror = \"red\"\nrunning = \"blue\"\ninfo = \"cyan\"\nbadge_bg = \"dark_gray\"\nbadge_fg = \"white\"\n";
        with_themes_dir(&[("nord", custom)], || {
            let t = resolve("nord");
            assert_eq!(t.name, "Custom Nord");
        });
    }

    #[test]
    fn malformed_user_theme_falls_through_to_builtin() {
        with_themes_dir(&[("nord", "broken = = =")], || {
            let t = resolve("nord");
            assert_eq!(t.name, "Nord");
        });
    }

    #[test]
    fn list_includes_builtins_and_user_themes() {
        let custom = "name = \"Ocean\"\nbg = \"#000001\"\nfg = \"#ffffff\"\ndim = \"gray\"\naccent = \"cyan\"\nhighlight = \"dark_gray\"\nselected_fg = \"white\"\nsuccess = \"green\"\nwarn = \"yellow\"\nerror = \"red\"\nrunning = \"blue\"\ninfo = \"cyan\"\nbadge_bg = \"dark_gray\"\nbadge_fg = \"white\"\n";
        with_themes_dir(&[("ocean", custom)], || {
            let names = list();
            assert!(names.iter().any(|n| n == "tokyo-night"));
            assert!(names.iter().any(|n| n == "ocean"));
            let mut expected = names.clone();
            expected.sort();
            expected.dedup();
            assert_eq!(names, expected);
        });
    }
}
