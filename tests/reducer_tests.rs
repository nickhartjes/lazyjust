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
            dependencies: Vec::new(),
        },
        Recipe {
            name: "test".into(),
            module_path: vec![],
            group: None,
            params: vec![],
            doc: None,
            command_preview: "cargo test".into(),
            runs: vec![],
            dependencies: Vec::new(),
        },
    ];
    let jf = Justfile {
        path: PathBuf::from("j"),
        recipes,
        groups: vec![],
    };
    App::new(
        vec![jf],
        vec![],
        0.3,
        lazyjust::theme::registry::resolve(lazyjust::theme::DEFAULT_THEME_NAME),
        lazyjust::theme::DEFAULT_THEME_NAME.to_string(),
        lazyjust::ui::icon_style::IconStyle::Round,
        lazyjust::app::types::ListMode::Active,
        std::path::PathBuf::from("."),
    )
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

#[test]
fn session_exited_transitions_running_to_exited() {
    use lazyjust::app::types::{SessionMeta, Status};
    use std::time::Instant;

    let mut app = make_app();
    app.sessions.push(SessionMeta {
        id: 1,
        recipe_name: "build".into(),
        command_line: "just build".into(),
        status: Status::Running,
        unread: false,
        started_at: Instant::now(),
        log_path: PathBuf::from("/tmp/x.log"),
        pid: None,
    });

    reduce(&mut app, Action::SessionExited { id: 1, code: 7 });
    let s = app.sessions.iter().find(|s| s.id == 1).unwrap();
    assert_eq!(s.status, Status::Exited { code: 7 });
    assert!(s.unread);
}

#[test]
fn recipe_exited_transitions_running_to_shell_after_exit() {
    use lazyjust::app::types::{SessionMeta, Status};
    use std::time::Instant;

    let mut app = make_app();
    app.sessions.push(SessionMeta {
        id: 2,
        recipe_name: "deploy".into(),
        command_line: "just deploy".into(),
        status: Status::Running,
        unread: false,
        started_at: Instant::now(),
        log_path: PathBuf::from("/tmp/y.log"),
        pid: None,
    });

    reduce(&mut app, Action::RecipeExited { id: 2, code: 0 });
    let s = app.sessions.iter().find(|s| s.id == 2).unwrap();
    assert_eq!(s.status, Status::ShellAfterExit { code: 0 });
    assert!(s.unread); // not active, so unread
}

#[test]
fn session_exited_idempotent_from_exited_state() {
    use lazyjust::app::types::{SessionMeta, Status};
    use std::time::Instant;

    let mut app = make_app();
    app.sessions.push(SessionMeta {
        id: 3,
        recipe_name: "test".into(),
        command_line: "just test".into(),
        status: Status::Exited { code: 5 },
        unread: true,
        started_at: Instant::now(),
        log_path: PathBuf::from("/tmp/z.log"),
        pid: None,
    });

    reduce(&mut app, Action::SessionExited { id: 3, code: 0 });
    let s = app.sessions.iter().find(|s| s.id == 3).unwrap();
    assert_eq!(s.status, Status::Exited { code: 5 }); // unchanged
}

#[test]
fn mark_read_unread_flip() {
    use lazyjust::app::types::{SessionMeta, Status};
    use std::time::Instant;

    let mut app = make_app();
    app.sessions.push(SessionMeta {
        id: 4,
        recipe_name: "x".into(),
        command_line: "x".into(),
        status: Status::Running,
        unread: true,
        started_at: Instant::now(),
        log_path: PathBuf::from("/tmp/w.log"),
        pid: None,
    });

    reduce(&mut app, Action::MarkRead(4));
    assert!(!app.sessions.iter().find(|s| s.id == 4).unwrap().unread);
    reduce(&mut app, Action::MarkUnread(4));
    assert!(app.sessions.iter().find(|s| s.id == 4).unwrap().unread);
}

#[test]
fn dropdown_switches_justfile() {
    use lazyjust::app::types::{Justfile, Mode};

    let a = Justfile {
        path: "/a".into(),
        recipes: vec![],
        groups: vec![],
    };
    let b = Justfile {
        path: "/b".into(),
        recipes: vec![],
        groups: vec![],
    };
    let mut app = App::new(
        vec![a, b],
        vec![],
        0.3,
        lazyjust::theme::registry::resolve(lazyjust::theme::DEFAULT_THEME_NAME),
        lazyjust::theme::DEFAULT_THEME_NAME.to_string(),
        lazyjust::ui::icon_style::IconStyle::Round,
        lazyjust::app::types::ListMode::Active,
        std::path::PathBuf::from("."),
    );

    reduce(&mut app, Action::OpenDropdown);
    assert!(matches!(app.mode, Mode::Dropdown { .. }));
    reduce(&mut app, Action::DropdownCursorDown);
    reduce(&mut app, Action::SelectDropdown);
    assert_eq!(app.active_justfile, 1);
    assert_eq!(app.mode, Mode::Normal);
}

