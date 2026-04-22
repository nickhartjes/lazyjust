use lazyjust::app::reducer::reduce;
use lazyjust::app::types::{Justfile, Mode, Recipe};
use lazyjust::app::{Action, App};
use std::path::PathBuf;

fn make_app() -> App {
    let recipes = vec![
        Recipe {
            name: "build".into(),
            module_path: vec![],
            group: None,
            params: vec![],
            doc: None,
            command_preview: "cargo build".into(),
            runs: vec![],
        },
        Recipe {
            name: "test".into(),
            module_path: vec![],
            group: None,
            params: vec![],
            doc: None,
            command_preview: "cargo test".into(),
            runs: vec![],
        },
    ];
    let jf = Justfile {
        path: PathBuf::from("j"),
        recipes,
        groups: vec![],
    };
    App::new(vec![jf], vec![], 0.3)
}

#[test]
fn cursor_up_down_clamps() {
    let mut app = make_app();
    reduce(&mut app, Action::CursorDown);
    assert_eq!(app.list_cursor, 1);
    reduce(&mut app, Action::CursorDown);
    assert_eq!(app.list_cursor, 1); // clamped
    reduce(&mut app, Action::CursorUp);
    reduce(&mut app, Action::CursorUp);
    assert_eq!(app.list_cursor, 0);
}

#[test]
fn filter_flow() {
    let mut app = make_app();
    reduce(&mut app, Action::EnterFilter);
    assert_eq!(app.mode, Mode::FilterInput);
    reduce(&mut app, Action::FilterChar('t'));
    reduce(&mut app, Action::FilterChar('e'));
    assert_eq!(app.filter, "te");
    reduce(&mut app, Action::FilterBackspace);
    assert_eq!(app.filter, "t");
    reduce(&mut app, Action::CommitFilter);
    assert_eq!(app.mode, Mode::Normal);
    assert_eq!(app.filter, "t");
    reduce(&mut app, Action::CancelFilter);
    assert_eq!(app.filter, "");
    assert_eq!(app.mode, Mode::Normal);
}

#[test]
fn split_resize_clamps() {
    let mut app = make_app();
    for _ in 0..20 {
        reduce(&mut app, Action::GrowLeftPane);
    }
    assert!(app.split_ratio <= 0.70);
    for _ in 0..30 {
        reduce(&mut app, Action::ShrinkLeftPane);
    }
    assert!(app.split_ratio >= 0.15);
    reduce(&mut app, Action::ResetSplit);
    assert!((app.split_ratio - 0.30).abs() < 1e-6);
}
