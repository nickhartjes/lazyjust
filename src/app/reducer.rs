use super::action::Action;
use super::state::App;
use super::types::{ConfirmAction, Mode};

const SPLIT_MIN: f32 = 0.15;
const SPLIT_MAX: f32 = 0.70;
const SPLIT_STEP: f32 = 0.05;
const SPLIT_DEFAULT: f32 = 0.30;

pub fn reduce(app: &mut App, action: Action) {
    match action {
        Action::NoOp => {}

        Action::CursorDown => cursor_down(app),
        Action::CursorUp => cursor_up(app),

        Action::EnterFilter => app.mode = Mode::FilterInput,
        Action::FilterChar(c) => filter_char(app, c),
        Action::FilterBackspace => filter_backspace(app),
        Action::CommitFilter => commit_filter(app),
        Action::CancelFilter => cancel_filter(app),

        Action::GrowLeftPane => grow_left_pane(app),
        Action::ShrinkLeftPane => shrink_left_pane(app),
        Action::ResetSplit => app.split_ratio = SPLIT_DEFAULT,

        Action::RequestQuit => request_quit(app),
        Action::CancelConfirm => app.mode = Mode::Normal,
        Action::Quit | Action::ConfirmQuit => {
            // handled by event loop — reducer leaves it as signal
        }

        Action::OpenHelp => open_help(app),
        Action::CloseHelp => app.mode = Mode::Normal,
        Action::HelpScrollDown(n) => help_scroll_down(app, n),
        Action::HelpScrollUp(n) => help_scroll_up(app, n),
        Action::HelpScrollHome => help_scroll_home(app),
        Action::HelpScrollEnd => help_scroll_end(app),

        Action::OpenErrors => app.mode = Mode::ErrorsList,
        Action::CloseErrors => app.mode = Mode::Normal,

        Action::OpenDropdown => open_dropdown(app),
        Action::DropdownChar(c) => dropdown_char(app, c),
        Action::DropdownBackspace => dropdown_backspace(app),
        Action::DropdownCursorDown => dropdown_cursor_down(app),
        Action::DropdownCursorUp => dropdown_cursor_up(app),
        Action::SelectDropdown => select_dropdown(app),
        Action::CancelDropdown => app.mode = Mode::Normal,

        Action::SessionExited { id, code } => session_exited(app, id, code),
        Action::RecipeExited { id, code } => recipe_exited(app, id, code),
        Action::MarkUnread(id) => mark_unread(app, id),
        Action::MarkRead(id) => mark_read(app, id),

        Action::CycleFocus => cycle_focus(app),
        Action::FocusList => app.focus = crate::app::types::Focus::List,
        Action::FocusSession => app.focus = crate::app::types::Focus::Session,
        Action::FocusNextSession => focus_next_session(app),
        Action::FocusPrevSession => focus_prev_session(app),

        Action::ParamChar(c) => param_char(app, c),
        Action::ParamBackspace => param_backspace(app),
        Action::ParamNext => param_next(app),
        Action::CancelParam => app.mode = Mode::Normal,
        // ParamCommit handled by event_loop (needs side effects)
        Action::CycleRecipeHistoryPrev => cycle_history(app, -1),
        Action::CycleRecipeHistoryNext => cycle_history(app, 1),

        Action::RequestKillSession => request_kill_session(app),
        Action::RequestCloseSession => request_close_session(app),
        Action::KillSession(id) => kill_session(app, id),
        Action::CloseSession(id) => close_session(app, id),
        Action::CopyLogPath => copy_log_path(app),

        Action::OpenThemePicker => open_theme_picker(app),
        Action::PickerMove(delta) => picker_move(app, delta),
        Action::PickerConfirm => picker_confirm(app),
        Action::PickerCancel => picker_cancel(app),

        // Remaining actions handled in later tasks.
        _ => {}
    }
}

fn cursor_down(app: &mut App) {
    let max = app.view.recipe_count().saturating_sub(1);
    if app.list_cursor < max {
        app.list_cursor += 1;
    }
}

fn cursor_up(app: &mut App) {
    app.list_cursor = app.list_cursor.saturating_sub(1);
}

fn filter_char(app: &mut App, c: char) {
    if app.mode == Mode::FilterInput {
        app.filter.push(c);
    }
}