#[test]
fn help_open_from_list_records_origin_list_focus() {
    use lazyjust::app::help_section::SectionId;
    use lazyjust::app::types::Focus;
    let mut app = make_app();
    app.focus = Focus::List;
    reduce(&mut app, Action::OpenHelp);
    match app.mode {
        Mode::Help { scroll, origin } => {
            assert_eq!(scroll, 0);
            assert_eq!(origin, SectionId::ListFocus);
        }
        other => panic!("expected Mode::Help, got {other:?}"),
    }
}

#[test]
fn help_open_from_filter_records_origin_filter() {
    use lazyjust::app::help_section::SectionId;
    let mut app = make_app();
    app.mode = Mode::FilterInput;
    reduce(&mut app, Action::OpenHelp);
    match app.mode {
        Mode::Help { origin, .. } => assert_eq!(origin, SectionId::Filter),
        other => panic!("expected Mode::Help, got {other:?}"),
    }
}

#[test]
fn help_scroll_down_monotonic() {
    use lazyjust::app::help_section::SectionId;
    let mut app = make_app();
    app.mode = Mode::Help {
        scroll: 0,
        origin: SectionId::ListFocus,
    };
    reduce(&mut app, Action::HelpScrollDown(1));
    reduce(&mut app, Action::HelpScrollDown(1));
    reduce(&mut app, Action::HelpScrollDown(1));
    match app.mode {
        Mode::Help { scroll, .. } => assert_eq!(scroll, 3),
        _ => panic!("not Help"),
    }
}

#[test]
fn help_scroll_up_floors_zero() {
    use lazyjust::app::help_section::SectionId;
    let mut app = make_app();
    app.mode = Mode::Help {
        scroll: 2,
        origin: SectionId::ListFocus,
    };
    reduce(&mut app, Action::HelpScrollUp(5));
    match app.mode {
        Mode::Help { scroll, .. } => assert_eq!(scroll, 0),
        _ => panic!("not Help"),
    }
}

#[test]
fn help_scroll_home_zeroes() {
    use lazyjust::app::help_section::SectionId;
    let mut app = make_app();
    app.mode = Mode::Help {
        scroll: 42,
        origin: SectionId::ListFocus,
    };
    reduce(&mut app, Action::HelpScrollHome);
    match app.mode {
        Mode::Help { scroll, .. } => assert_eq!(scroll, 0),
        _ => panic!("not Help"),
    }
}

#[test]
fn help_scroll_end_saturates_max() {
    use lazyjust::app::help_section::SectionId;
    let mut app = make_app();
    app.mode = Mode::Help {
        scroll: 0,
        origin: SectionId::ListFocus,
    };
    reduce(&mut app, Action::HelpScrollEnd);
    match app.mode {
        Mode::Help { scroll, .. } => assert_eq!(scroll, u16::MAX),
        _ => panic!("not Help"),
    }
}

#[test]
fn help_close_returns_to_normal() {
    use lazyjust::app::help_section::SectionId;
    let mut app = make_app();
    app.mode = Mode::Help {
        scroll: 5,
        origin: SectionId::ListFocus,
    };
    reduce(&mut app, Action::CloseHelp);
    assert_eq!(app.mode, Mode::Normal);
}

#[test]
fn recipe_at_cursor_returns_recipe_from_owning_justfile_in_all_mode() {
    use lazyjust::app::reducer::reduce;
    use lazyjust::app::types::{Justfile, ListMode, Recipe};
    use lazyjust::app::{Action, App};
    use std::path::PathBuf;

    let r = |n: &str| Recipe {
        name: n.into(),
        module_path: vec![],
        group: None,
        params: vec![],
        doc: None,
        command_preview: String::new(),
        runs: vec![],
        dependencies: vec![],
    };
    let a = Justfile {
        path: PathBuf::from("/root/a/justfile"),
        recipes: vec![r("a1")],
        groups: vec![],
    };
    let b = Justfile {
        path: PathBuf::from("/root/b/justfile"),
        recipes: vec![r("b1"), r("b2")],
        groups: vec![],
    };
    let mut app = App::new(
        vec![a, b],
        vec![],
        0.3,
        lazyjust::theme::registry::resolve(lazyjust::theme::DEFAULT_THEME_NAME),
        lazyjust::theme::DEFAULT_THEME_NAME.to_string(),
        lazyjust::ui::icon_style::IconStyle::Round,
        ListMode::All,
        PathBuf::from("/root"),
    );
    // active_justfile defaults to 0 (justfile A); cursor 1 in All mode is
    // recipe `b1` from justfile B.
    reduce(&mut app, Action::CursorDown);
    assert_eq!(app.list_cursor, 1);
    let recipe = app.recipe_at_cursor().expect("recipe");
    assert_eq!(recipe.name, "b1");
}

