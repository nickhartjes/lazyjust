use super::action::{Action, AppEvent};
use super::reducer::reduce;
use super::state::App;
use crate::config::Config;
use crate::input;
use crate::session::manager::SessionManager;
use crate::ui;
use anyhow::Result;
use crossterm::event::EventStream;
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use futures::StreamExt;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::collections::HashMap;
use std::io::stdout;
use std::time::Instant;
use tokio::sync::mpsc;

/// vt100 scrollback capacity per session. User scrolls with PgUp/PgDn/Home/End
/// when session pane is focused.
const SCROLLBACK_LINES: usize = 2000;

pub struct TerminalGuard;

impl TerminalGuard {
    pub fn enter() -> Result<Self> {
        enable_raw_mode()?;
        execute!(stdout(), EnterAlternateScreen)?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), LeaveAlternateScreen);
    }
}

pub fn install_panic_hook() {
    let default = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), LeaveAlternateScreen);
        default(info);
    }));
}

pub async fn run(mut app: App, cfg: Config) -> Result<()> {
    install_panic_hook();
    let _guard = TerminalGuard::enter()?;
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    let (event_tx, mut event_rx) = mpsc::channel::<AppEvent>(256);
    let mut crossterm_events = EventStream::new();
    let mut tick = tokio::time::interval(cfg.tick_interval);
    let mut last_render = Instant::now();
    let mut dirty = true;
    let mut mgr = SessionManager::default();
    let mut screens: HashMap<crate::app::types::SessionId, vt100::Parser> = HashMap::new();

    loop {
        if dirty && last_render.elapsed() >= cfg.render_throttle {
            terminal.draw(|f| ui::render(f, &app, &screens))?;
            last_render = Instant::now();
            dirty = false;
        }

        tokio::select! {
            Some(ct) = crossterm_events.next() => {
                if let Ok(evt) = ct {
                    if let crossterm::event::Event::Resize(_, _) = evt {
                        let size = terminal.size()?;
                        let panes = crate::ui::layout::compute(size.into(), &app);
                        // right pane body = pane.height - borders (2) - session header (1) - spacer (1)
                        // right pane cols  = pane.width  - borders (2) - scroll thumb (1)
                        let pane_rows = panes.right.height.saturating_sub(4);
                        let pane_cols = panes.right.width.saturating_sub(3);
                        for (id, screen) in screens.iter_mut() {
                            screen.screen_mut().set_size(pane_rows, pane_cols);
                            let _ = mgr.resize(*id, pane_rows, pane_cols);
                        }
                        dirty = true;
                        continue;
                    }
                    if let crossterm::event::Event::Key(key) = evt {
                        if key.code == crossterm::event::KeyCode::F(1)
                            && !matches!(app.mode, crate::app::types::Mode::Help { .. })
                        {
                            crate::app::reducer::reduce(&mut app, Action::OpenHelp);
                            dirty = true;
                            continue;
                        }
                        if app.focus == crate::app::types::Focus::Session
                            && app.mode == crate::app::types::Mode::Normal
                        {
                            let is_exit_key = matches!(
                                key.code,
                                crossterm::event::KeyCode::F(12)
                            ) || (key.code
                                == crossterm::event::KeyCode::Char('g')
                                && key.modifiers.contains(
                                    crossterm::event::KeyModifiers::CONTROL,
                                ));
                            if is_exit_key {
                                crate::app::reducer::reduce(&mut app, Action::FocusList);
                                dirty = true;
                                continue;
                            }
                            // Scrollback controls — consumed locally, not forwarded to PTY.
                            if let Some(sid) = app.active_session {
                                if let Some(screen) = screens.get_mut(&sid) {
                                    let cur = screen.screen().scrollback();
                                    match key.code {
                                        crossterm::event::KeyCode::PageUp => {
                                            screen.screen_mut().set_scrollback(cur.saturating_add(10));
                                            dirty = true;
                                            continue;
                                        }
                                        crossterm::event::KeyCode::PageDown => {
                                            screen.screen_mut().set_scrollback(cur.saturating_sub(10));
                                            dirty = true;
                                            continue;
                                        }
                                        crossterm::event::KeyCode::Home => {
                                            // Top of scrollback = large offset, vt100 clamps to actual size.
                                            screen.screen_mut().set_scrollback(usize::MAX / 2);
                                            dirty = true;
                                            continue;
                                        }
                                        crossterm::event::KeyCode::End => {
                                            screen.screen_mut().set_scrollback(0);
                                            dirty = true;
                                            continue;
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            let bytes = encode_key(key);
                            if !bytes.is_empty() {
                                if let Some(sid) = app.active_session {
                                    let _ = mgr.write(sid, &bytes);
                                }
                                continue;
                            }
                        }
                    }
                    if let Some(action) = input::handle_event(&evt, &app.mode) {
                        if let Action::ConfirmQuit = action {
                            let accept = if let crate::app::types::Mode::Confirm { on_accept, .. } =
                                app.mode.clone()
                            {
                                Some(on_accept)
                            } else {
                                None
                            };
                            app.mode = crate::app::types::Mode::Normal;
                            match accept {
                                Some(crate::app::types::ConfirmAction::QuitKillAll) => {
                                    for id in mgr.running_ids() {
                                        mgr.kill(id);
                                    }
                                    screens.clear();
                                    break;
                                }
                                Some(crate::app::types::ConfirmAction::KillSession(id)) => {
                                    mgr.kill(id);
                                    reduce(&mut app, Action::KillSession(id));
                                }
                                Some(crate::app::types::ConfirmAction::CloseSession(id)) => {
                                    mgr.kill(id);
                                    reduce(&mut app, Action::CloseSession(id));
                                    screens.remove(&id);
                                }
                                None => {}
                            }
                            dirty = true;
                            continue;
                        }
                        if matches!(action, Action::RequestQuit)
                            && app.sessions.is_empty()
                        {
                            break;
                        }
                        if let Action::ParamCommit = action {
                            if matches!(app.mode, crate::app::types::Mode::ParamInput { .. }) {
                                if let crate::app::types::Mode::ParamInput {
                                    recipe_idx,
                                    values,
                                    ..
                                } = std::mem::replace(
                                    &mut app.mode,
                                    crate::app::types::Mode::Normal,
                                ) {
                                    // Pin the cursor to the recipe the modal was opened for so
                                    // do_spawn's `app.list_cursor` lookup can't drift.
                                    app.list_cursor = recipe_idx;
                                    do_spawn(
                                        &mut app,
                                        &mut mgr,
                                        &mut screens,
                                        &cfg,
                                        &values,
                                        event_tx.clone(),
                                    )?;
                                    dirty = true;
                                    continue;
                                }
                            }
                        }
                        if let Action::RunHighlighted { force_new } = action {
                            spawn_highlighted(&mut app, &mut mgr, &mut screens, &cfg, force_new, event_tx.clone())?;
                        } else {
                            reduce(&mut app, action);
                        }
                        dirty = true;
                    }
                }
            }
            Some(evt) = event_rx.recv() => {
                handle_app_event(&mut app, &mut screens, &mut mgr, evt);
                dirty = true;
            }
            _ = tick.tick() => {
                for id in mgr.running_ids() {
                    if let Some(code) = mgr.try_wait(id) {
                        reduce(&mut app, Action::SessionExited { id, code });
                    }
                }
                dirty |= app.status_message.take().is_some();
            }
        }
    }

    Ok(())
}

fn handle_app_event(
    app: &mut App,
    screens: &mut HashMap<crate::app::types::SessionId, vt100::Parser>,
    mgr: &mut SessionManager,
    evt: AppEvent,
) {
    match evt {
        AppEvent::SessionBytes { id, bytes } => {
            mgr.write_log(id, &bytes);
            if let Some(screen) = screens.get_mut(&id) {
                screen.process(&bytes);
            }
            if app.active_session != Some(id) {
                if let Some(s) = app.session_mut(id) {
                    s.unread = true;
                }
            }
        }
        AppEvent::SessionExited { id, code } => {
            reduce(app, Action::SessionExited { id, code });
        }
        AppEvent::RecipeExited { id, code } => {
            reduce(app, Action::RecipeExited { id, code });
        }
        AppEvent::Crossterm(_) | AppEvent::Tick => {}
    }
}

pub fn spawn_highlighted(
    app: &mut App,
    mgr: &mut SessionManager,
    screens: &mut HashMap<crate::app::types::SessionId, vt100::Parser>,
    cfg: &Config,
    force_new: bool,
    tx: mpsc::Sender<AppEvent>,
) -> Result<()> {
    use crate::app::types::{Mode, Status};

    if !force_new {
        if let Some(r) = app.recipe_at_cursor() {
            let running = r
                .runs
                .iter()
                .rev()
                .find(|id| {
                    app.sessions.iter().any(|s| {
                        s.id == **id
                            && matches!(s.status, Status::Running | Status::ShellAfterExit { .. })
                    })
                })
                .copied();
            if let Some(sid) = running {
                app.active_session = Some(sid);
                app.focus = crate::app::types::Focus::Session;
                if let Some(s) = app.session_mut(sid) {
                    s.unread = false;
                }
                return Ok(());
            }
        }
    }

    if let Some(r) = app.recipe_at_cursor() {
        if !r.params.is_empty() {
            let values = r
                .params
                .iter()
                .map(|p| p.default.clone().unwrap_or_default())
                .collect::<Vec<_>>();
            app.mode = Mode::ParamInput {
                recipe_idx: app.list_cursor,
                values,
                cursor: 0,
            };
            return Ok(());
        }
    }

    do_spawn(app, mgr, screens, cfg, &[], tx)
}

pub fn do_spawn(
    app: &mut App,
    mgr: &mut SessionManager,
    screens: &mut HashMap<crate::app::types::SessionId, vt100::Parser>,
    cfg: &Config,
    args: &[String],
    tx: mpsc::Sender<AppEvent>,
) -> Result<()> {
    let recipe_name;
    let justfile_path;
    let cwd;
    {
        let jf = match app.active_justfile() {
            Some(j) => j,
            None => return Ok(()),
        };
        let r = match jf.recipes.get(app.list_cursor) {
            Some(r) => r,
            None => return Ok(()),
        };
        recipe_name = r.name.clone();
        justfile_path = jf.path.clone();
        cwd = jf
            .path
            .parent()
            .unwrap_or(std::path::Path::new("."))
            .to_path_buf();
    }

    let id = app.next_session_id();
    let log_path = crate::logging::session_log_path(cfg, id, &recipe_name)?;

    let (rows, cols) = match crossterm::terminal::size() {
        Ok((w, h)) => {
            let size = ratatui::layout::Rect {
                x: 0,
                y: 0,
                width: w,
                height: h,
            };
            let panes = crate::ui::layout::compute(size, app);
            (
                panes.right.height.saturating_sub(4).max(1),
                panes.right.width.saturating_sub(3).max(1),
            )
        }
        Err(_) => (24, 80),
    };

    let meta = mgr.spawn_recipe(
        id,
        &justfile_path,
        &recipe_name,
        args,
        &cwd,
        rows,
        cols,
        log_path,
        tx,
        cfg.session_log_size_cap,
    )?;

    screens.insert(id, vt100::Parser::new(rows, cols, SCROLLBACK_LINES));
    app.sessions.push(meta);
    app.active_session = Some(id);
    let cursor = app.list_cursor;
    if let Some(jf) = app.active_justfile_mut() {
        if let Some(r) = jf.recipes.get_mut(cursor) {
            r.runs.push(id);
        }
    }
    Ok(())
}

fn encode_key(key: crossterm::event::KeyEvent) -> Vec<u8> {
    use crossterm::event::{KeyCode, KeyModifiers};
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    match key.code {
        KeyCode::Char(c) => {
            if ctrl {
                let byte = match c {
                    '@' => 0x00,
                    'a'..='z' => (c as u8) - b'a' + 1,
                    '[' => 0x1b,
                    '\\' => 0x1c,
                    ']' => 0x1d,
                    '^' => 0x1e,
                    '_' => 0x1f,
                    _ => return Vec::new(),
                };
                vec![byte]
            } else {
                c.to_string().into_bytes()
            }
        }
        KeyCode::Enter => b"\r".to_vec(),
        KeyCode::Tab => b"\t".to_vec(),
        KeyCode::Backspace => b"\x7f".to_vec(),
        KeyCode::Esc => b"\x1b".to_vec(),
        KeyCode::Up => b"\x1b[A".to_vec(),
        KeyCode::Down => b"\x1b[B".to_vec(),
        KeyCode::Right => b"\x1b[C".to_vec(),
        KeyCode::Left => b"\x1b[D".to_vec(),
        KeyCode::Home => b"\x1b[H".to_vec(),
        KeyCode::End => b"\x1b[F".to_vec(),
        _ => Vec::new(),
    }
}
