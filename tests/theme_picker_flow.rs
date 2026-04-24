use lazyjust::app::action::Action;
use lazyjust::app::reducer::reduce;
use lazyjust::app::types::{Justfile, Recipe};
use lazyjust::app::App;
use lazyjust::theme::{registry::resolve, DEFAULT_THEME_NAME};
use lazyjust::ui::icon_style::IconStyle;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

fn lock() -> &'static Mutex<()> {
    static M: OnceLock<Mutex<()>> = OnceLock::new();
    M.get_or_init(|| Mutex::new(()))
}

fn minimal_app() -> App {
    let recipes = vec![Recipe {
        name: "build".into(),
        module_path: vec![],
        group: None,
        params: vec![],
        dependencies: vec![],
        doc: None,
        command_preview: "cargo build".into(),
        runs: vec![],
    }];
    let jf = Justfile {
        path: PathBuf::from("./justfile"),
        recipes,
        groups: vec![],
    };
    App::new(
        vec![jf],
        vec![],
        0.3,
        resolve(DEFAULT_THEME_NAME),
        DEFAULT_THEME_NAME.into(),
        IconStyle::Round,
    )
}

#[test]
fn picker_cancel_reverts_theme_no_write() {
    let _g = lock().lock().unwrap_or_else(|e| e.into_inner());
    let tmp = tempfile::tempdir().unwrap();
    std::env::set_var("LAZYJUST_CONFIG_DIR", tmp.path());
    let cfg_path = tmp.path().join("config.toml");
    let original = "[ui]\ntheme = \"tokyo-night\"\n";
    std::fs::write(&cfg_path, original).unwrap();

    let mut app = minimal_app();
    reduce(&mut app, Action::OpenThemePicker);
    reduce(&mut app, Action::PickerMove(1));
    // live preview applied — name should differ from default (tokyo-night)
    assert_ne!(app.theme_name, "tokyo-night");
    reduce(&mut app, Action::PickerCancel);
    // cancel reverts to the original app theme
    assert_eq!(app.theme_name, "tokyo-night");
    // file untouched
    assert_eq!(std::fs::read_to_string(&cfg_path).unwrap(), original);

    std::env::remove_var("LAZYJUST_CONFIG_DIR");
}

#[test]
fn picker_confirm_writes_theme_preserving_other_keys() {
    let _g = lock().lock().unwrap_or_else(|e| e.into_inner());
    let tmp = tempfile::tempdir().unwrap();
    std::env::set_var("LAZYJUST_CONFIG_DIR", tmp.path());
    let cfg_path = tmp.path().join("config.toml");
    let original =
        "# user comment\n[ui]\ntheme = \"tokyo-night\"\n\n[engine]\nrender_throttle_ms = 8\n";
    std::fs::write(&cfg_path, original).unwrap();

    let mut app = minimal_app();
    reduce(&mut app, Action::OpenThemePicker);
    reduce(&mut app, Action::PickerMove(1));
    reduce(&mut app, Action::PickerConfirm);

    let after = std::fs::read_to_string(&cfg_path).unwrap();
    // user comment preserved
    assert!(
        after.contains("# user comment"),
        "comment dropped: {after:?}"
    );
    // other section preserved
    assert!(
        after.contains("render_throttle_ms = 8"),
        "engine dropped: {after:?}"
    );
    // theme no longer tokyo-night
    assert!(
        !after.contains("theme = \"tokyo-night\""),
        "old theme kept: {after:?}"
    );

    std::env::remove_var("LAZYJUST_CONFIG_DIR");
}
