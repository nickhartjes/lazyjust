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
                        let panes = crate::ui::layout::compute(size, &app);
                        let pane_rows = panes.right.height.saturating_sub(2);
                        let pane_cols = panes.right.width.saturating_sub(2);
                        for (id, screen) in screens.iter_mut() {
                            screen.set_size(pane_rows, pane_cols);
                            let _ = mgr.resize(*id, pane_rows, pane_cols);
                        }
                        dirty = true;
                        continue;
                    }
                    if let Some(action) = input::handle_event(&evt, &app.mode) {
                        if matches!(action, Action::ConfirmQuit) {
                            for id in mgr.running_ids() {
                                mgr.kill(id);
                            }
                            break;
                        }
                        if matches!(action, Action::RequestQuit)
                            && app.sessions.is_empty()
                        {
                            break;
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
                handle_app_event(&mut app, &mut screens, evt);
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
    evt: AppEvent,
) {
    match evt {
        AppEvent::SessionBytes { id, bytes } => {
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
    let meta = mgr.spawn_recipe(
        id,
        &justfile_path,
        &recipe_name,
        args,
        &cwd,
        24,
        80,
        log_path,
        tx,
    )?;

    screens.insert(id, vt100::Parser::new(24, 80, 0));
    app.sessions.push(meta);
    app.active_session = Some(id);
    app.focus = crate::app::types::Focus::Session;
    let cursor = app.list_cursor;
    if let Some(jf) = app.active_justfile_mut() {
        if let Some(r) = jf.recipes.get_mut(cursor) {
            r.runs.push(id);
        }
    }
    Ok(())
}
