use crate::app::types::Mode;
use crate::app::Action;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

pub fn handle_event(evt: &Event, mode: &Mode) -> Option<Action> {
    let key = match evt {
        Event::Key(k) => k,
        Event::Resize(_, _) => return None,
        _ => return None,
    };

    match mode {
        Mode::Normal => normal_mode(key),
        Mode::FilterInput => filter_mode(key),
        Mode::Help { .. } => help_mode(key),
        Mode::Confirm { .. } => confirm_mode(key),
        Mode::Dropdown { .. } => dropdown_mode(key),
        Mode::ParamInput { .. } => param_mode(key),
        Mode::ErrorsList => errors_mode(key),
    }
}

fn normal_mode(k: &KeyEvent) -> Option<Action> {
    let shift = k.modifiers.contains(KeyModifiers::SHIFT);
    let ctrl = k.modifiers.contains(KeyModifiers::CONTROL);
    match (k.code, ctrl, shift) {
        (KeyCode::Char('q'), false, _) => Some(Action::RequestQuit),
        (KeyCode::Char('j'), false, _) | (KeyCode::Down, _, _) => Some(Action::CursorDown),
        (KeyCode::Char('k'), false, _) | (KeyCode::Up, _, _) => Some(Action::CursorUp),
        (KeyCode::Char('h'), false, _) | (KeyCode::Left, _, _) => {
            Some(Action::CycleRecipeHistoryPrev)
        }
        (KeyCode::Char('l'), false, _) | (KeyCode::Right, _, _) => {
            Some(Action::CycleRecipeHistoryNext)
        }
        (KeyCode::Char('/'), false, _) => Some(Action::EnterFilter),
        (KeyCode::Char('d'), false, _) => Some(Action::OpenDropdown),
        (KeyCode::Char('?'), false, _) => Some(Action::OpenHelp),
        (KeyCode::Char('e'), false, _) => Some(Action::OpenErrors),
        (KeyCode::Char('>'), false, _) => Some(Action::GrowLeftPane),
        (KeyCode::Char('<'), false, _) => Some(Action::ShrinkLeftPane),
        (KeyCode::Char('='), false, _) => Some(Action::ResetSplit),
        (KeyCode::Tab, _, _) => Some(Action::CycleFocus),
        (KeyCode::Char('K'), false, true) => Some(Action::RequestKillSession),
        (KeyCode::Char('x'), false, _) => Some(Action::RequestCloseSession),
        (KeyCode::Char('L'), false, true) => Some(Action::CopyLogPath),
        (KeyCode::Char('r'), false, _) => Some(Action::RunHighlighted { force_new: true }),
        (KeyCode::Enter, _, true) => Some(Action::RunHighlighted { force_new: true }),
        (KeyCode::Enter, _, _) => Some(Action::RunHighlighted { force_new: false }),
        (KeyCode::Char('o'), true, _) => Some(Action::FocusNextSession),
        (KeyCode::Char('i'), true, _) => Some(Action::FocusPrevSession),
        (KeyCode::F(12), _, _) => Some(Action::FocusList),
        _ => None,
    }
}

fn filter_mode(k: &KeyEvent) -> Option<Action> {
    match k.code {
        KeyCode::Esc => Some(Action::CancelFilter),
        KeyCode::Enter => Some(Action::CommitFilter),
        KeyCode::Backspace => Some(Action::FilterBackspace),
        KeyCode::Char(c) => Some(Action::FilterChar(c)),
        _ => None,
    }
}

fn help_mode(k: &KeyEvent) -> Option<Action> {
    match k.code {
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') | KeyCode::F(1) => {
            Some(Action::CloseHelp)
        }
        KeyCode::Char('j') | KeyCode::Down => Some(Action::HelpScrollDown(1)),
        KeyCode::Char('k') | KeyCode::Up => Some(Action::HelpScrollUp(1)),
        KeyCode::PageDown => Some(Action::HelpScrollDown(10)),
        KeyCode::PageUp => Some(Action::HelpScrollUp(10)),
        KeyCode::Home => Some(Action::HelpScrollHome),
        KeyCode::End => Some(Action::HelpScrollEnd),
        _ => None,
    }
}

fn errors_mode(k: &KeyEvent) -> Option<Action> {
    match k.code {
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('e') => Some(Action::CloseErrors),
        _ => None,
    }
}

fn confirm_mode(k: &KeyEvent) -> Option<Action> {
    match k.code {
        KeyCode::Char('y') | KeyCode::Enter | KeyCode::Char('q') => Some(Action::ConfirmQuit),
        KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('c') => Some(Action::CancelConfirm),
        _ => None,
    }
}

fn dropdown_mode(k: &KeyEvent) -> Option<Action> {
    match k.code {
        KeyCode::Esc => Some(Action::CancelDropdown),
        KeyCode::Enter => Some(Action::SelectDropdown),
        KeyCode::Down | KeyCode::Char('j') => Some(Action::DropdownCursorDown),
        KeyCode::Up | KeyCode::Char('k') => Some(Action::DropdownCursorUp),
        KeyCode::Backspace => Some(Action::DropdownBackspace),
        KeyCode::Char(c) => Some(Action::DropdownChar(c)),
        _ => None,
    }
}

fn param_mode(k: &KeyEvent) -> Option<Action> {
    match k.code {
        KeyCode::Esc => Some(Action::CancelParam),
        KeyCode::Tab => Some(Action::ParamNext),
        KeyCode::Enter => Some(Action::ParamCommit),
        KeyCode::Backspace => Some(Action::ParamBackspace),
        KeyCode::Char(c) => Some(Action::ParamChar(c)),
        _ => None,
    }
}
