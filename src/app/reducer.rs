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

        Action::OpenHelp => {
            let origin = crate::app::help_section::active_section(app);
            app.mode = Mode::Help { scroll: 0, origin };
        }
        Action::CloseHelp => app.mode = Mode::Normal,
        Action::HelpScrollDown(n) => {
            if let Mode::Help { scroll, .. } = &mut app.mode {
                *scroll = scroll.saturating_add(n);
            }
        }
        Action::HelpScrollUp(n) => {
            if let Mode::Help { scroll, .. } = &mut app.mode {
                *scroll = scroll.saturating_sub(n);
            }
        }
        Action::HelpScrollHome => {
            if let Mode::Help { scroll, .. } = &mut app.mode {
                *scroll = 0;
            }
        }
        Action::HelpScrollEnd => {
            if let Mode::Help { scroll, .. } = &mut app.mode {
                *scroll = u16::MAX;
            }
        }

        Action::OpenErrors => app.mode = Mode::ErrorsList,
        Action::CloseErrors => app.mode = Mode::Normal,

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

        Action::ParamChar(c) => {
            if let Mode::ParamInput { values, cursor, .. } = &mut app.mode {
                if let Some(v) = values.get_mut(*cursor) {
                    v.push(c);
                }
            }
        }
        Action::ParamBackspace => {
            if let Mode::ParamInput { values, cursor, .. } = &mut app.mode {
                if let Some(v) = values.get_mut(*cursor) {
                    v.pop();
                }
            }
        }
        Action::ParamNext => {
            if let Mode::ParamInput { values, cursor, .. } = &mut app.mode {
                if *cursor + 1 < values.len() {
                    *cursor += 1;
                }
            }
        }
        Action::CancelParam => app.mode = Mode::Normal,
        // ParamCommit handled by event_loop (needs side effects)
        Action::CycleRecipeHistoryPrev => cycle_history(app, -1),
        Action::CycleRecipeHistoryNext => cycle_history(app, 1),

        Action::RequestKillSession => {
            if let Some(id) = app.active_session {
                app.mode = Mode::Confirm {
                    prompt: format!("Kill session {id}?"),
                    on_accept: crate::app::types::ConfirmAction::KillSession(id),
                };
            }
        }
        Action::RequestCloseSession => {
            if let Some(id) = app.active_session {
                app.mode = Mode::Confirm {
                    prompt: format!("Close session {id}?"),
                    on_accept: crate::app::types::ConfirmAction::CloseSession(id),
                };
            }
        }
        Action::KillSession(id) => {
            if let Some(s) = app.session_mut(id) {
                s.status = crate::app::types::Status::Exited { code: 130 };
            }
            // actual PTY kill done in event loop
        }
        Action::CloseSession(id) => {
            app.sessions.retain(|s| s.id != id);
            if app.active_session == Some(id) {
                app.active_session = None;
            }
            for jf in &mut app.justfiles {
                for r in &mut jf.recipes {
                    r.runs.retain(|rid| *rid != id);
                }
            }
        }
        Action::CopyLogPath => {
            if let Some(id) = app.active_session {
                if let Some(s) = app.session(id) {
                    if let Ok(mut cb) = arboard::Clipboard::new() {
                        let _ = cb.set_text(s.log_path.display().to_string());
                        app.status_message = Some(format!("copied {}", s.log_path.display()));
                    }
                }
            }
        }

        Action::OpenThemePicker => {
            let names = crate::theme::registry::list();
            let original_name = app.theme_name.clone();
            let highlighted = names.iter().position(|n| *n == original_name).unwrap_or(0);
            app.mode = Mode::ThemePicker {
                original_name,
                highlighted,
                names,
            };
        }
        Action::PickerMove(delta) => {
            if let Mode::ThemePicker {
                highlighted,
                names,
                ..
            } = &mut app.mode
            {
                let len = names.len() as isize;
                if len > 0 {
                    let mut idx = *highlighted as isize + delta;
                    idx = idx.rem_euclid(len);
                    *highlighted = idx as usize;
                    let stem = names[*highlighted].clone();
                    app.theme = crate::theme::registry::resolve(&stem);
                    app.theme_name = stem;
                }
            }
        }
        Action::PickerConfirm => {
            if let Mode::ThemePicker { .. } = app.mode {
                // T16 wires the toml_edit writer. For now, just close the modal —
                // `app.theme_name` is already updated by PickerMove, so next startup
                // would read the old value. We accept that gap; T16 closes it.
                app.mode = Mode::Normal;
            }
        }
        Action::PickerCancel => {
            // Separate scope so we can re-borrow app immutably after restoring.
            let original = if let Mode::ThemePicker { original_name, .. } = &app.mode {
                Some(original_name.clone())
            } else {
                None
            };
            if let Some(original) = original {
                app.theme = crate::theme::registry::resolve(&original);
                app.theme_name = original;
                app.mode = Mode::Normal;
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

#[cfg(test)]
mod theme_picker_tests {
    use super::*;
    use crate::app::action::Action;
    use crate::app::types::Mode;

    fn test_app() -> App {
        App::new(
            vec![],
            vec![],
            0.3,
            crate::theme::registry::resolve(crate::theme::DEFAULT_THEME_NAME),
            crate::theme::DEFAULT_THEME_NAME.to_string(),
        )
    }

    #[test]
    fn open_picker_enters_mode_with_current_theme_highlighted() {
        let mut app = test_app();
        reduce(&mut app, Action::OpenThemePicker);
        match &app.mode {
            Mode::ThemePicker {
                original_name,
                highlighted,
                names,
            } => {
                assert_eq!(original_name, crate::theme::DEFAULT_THEME_NAME);
                assert_eq!(names[*highlighted], crate::theme::DEFAULT_THEME_NAME);
            }
            _ => panic!("expected ThemePicker mode"),
        }
    }

    #[test]
    fn picker_move_wraps_around() {
        let mut app = test_app();
        reduce(&mut app, Action::OpenThemePicker);
        // Moving up from index 0 wraps to last element.
        reduce(&mut app, Action::PickerMove(-1));
        let last_name = match &app.mode {
            Mode::ThemePicker {
                names, highlighted, ..
            } => names[*highlighted].clone(),
            _ => panic!("expected ThemePicker mode"),
        };
        // theme_name should be updated to whatever wrapped-to entry is.
        assert_eq!(app.theme_name, last_name);
        // Wrapping from 0 up lands on a different entry than the default.
        assert_ne!(app.theme_name, crate::theme::DEFAULT_THEME_NAME);
    }

    #[test]
    fn picker_cancel_restores_original() {
        let mut app = test_app();
        reduce(&mut app, Action::OpenThemePicker);
        // Move so theme_name drifts away from the original.
        reduce(&mut app, Action::PickerMove(1));
        assert_ne!(app.theme_name, crate::theme::DEFAULT_THEME_NAME);
        // Cancel must restore the original theme.
        reduce(&mut app, Action::PickerCancel);
        assert_eq!(app.theme_name, crate::theme::DEFAULT_THEME_NAME);
        assert!(matches!(app.mode, Mode::Normal));
    }

    #[test]
    fn picker_confirm_commits_and_returns_to_normal() {
        let mut app = test_app();
        reduce(&mut app, Action::OpenThemePicker);
        reduce(&mut app, Action::PickerMove(1));
        let chosen = app.theme_name.clone();
        assert_ne!(chosen, crate::theme::DEFAULT_THEME_NAME);
        reduce(&mut app, Action::PickerConfirm);
        assert!(matches!(app.mode, Mode::Normal));
        // After confirm the theme_name remains the chosen one.
        assert_eq!(app.theme_name, chosen);
    }
}

fn cycle_history(app: &mut App, dir: i32) {
    let Some(r) = app
        .active_justfile()
        .and_then(|jf| jf.recipes.get(app.list_cursor))
    else {
        return;
    };
    let runs = r.runs.clone();
    if runs.is_empty() {
        return;
    }
    let current_pos = app
        .active_session
        .and_then(|sid| runs.iter().position(|x| *x == sid));
    let next = match current_pos {
        Some(i) => {
            let new_i = (i as i32 + dir).clamp(0, (runs.len() - 1) as i32) as usize;
            runs[new_i]
        }
        None => *runs.last().unwrap(),
    };
    app.active_session = Some(next);
    if let Some(s) = app.session_mut(next) {
        s.unread = false;
    }
}
