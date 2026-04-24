use crate::app::types::{Focus, Mode};
use crate::app::App;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum SectionId {
    ListFocus,
    SessionFocus,
    Filter,
    Dropdown,
    Param,
    Confirm,
    Errors,
    HelpItself,
}

pub fn active_section(app: &App) -> SectionId {
    match &app.mode {
        Mode::Help { .. } => SectionId::HelpItself,
        Mode::FilterInput => SectionId::Filter,
        Mode::Dropdown { .. } => SectionId::Dropdown,
        Mode::ParamInput { .. } => SectionId::Param,
        Mode::Confirm { .. } => SectionId::Confirm,
        Mode::ErrorsList => SectionId::Errors,
        Mode::ThemePicker { .. } => SectionId::ListFocus,
        Mode::Normal => match app.focus {
            Focus::Session => SectionId::SessionFocus,
            _ => SectionId::ListFocus,
        },
    }
}
