use super::action::Action;
use super::state::App;
use super::types::{ConfirmAction, Mode};

const SPLIT_MIN: f32 = 0.15;
const SPLIT_MAX: f32 = 0.70;
const SPLIT_STEP: f32 = 0.05;
const SPLIT_DEFAULT: f32 = 0.30;

#[allow(clippy::collapsible_if, clippy::collapsible_match)]
pub fn reduce(app: &mut App, action: Action) {
    match action {
        Action::NoOp => {}

        Action::CursorDown => {
            if let Some(jf) = app.active_justfile() {
                let max = jf.recipes.len().saturating_sub(1);
                if app.list_cursor < max {
                    app.list_cursor += 1;
                }
            }
        }
        Action::CursorUp => {
            app.list_cursor = app.list_cursor.saturating_sub(1);
        }

        Action::EnterFilter => app.mode = Mode::FilterInput,
        Action::FilterChar(c) => {
            if app.mode == Mode::FilterInput {
                app.filter.push(c);
            }
        }
        Action::FilterBackspace => {
            if app.mode == Mode::FilterInput {
                app.filter.pop();
            }
        }
        Action::CommitFilter => {
            if app.mode == Mode::FilterInput {
                app.mode = Mode::Normal;
            }
        }
        Action::CancelFilter => {
            app.filter.clear();
            app.mode = Mode::Normal;
        }

        Action::GrowLeftPane => {
            app.split_ratio = (app.split_ratio + SPLIT_STEP).min(SPLIT_MAX);
        }
        Action::ShrinkLeftPane => {
            app.split_ratio = (app.split_ratio - SPLIT_STEP).max(SPLIT_MIN);
        }
        Action::ResetSplit => app.split_ratio = SPLIT_DEFAULT,

        Action::RequestQuit => {
            if app.sessions.iter().any(|s| {
                matches!(
                    s.status,
                    crate::app::types::Status::Running
                        | crate::app::types::Status::ShellAfterExit { .. }
                )
            }) {
                app.mode = Mode::Confirm {
                    prompt: "Sessions running. Quit & kill all?".into(),
                    on_accept: ConfirmAction::QuitKillAll,
                };
            } else {
                // caller handles actual quit after reducer by checking Quit elsewhere.
                app.mode = Mode::Normal;
            }
        }
        Action::CancelConfirm => app.mode = Mode::Normal,
        Action::Quit | Action::ConfirmQuit => {
            // handled by event loop — reducer leaves it as signal
        }

        Action::OpenHelp => app.mode = Mode::Help,
        Action::CloseHelp => app.mode = Mode::Normal,

        // Remaining actions handled in later tasks.
        _ => {}
    }
}
