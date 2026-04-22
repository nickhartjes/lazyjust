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

        Action::OpenDropdown => {
            app.mode = Mode::Dropdown {
                filter: String::new(),
                cursor: app.active_justfile,
            };
        }
        Action::DropdownChar(c) => {
            if let Mode::Dropdown { filter, cursor } = &mut app.mode {
                filter.push(c);
                *cursor = 0;
            }
        }
        Action::DropdownBackspace => {
            if let Mode::Dropdown { filter, .. } = &mut app.mode {
                filter.pop();
            }
        }
        Action::DropdownCursorDown => {
            let max = app.justfiles.len().saturating_sub(1);
            if let Mode::Dropdown { cursor, .. } = &mut app.mode {
                if *cursor < max {
                    *cursor += 1;
                }
            }
        }
        Action::DropdownCursorUp => {
            if let Mode::Dropdown { cursor, .. } = &mut app.mode {
                *cursor = cursor.saturating_sub(1);
            }
        }
        Action::SelectDropdown => {
            if let Mode::Dropdown { cursor, filter } = app.mode.clone() {
                let filtered = filtered_justfile_indices(app, &filter);
                if let Some(&chosen) = filtered.get(cursor) {
                    app.active_justfile = chosen;
                    app.list_cursor = 0;
                    app.filter.clear();
                }
                app.mode = Mode::Normal;
            }
        }
        Action::CancelDropdown => app.mode = Mode::Normal,

        Action::SessionExited { id, code } => {
            if let Some(s) = app.session_mut(id) {
                if matches!(s.status, crate::app::types::Status::Running) {
                    s.status = crate::app::types::Status::Exited { code };
                    s.unread = true;
                } else if let crate::app::types::Status::ShellAfterExit { .. } = s.status {
                    s.status = crate::app::types::Status::Exited { code };
                }
            }
        }
        Action::RecipeExited { id, code } => {
            let is_active = Some(id) == app.active_session;
            if let Some(s) = app.session_mut(id) {
                s.status = crate::app::types::Status::ShellAfterExit { code };
                if !is_active {
                    s.unread = true;
                }
            }
        }
        Action::MarkUnread(id) => {
            if let Some(s) = app.session_mut(id) {
                s.unread = true;
            }
        }
        Action::MarkRead(id) => {
            if let Some(s) = app.session_mut(id) {
                s.unread = false;
            }
        }

        Action::CycleFocus => {
            app.focus = match app.focus {
                crate::app::types::Focus::List => crate::app::types::Focus::Session,
                crate::app::types::Focus::Session => crate::app::types::Focus::List,
                other => other,
            };
        }
        Action::FocusList => app.focus = crate::app::types::Focus::List,
        Action::FocusSession => app.focus = crate::app::types::Focus::Session,
        Action::FocusNextSession => {
            let ids: Vec<_> = app.sessions.iter().map(|s| s.id).collect();
            if let Some(cur) = app.active_session {
                if let Some(i) = ids.iter().position(|id| *id == cur) {
                    if let Some(next) = ids.get(i + 1) {
                        app.active_session = Some(*next);
                        if let Some(s) = app.session_mut(*next) {
                            s.unread = false;
                        }
                    }
                }
            } else if let Some(first) = ids.first() {
                app.active_session = Some(*first);
            }
        }
        Action::FocusPrevSession => {
            let ids: Vec<_> = app.sessions.iter().map(|s| s.id).collect();
            if let Some(cur) = app.active_session {
                if let Some(i) = ids.iter().position(|id| *id == cur) {
                    if i > 0 {
                        let prev = ids[i - 1];
                        app.active_session = Some(prev);
                        if let Some(s) = app.session_mut(prev) {
                            s.unread = false;
                        }
                    }
                }
            }
        }

        // Remaining actions handled in later tasks.
        _ => {}
    }
}

pub fn filtered_justfile_indices(app: &App, filter: &str) -> Vec<usize> {
    let paths: Vec<String> = app
        .justfiles
        .iter()
        .map(|j| j.path.display().to_string())
        .collect();
    let refs: Vec<&str> = paths.iter().map(|s| s.as_str()).collect();
    crate::app::filter::fuzzy_match(&refs, filter)
        .into_iter()
        .map(|(i, _)| i)
        .collect()
}