fn filter_backspace(app: &mut App) {
    if app.mode == Mode::FilterInput {
        app.filter.pop();
    }
}

fn commit_filter(app: &mut App) {
    if app.mode == Mode::FilterInput {
        app.mode = Mode::Normal;
    }
}

fn cancel_filter(app: &mut App) {
    app.filter.clear();
    app.mode = Mode::Normal;
}

fn grow_left_pane(app: &mut App) {
    app.split_ratio = (app.split_ratio + SPLIT_STEP).min(SPLIT_MAX);
}

fn shrink_left_pane(app: &mut App) {
    app.split_ratio = (app.split_ratio - SPLIT_STEP).max(SPLIT_MIN);
}

fn request_quit(app: &mut App) {
    if app.sessions.iter().any(|s| {
        matches!(
            s.status,
            crate::app::types::Status::Running | crate::app::types::Status::ShellAfterExit { .. }
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

fn open_help(app: &mut App) {
    let origin = crate::app::help_section::active_section(app);
    app.mode = Mode::Help { scroll: 0, origin };
}

fn help_scroll_down(app: &mut App, n: u16) {
    if let Mode::Help { scroll, .. } = &mut app.mode {
        *scroll = scroll.saturating_add(n);
    }
}

fn help_scroll_up(app: &mut App, n: u16) {
    if let Mode::Help { scroll, .. } = &mut app.mode {
        *scroll = scroll.saturating_sub(n);
    }
}

fn help_scroll_home(app: &mut App) {
    if let Mode::Help { scroll, .. } = &mut app.mode {
        *scroll = 0;
    }
}

fn help_scroll_end(app: &mut App) {
    if let Mode::Help { scroll, .. } = &mut app.mode {
        *scroll = u16::MAX;
    }
}

fn open_dropdown(app: &mut App) {
    app.mode = Mode::Dropdown {
        filter: String::new(),
        cursor: app.active_justfile,
    };
}

fn dropdown_char(app: &mut App, c: char) {
    if let Mode::Dropdown { filter, cursor } = &mut app.mode {
        filter.push(c);
        *cursor = 0;
    }
}

fn dropdown_backspace(app: &mut App) {
    if let Mode::Dropdown { filter, .. } = &mut app.mode {
        filter.pop();
    }
}

fn dropdown_cursor_down(app: &mut App) {
    let max = app.justfiles.len().saturating_sub(1);
    if let Mode::Dropdown { cursor, .. } = &mut app.mode {
        if *cursor < max {
            *cursor += 1;
        }
    }
}

fn dropdown_cursor_up(app: &mut App) {
    if let Mode::Dropdown { cursor, .. } = &mut app.mode {
        *cursor = cursor.saturating_sub(1);
    }
}

fn select_dropdown(app: &mut App) {
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

fn session_exited(app: &mut App, id: crate::app::types::SessionId, code: i32) {
    if let Some(s) = app.session_mut(id) {
        if matches!(s.status, crate::app::types::Status::Running) {
            s.status = crate::app::types::Status::Exited { code };
            s.unread = true;
        } else if let crate::app::types::Status::ShellAfterExit { .. } = s.status {
            s.status = crate::app::types::Status::Exited { code };
        } else {
            // Status is already Exited; another channel reported the exit first.
        }
    }
}

fn recipe_exited(app: &mut App, id: crate::app::types::SessionId, code: i32) {
    let is_active = Some(id) == app.active_session;
    if let Some(s) = app.session_mut(id) {
        s.status = crate::app::types::Status::ShellAfterExit { code };
        if !is_active {
            s.unread = true;
        }
    }
}

fn mark_unread(app: &mut App, id: crate::app::types::SessionId) {
    if let Some(s) = app.session_mut(id) {
        s.unread = true;
    }
}

fn mark_read(app: &mut App, id: crate::app::types::SessionId) {
    if let Some(s) = app.session_mut(id) {
        s.unread = false;
    }
}

fn cycle_focus(app: &mut App) {
    app.focus = match app.focus {
        crate::app::types::Focus::List => crate::app::types::Focus::Session,
        crate::app::types::Focus::Session => crate::app::types::Focus::List,
        other => other,
    };
}

fn focus_next_session(app: &mut App) {
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
    } else {
        // No active session and no sessions exist; nothing to focus.
    }
}

fn focus_prev_session(app: &mut App) {
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

fn param_char(app: &mut App, c: char) {
    if let Mode::ParamInput { values, cursor, .. } = &mut app.mode {
        if let Some(v) = values.get_mut(*cursor) {
            v.push(c);
        }
    }
}

fn param_backspace(app: &mut App) {
    if let Mode::ParamInput { values, cursor, .. } = &mut app.mode {
        if let Some(v) = values.get_mut(*cursor) {
            v.pop();
        }
    }
}

fn param_next(app: &mut App) {
    if let Mode::ParamInput { values, cursor, .. } = &mut app.mode {
        if *cursor + 1 < values.len() {
            *cursor += 1;
        }
    }
}

fn request_kill_session(app: &mut App) {
    if let Some(id) = app.active_session {
        app.mode = Mode::Confirm {
            prompt: format!("Kill session {id}?"),
            on_accept: crate::app::types::ConfirmAction::KillSession(id),
        };
    }
}

fn request_close_session(app: &mut App) {
    if let Some(id) = app.active_session {
        app.mode = Mode::Confirm {
            prompt: format!("Close session {id}?"),
            on_accept: crate::app::types::ConfirmAction::CloseSession(id),
        };
    }
}

fn kill_session(app: &mut App, id: crate::app::types::SessionId) {
    if let Some(s) = app.session_mut(id) {
        s.status = crate::app::types::Status::Exited { code: 130 };
    }
    // actual PTY kill done in event loop
}

fn close_session(app: &mut App, id: crate::app::types::SessionId) {
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

fn copy_log_path(app: &mut App) {
    if let Some(id) = app.active_session {
        if let Some(s) = app.session(id) {
            if let Ok(mut cb) = arboard::Clipboard::new() {
                let _ = cb.set_text(s.log_path.display().to_string());
                app.status_message = Some(format!("copied {}", s.log_path.display()));
            }
        }
    }
}

fn open_theme_picker(app: &mut App) {
    let names = crate::theme::registry::list();
    let original_name = app.theme_name.clone();
    let highlighted = names.iter().position(|n| *n == original_name).unwrap_or(0);
    app.mode = Mode::ThemePicker {
        original_name,
        highlighted,
        names,
    };
}

fn picker_move(app: &mut App, delta: isize) {
    if let Mode::ThemePicker {
        highlighted, names, ..
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

fn picker_confirm(app: &mut App) {
    if let Mode::ThemePicker { .. } = app.mode {
        let stem = app.theme_name.clone();
        let path = crate::config::paths::config_file_path();
        if let Err(e) = crate::config::writer::set_theme(&path, &stem) {
            tracing::warn!(
                target: "lazyjust::theme",
                error = %e,
                "failed to persist theme",
            );
            app.status_message = Some(format!("theme persist failed: {e}"));
        }
        app.mode = Mode::Normal;
    }
}

fn picker_cancel(app: &mut App) {
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

pub fn filtered_justfile_indices(app: &App, filter: &str) -> Vec<usize> {
    let paths: Vec<String> = app
        .justfiles
        .iter()
        .map(|j| j.path.display().to_string())
        .collect();
    let refs: Vec<&str> = paths.iter().map(String::as_str).collect();
    crate::app::filter::fuzzy_match(&refs, filter)
        .into_iter()
        .map(|(i, _)| i)
        .collect()
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
            crate::ui::icon_style::IconStyle::Round,
            crate::app::types::ListMode::Active,
            std::path::PathBuf::from("."),
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
        reduce(&mut app, Action::PickerMove(-1));
        let last_name = match &app.mode {
            Mode::ThemePicker {
                names, highlighted, ..
            } => names[*highlighted].clone(),
            _ => panic!("expected ThemePicker mode"),
        };
        assert_eq!(app.theme_name, last_name);
        assert_ne!(app.theme_name, crate::theme::DEFAULT_THEME_NAME);
    }

    #[test]
    fn picker_cancel_restores_original() {
        let mut app = test_app();
        reduce(&mut app, Action::OpenThemePicker);
        reduce(&mut app, Action::PickerMove(1));
        assert_ne!(app.theme_name, crate::theme::DEFAULT_THEME_NAME);
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
        assert_eq!(app.theme_name, chosen);
    }
}