#[test]
fn set_list_mode_rebuilds_view_and_resets_cursor_and_filter() {
    use lazyjust::app::reducer::reduce;
    use lazyjust::app::types::{Justfile, ListMode, Recipe};
    use lazyjust::app::{Action, App};
    use std::path::PathBuf;

    let r = |n: &str| Recipe {
        name: n.into(),
        module_path: vec![],
        group: None,
        params: vec![],
        doc: None,
        command_preview: String::new(),
        runs: vec![],
        dependencies: vec![],
    };
    let a = Justfile {
        path: PathBuf::from("/root/a/justfile"),
        recipes: vec![r("a1"), r("a2")],
        groups: vec![],
    };
    let b = Justfile {
        path: PathBuf::from("/root/b/justfile"),
        recipes: vec![r("b1")],
        groups: vec![],
    };
    let mut app = App::new(
        vec![a, b],
        vec![],
        0.3,
        lazyjust::theme::registry::resolve(lazyjust::theme::DEFAULT_THEME_NAME),
        lazyjust::theme::DEFAULT_THEME_NAME.to_string(),
        lazyjust::ui::icon_style::IconStyle::Round,
        ListMode::Active,
        PathBuf::from("/root"),
    );
    reduce(&mut app, Action::CursorDown);
    app.filter = "x".into();
    assert_eq!(app.list_cursor, 1);
    assert_eq!(app.view.recipe_count(), 2); // Active mode

    reduce(&mut app, Action::SetListMode(ListMode::All));

    assert_eq!(app.list_mode, ListMode::All);
    assert_eq!(app.view.recipe_count(), 3);
    assert_eq!(app.list_cursor, 0);
    assert_eq!(app.filter, "");
}

mod list_mode_cursor {
    use lazyjust::app::reducer::reduce;
    use lazyjust::app::types::{Justfile, ListMode, Recipe};
    use lazyjust::app::{Action, App};
    use std::path::PathBuf;

    fn r(n: &str) -> Recipe {
        Recipe {
            name: n.into(),
            module_path: vec![],
            group: None,
            params: vec![],
            doc: None,
            command_preview: String::new(),
            runs: vec![],
            dependencies: vec![],
        }
    }

    fn make_app(mode: ListMode) -> App {
        let a = Justfile {
            path: PathBuf::from("/root/a/justfile"),
            recipes: vec![r("a1"), r("a2")],
            groups: vec![],
        };
        let b = Justfile {
            path: PathBuf::from("/root/b/justfile"),
            recipes: vec![r("b1")],
            groups: vec![],
        };
        App::new(
            vec![a, b],
            vec![],
            0.3,
            lazyjust::theme::registry::resolve(lazyjust::theme::DEFAULT_THEME_NAME),
            lazyjust::theme::DEFAULT_THEME_NAME.to_string(),
            lazyjust::ui::icon_style::IconStyle::Round,
            mode,
            PathBuf::from("/root"),
        )
    }

    #[test]
    fn cursor_in_all_mode_advances_across_justfiles_clamping_at_total() {
        let mut app = make_app(ListMode::All);
        // recipe_count = 3 → cursor should clamp at 2
        reduce(&mut app, Action::CursorDown);
        assert_eq!(app.list_cursor, 1);
        reduce(&mut app, Action::CursorDown);
        assert_eq!(app.list_cursor, 2);
        reduce(&mut app, Action::CursorDown);
        assert_eq!(app.list_cursor, 2); // clamped
    }

    #[test]
    fn cursor_in_active_mode_clamps_to_active_justfile_recipes() {
        let mut app = make_app(ListMode::Active);
        // recipe_count = 2 (only justfile A is active)
        reduce(&mut app, Action::CursorDown);
        assert_eq!(app.list_cursor, 1);
        reduce(&mut app, Action::CursorDown);
        assert_eq!(app.list_cursor, 1); // clamped
    }
}
