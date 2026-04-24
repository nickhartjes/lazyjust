use lazyrust::app::help_section::SectionId;
use lazyrust::app::types::{
    ConfirmAction, Focus, Justfile, Mode, Param, ParamKind, Recipe, SessionMeta, Status,
};
use lazyrust::app::App;
use lazyrust::theme::registry::resolve;
use lazyrust::ui;
use lazyrust::ui::icon_style::IconStyle;
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use std::path::PathBuf;
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

fn make_justfile_default() -> Justfile {
    let recipes = vec![
        Recipe {
            name: "build".into(),
            module_path: vec![],
            group: Some("ci".into()),
            params: vec![],
            doc: Some("Build release".into()),
            command_preview: "cargo build --release".into(),
            runs: vec![],
            dependencies: Vec::new(),
        },
        Recipe {
            name: "test".into(),
            module_path: vec![],
            group: Some("ci".into()),
            params: vec![],
            doc: None,
            command_preview: "cargo test".into(),
            runs: vec![],
            dependencies: Vec::new(),
        },
    ];
    Justfile {
        path: PathBuf::from("./justfile"),
        recipes,
        groups: vec!["ci".into()],
    }
}

fn make_justfile_ungrouped() -> Justfile {
    let recipes = vec![
        Recipe {
            name: "build".into(),
            module_path: vec![],
            group: None,
            params: vec![],
            doc: Some("Build release".into()),
            command_preview: "cargo build --release".into(),
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
        Recipe {
            name: "fmt".into(),
            module_path: vec![],
            group: None,
            params: vec![],
            doc: Some("Format code".into()),
            command_preview: "cargo fmt".into(),
            runs: vec![],
            dependencies: Vec::new(),
        },
    ];
    Justfile {
        path: PathBuf::from("./justfile"),
        recipes,
        groups: vec![],
    }
}

fn make_justfile_with_deps() -> Justfile {
    let mut jf = make_justfile_default();
    jf.recipes[1].dependencies = vec!["fmt".into(), "lint".into()];
    jf
}

fn make_app(justfile: Justfile, theme_name: &str, icon_style: IconStyle) -> App {
    App::new(
        vec![justfile],
        vec![],
        0.3,
        resolve(theme_name),
        theme_name.to_string(),
        icon_style,
    )
}

fn fixture_default(theme_name: &str, icon_style: IconStyle) -> App {
    make_app(make_justfile_default(), theme_name, icon_style)
}

fn fixture_with_errors(theme_name: &str, icon_style: IconStyle) -> App {
    let mut app = fixture_default(theme_name, icon_style);
    app.startup_errors = vec![
        (PathBuf::from("./justfile"), "syntax error on line 3".into()),
        (
            PathBuf::from("./other.justfile"),
            "unknown recipe: foo".into(),
        ),
    ];
    app
}

fn fixture_with_deps(theme_name: &str, icon_style: IconStyle) -> App {
    make_app(make_justfile_with_deps(), theme_name, icon_style)
}

fn fixture_ungrouped(theme_name: &str, icon_style: IconStyle) -> App {
    make_app(make_justfile_ungrouped(), theme_name, icon_style)
}

/// Stable started_at: 1 hour ago, so elapsed bucket is "1h" regardless of test speed.
fn started_at_stable() -> Instant {
    Instant::now() - Duration::from_secs(3600)
}

fn fixture_session_running(theme_name: &str, icon_style: IconStyle) -> App {
    let mut app = fixture_default(theme_name, icon_style);
    let sid = app.next_session_id();
    app.justfiles[0].recipes[0].runs.push(sid);
    app.sessions.push(SessionMeta {
        id: sid,
        recipe_name: "build".into(),
        command_line: "just build".into(),
        status: Status::Running,
        unread: false,
        started_at: started_at_stable(),
        log_path: PathBuf::from("/tmp/lazyrust-test.log"),
        pid: Some(12345),
    });
    app.active_session = Some(sid);
    app
}

fn fixture_session_exited_fail(theme_name: &str, icon_style: IconStyle) -> App {
    let mut app = fixture_default(theme_name, icon_style);
    let sid = app.next_session_id();
    app.justfiles[0].recipes[0].runs.push(sid);
    app.sessions.push(SessionMeta {
        id: sid,
        recipe_name: "build".into(),
        command_line: "just build".into(),
        status: Status::Exited { code: 1 },
        unread: false,
        started_at: started_at_stable(),
        log_path: PathBuf::from("/tmp/lazyrust-test.log"),
        pid: Some(12345),
    });
    app.active_session = Some(sid);
    app
}

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn render_to_string(app: &App, width: u16, height: u16) -> String {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    let screens = ui::SessionScreens::new();
    terminal.draw(|f| ui::render(f, app, &screens)).unwrap();
    let buf = terminal.backend().buffer().clone();
    buffer_to_string(&buf)
}

fn buffer_to_string(buf: &ratatui::buffer::Buffer) -> String {
    let area = buf.area;
    let mut symbols = String::new();
    let mut styled = String::new();
    for y in 0..area.height {
        for x in 0..area.width {
            let cell = buf.get(x, y);
            symbols.push_str(cell.symbol());
            let sym = cell.symbol();
            let is_ws = sym.chars().all(char::is_whitespace);
            if !is_ws {
                styled.push_str(&format!(
                    "({x},{y}) {:?} fg={:?} bg={:?} mod={:?}\n",
                    sym, cell.fg, cell.bg, cell.modifier
                ));
            }
        }
        symbols.push('\n');
    }
    format!("{symbols}\n--- styled cells ---\n{styled}")
}

// ---------------------------------------------------------------------------
// 1. initial_tokyo_night
// ---------------------------------------------------------------------------
#[test]
fn initial_tokyo_night() {
    let app = fixture_default("tokyo-night", IconStyle::Round);
    insta::assert_snapshot!(render_to_string(&app, 80, 24));
}

// ---------------------------------------------------------------------------
// 2. initial_mono_amber
// ---------------------------------------------------------------------------
#[test]
fn initial_mono_amber() {
    let app = fixture_default("mono-amber", IconStyle::Round);
    insta::assert_snapshot!(render_to_string(&app, 80, 24));
}

// ---------------------------------------------------------------------------
// 3. with_errors_tokyo_night
// ---------------------------------------------------------------------------
#[test]
fn with_errors_tokyo_night() {
    let app = fixture_with_errors("tokyo-night", IconStyle::Round);
    insta::assert_snapshot!(render_to_string(&app, 80, 24));
}

// ---------------------------------------------------------------------------
// 4. with_errors_mono_amber
// ---------------------------------------------------------------------------
#[test]
fn with_errors_mono_amber() {
    let app = fixture_with_errors("mono-amber", IconStyle::Round);
    insta::assert_snapshot!(render_to_string(&app, 80, 24));
}

// ---------------------------------------------------------------------------
// 5. list_with_deps_tokyo_night
// ---------------------------------------------------------------------------
#[test]
fn list_with_deps_tokyo_night() {
    let app = fixture_with_deps("tokyo-night", IconStyle::Round);
    insta::assert_snapshot!(render_to_string(&app, 80, 24));
}

// ---------------------------------------------------------------------------
// 6. list_with_deps_mono_amber
// ---------------------------------------------------------------------------
#[test]
fn list_with_deps_mono_amber() {
    let app = fixture_with_deps("mono-amber", IconStyle::Round);
    insta::assert_snapshot!(render_to_string(&app, 80, 24));
}

// ---------------------------------------------------------------------------
// 7. ungrouped_tokyo_night
// ---------------------------------------------------------------------------
#[test]
fn ungrouped_tokyo_night() {
    let app = fixture_ungrouped("tokyo-night", IconStyle::Round);
    insta::assert_snapshot!(render_to_string(&app, 80, 24));
}

// ---------------------------------------------------------------------------
// 8. session_running_tokyo_night
// ---------------------------------------------------------------------------
#[test]
fn session_running_tokyo_night() {
    let app = fixture_session_running("tokyo-night", IconStyle::Round);
    insta::assert_snapshot!(render_to_string(&app, 80, 24));
}

// ---------------------------------------------------------------------------
// 9. session_running_mono_amber
// ---------------------------------------------------------------------------
#[test]
fn session_running_mono_amber() {
    let app = fixture_session_running("mono-amber", IconStyle::Round);
    insta::assert_snapshot!(render_to_string(&app, 80, 24));
}

// ---------------------------------------------------------------------------
// 10. session_exited_fail_tokyo_night
// ---------------------------------------------------------------------------
#[test]
fn session_exited_fail_tokyo_night() {
    let app = fixture_session_exited_fail("tokyo-night", IconStyle::Round);
    insta::assert_snapshot!(render_to_string(&app, 80, 24));
}

// ---------------------------------------------------------------------------
// 11. filter_mode_tokyo_night
// ---------------------------------------------------------------------------
#[test]
fn filter_mode_tokyo_night() {
    let mut app = fixture_default("tokyo-night", IconStyle::Round);
    app.mode = Mode::FilterInput;
    app.filter = "bu".into();
    insta::assert_snapshot!(render_to_string(&app, 80, 24));
}

// ---------------------------------------------------------------------------
// 12. help_modal_tokyo_night
// ---------------------------------------------------------------------------
#[test]
fn help_modal_tokyo_night() {
    let mut app = fixture_default("tokyo-night", IconStyle::Round);
    app.mode = Mode::Help {
        scroll: 0,
        origin: SectionId::ListFocus,
    };
    insta::assert_snapshot!(render_to_string(&app, 80, 24));
}

// ---------------------------------------------------------------------------
// 13. errors_modal_tokyo_night
// ---------------------------------------------------------------------------
#[test]
fn errors_modal_tokyo_night() {
    let mut app = fixture_with_errors("tokyo-night", IconStyle::Round);
    app.mode = Mode::ErrorsList;
    insta::assert_snapshot!(render_to_string(&app, 80, 24));
}

// ---------------------------------------------------------------------------
// 14. theme_picker_tokyo_night
// ---------------------------------------------------------------------------
#[test]
fn theme_picker_tokyo_night() {
    let mut app = fixture_default("tokyo-night", IconStyle::Round);
    app.mode = Mode::ThemePicker {
        original_name: "tokyo-night".into(),
        highlighted: 0,
        names: vec!["tokyo-night".into(), "mono-amber".into()],
    };
    insta::assert_snapshot!(render_to_string(&app, 80, 24));
}

// ---------------------------------------------------------------------------
// 15. param_input_tokyo_night
// ---------------------------------------------------------------------------
#[test]
fn param_input_tokyo_night() {
    let mut jf = make_justfile_default();
    jf.recipes[0].params = vec![Param {
        name: "target".into(),
        default: Some("x86_64".into()),
        kind: ParamKind::Positional,
    }];
    let mut app = make_app(jf, "tokyo-night", IconStyle::Round);
    app.mode = Mode::ParamInput {
        recipe_idx: 0,
        values: vec!["x86_64".into()],
        cursor: 0,
    };
    insta::assert_snapshot!(render_to_string(&app, 80, 24));
}

// ---------------------------------------------------------------------------
// 16. confirm_modal_tokyo_night
// ---------------------------------------------------------------------------
#[test]
fn confirm_modal_tokyo_night() {
    let mut app = fixture_session_running("tokyo-night", IconStyle::Round);
    let sid = app.active_session.unwrap();
    app.mode = Mode::Confirm {
        prompt: "kill session?".into(),
        on_accept: ConfirmAction::KillSession(sid),
    };
    app.focus = Focus::Modal;
    insta::assert_snapshot!(render_to_string(&app, 80, 24));
}

// ---------------------------------------------------------------------------
// 17. list_ascii_tokyo_night
// ---------------------------------------------------------------------------
#[test]
fn list_ascii_tokyo_night() {
    let app = fixture_default("tokyo-night", IconStyle::Ascii);
    insta::assert_snapshot!(render_to_string(&app, 80, 24));
}

// ---------------------------------------------------------------------------
// 18. list_none_tokyo_night
// ---------------------------------------------------------------------------
#[test]
fn list_none_tokyo_night() {
    let app = fixture_default("tokyo-night", IconStyle::None);
    insta::assert_snapshot!(render_to_string(&app, 80, 24));
}

// ---------------------------------------------------------------------------
// 19. small_40x10_tokyo_night
// ---------------------------------------------------------------------------
#[test]
fn small_40x10_tokyo_night() {
    let app = fixture_default("tokyo-night", IconStyle::Round);
    insta::assert_snapshot!(render_to_string(&app, 40, 10));
}

// ---------------------------------------------------------------------------
// 20. wide_160x50_tokyo_night
// ---------------------------------------------------------------------------
#[test]
fn wide_160x50_tokyo_night() {
    let app = fixture_default("tokyo-night", IconStyle::Round);
    insta::assert_snapshot!(render_to_string(&app, 160, 50));
}
