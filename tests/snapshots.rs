use lazyjust::app::types::{Focus, Justfile, Recipe};
use lazyjust::app::App;
use lazyjust::ui;
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use std::path::PathBuf;

fn fixture_app() -> App {
    let recipes = vec![
        Recipe {
            name: "build".into(),
            module_path: vec![],
            group: Some("ci".into()),
            params: vec![],
            doc: Some("Build release".into()),
            command_preview: "cargo build --release".into(),
            runs: vec![],
        },
        Recipe {
            name: "test".into(),
            module_path: vec![],
            group: Some("ci".into()),
            params: vec![],
            doc: None,
            command_preview: "cargo test".into(),
            runs: vec![],
        },
    ];
    let jf = Justfile {
        path: PathBuf::from("./justfile"),
        recipes,
        groups: vec!["ci".into()],
    };
    App::new(vec![jf], vec![], 0.3)
}

#[test]
fn initial_render_snapshot() {
    let backend = TestBackend::new(80, 20);
    let mut terminal = Terminal::new(backend).unwrap();
    let app = fixture_app();
    let screens = ui::SessionScreens::new();
    terminal.draw(|f| ui::render(f, &app, &screens)).unwrap();
    let buf = terminal.backend().buffer().clone();
    insta::assert_snapshot!(buffer_to_string(&buf));
}

#[test]
fn session_focus_render_snapshot() {
    let backend = TestBackend::new(80, 20);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = fixture_app();
    app.focus = Focus::Session;
    let screens = ui::SessionScreens::new();
    terminal.draw(|f| ui::render(f, &app, &screens)).unwrap();
    let buf = terminal.backend().buffer().clone();
    insta::assert_snapshot!(buffer_to_string(&buf));
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
