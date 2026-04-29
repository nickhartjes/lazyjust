//! Cross-layer smoke: configure list_mode = all, exercise cursor + recipe
//! lookup, confirm `d` no-ops with status message.

use lazyjust::app::reducer::reduce;
use lazyjust::app::types::{Justfile, ListMode, Mode, Recipe};
use lazyjust::app::{Action, App};
use std::path::PathBuf;

fn recipe(name: &str) -> Recipe {
    Recipe {
        name: name.into(),
        module_path: vec![],
        group: None,
        params: vec![],
        doc: None,
        command_preview: String::new(),
        runs: vec![],
        dependencies: vec![],
    }
}

fn make_app() -> App {
    let api = Justfile {
        path: PathBuf::from("/root/api/justfile"),
        recipes: vec![recipe("build"), recipe("test")],
        groups: vec![],
    };
    let web = Justfile {
        path: PathBuf::from("/root/web/justfile"),
        recipes: vec![recipe("dev")],
        groups: vec![],
    };
    App::new(
        vec![api, web],
        vec![],
        0.3,
        lazyjust::theme::registry::resolve(lazyjust::theme::DEFAULT_THEME_NAME),
        lazyjust::theme::DEFAULT_THEME_NAME.to_string(),
        lazyjust::ui::icon_style::IconStyle::Round,
        ListMode::All,
        PathBuf::from("/root"),
    )
}

#[test]
fn cursor_traverses_recipes_across_justfiles() {
    let mut app = make_app();
    assert_eq!(app.recipe_at_cursor().unwrap().name, "build");
    reduce(&mut app, Action::CursorDown);
    assert_eq!(app.recipe_at_cursor().unwrap().name, "test");
    reduce(&mut app, Action::CursorDown);
    assert_eq!(app.recipe_at_cursor().unwrap().name, "dev");
    reduce(&mut app, Action::CursorDown);
    assert_eq!(app.recipe_at_cursor().unwrap().name, "dev"); // clamps
}

#[test]
fn dropdown_is_disabled_with_status_message() {
    let mut app = make_app();
    reduce(&mut app, Action::OpenDropdown);
    assert!(matches!(app.mode, Mode::Normal));
    assert_eq!(
        app.status_message.as_deref(),
        Some("dropdown disabled in list_mode=all")
    );
}

#[test]
fn switching_back_to_active_resets_cursor_and_filters_to_active_justfile() {
    let mut app = make_app();
    reduce(&mut app, Action::CursorDown);
    reduce(&mut app, Action::CursorDown); // on "dev" from web
    assert_eq!(app.recipe_at_cursor().unwrap().name, "dev");
    reduce(&mut app, Action::SetListMode(ListMode::Active));
    // active_justfile is still 0 → recipes from api
    assert_eq!(app.recipe_at_cursor().unwrap().name, "build");
    assert_eq!(app.list_cursor, 0);
}
