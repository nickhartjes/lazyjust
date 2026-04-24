use lazyjust::config::paths::OVERRIDE_ENV;
use lazyjust::theme::registry;
use std::sync::{Mutex, OnceLock};

fn guard() -> &'static Mutex<()> {
    static M: OnceLock<Mutex<()>> = OnceLock::new();
    M.get_or_init(|| Mutex::new(()))
}

#[test]
fn user_theme_shadows_builtin_via_registry() {
    let _lock = guard().lock().unwrap_or_else(|e| e.into_inner());
    let tmp = tempfile::tempdir().unwrap();
    let themes = tmp.path().join("themes");
    std::fs::create_dir_all(&themes).unwrap();
    std::fs::write(
        themes.join("tokyo-night.toml"),
        "name = \"Shadowed Tokyo\"\nbg = \"#000001\"\nfg = \"#ffffff\"\ndim = \"gray\"\naccent = \"cyan\"\nhighlight = \"dark_gray\"\nselected_fg = \"white\"\nsuccess = \"green\"\nwarn = \"yellow\"\nerror = \"red\"\nrunning = \"blue\"\ninfo = \"cyan\"\nbadge_bg = \"dark_gray\"\nbadge_fg = \"white\"\n",
    )
    .unwrap();

    std::env::set_var(OVERRIDE_ENV, tmp.path());
    let t = registry::resolve("tokyo-night");
    std::env::remove_var(OVERRIDE_ENV);

    assert_eq!(t.name, "Shadowed Tokyo");
}

#[test]
fn list_includes_user_theme_via_registry() {
    let _lock = guard().lock().unwrap_or_else(|e| e.into_inner());
    let tmp = tempfile::tempdir().unwrap();
    let themes = tmp.path().join("themes");
    std::fs::create_dir_all(&themes).unwrap();
    std::fs::write(
        themes.join("solarpunk.toml"),
        "name = \"Solarpunk\"\nbg = \"#000001\"\nfg = \"#ffffff\"\ndim = \"gray\"\naccent = \"cyan\"\nhighlight = \"dark_gray\"\nselected_fg = \"white\"\nsuccess = \"green\"\nwarn = \"yellow\"\nerror = \"red\"\nrunning = \"blue\"\ninfo = \"cyan\"\nbadge_bg = \"dark_gray\"\nbadge_fg = \"white\"\n",
    )
    .unwrap();

    std::env::set_var(OVERRIDE_ENV, tmp.path());
    let names = registry::list();
    std::env::remove_var(OVERRIDE_ENV);

    assert!(names.iter().any(|n| n == "solarpunk"));
    assert!(names.iter().any(|n| n == "tokyo-night"));
}
