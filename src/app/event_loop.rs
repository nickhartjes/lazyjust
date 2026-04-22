use super::action::{Action, AppEvent};
use super::reducer::reduce;
use super::state::App;
use crate::config::Config;
use crate::input;
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

    // event_tx is held for later tasks (PTY readers). Silence unused warning.
    let _ = &event_tx;

    loop {
        if dirty && last_render.elapsed() >= cfg.render_throttle {
            terminal.draw(|f| ui::render(f, &app))?;
            last_render = Instant::now();
            dirty = false;
        }

        tokio::select! {
            Some(ct) = crossterm_events.next() => {
                if let Ok(evt) = ct {
                    if let crossterm::event::Event::Resize(_, _) = evt {
                        dirty = true;
                        continue;
                    }
                    if let Some(action) = input::handle_event(&evt, &app.mode) {
                        if matches!(action, Action::ConfirmQuit) {
                            break;
                        }
                        if matches!(action, Action::RequestQuit)
                            && app.sessions.is_empty()
                        {
                            break;
                        }
                        reduce(&mut app, action);
                        dirty = true;
                    }
                }
            }
            Some(evt) = event_rx.recv() => {
                handle_app_event(&mut app, evt);
                dirty = true;
            }
            _ = tick.tick() => {
                // session try_wait polling added later
                dirty |= app.status_message.take().is_some();
            }
        }
    }

    Ok(())
}

fn handle_app_event(_app: &mut App, _evt: AppEvent) {
    // session byte / exit handling wired in later tasks
}
