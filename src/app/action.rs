use super::types::{ConfirmAction, SessionId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Quit,
    RequestQuit,
    ConfirmQuit,
    CancelConfirm,

    CursorUp,
    CursorDown,
    ToggleGroupCollapse,

    EnterFilter,
    FilterChar(char),
    FilterBackspace,
    CommitFilter,
    CancelFilter,

    OpenDropdown,
    DropdownCursorUp,
    DropdownCursorDown,
    DropdownChar(char),
    DropdownBackspace,
    SelectDropdown,
    CancelDropdown,

    RunHighlighted { force_new: bool },
    CycleRecipeHistoryPrev,
    CycleRecipeHistoryNext,

    ParamChar(char),
    ParamBackspace,
    ParamNext,
    ParamCommit,
    CancelParam,

    FocusNextSession,
    FocusPrevSession,
    FocusList,
    FocusSession,
    CycleFocus,
    RequestKillSession,
    RequestCloseSession,
    KillSession(SessionId),
    CloseSession(SessionId),
    CopyLogPath,

    GrowLeftPane,
    ShrinkLeftPane,
    ResetSplit,

    OpenHelp,
    CloseHelp,

    Confirm(ConfirmAction),

    // PTY / external events folded into reducer via dispatcher:
    SessionExited { id: SessionId, code: i32 },
    RecipeExited { id: SessionId, code: i32 },
    MarkUnread(SessionId),
    MarkRead(SessionId),

    NoOp,
}

#[derive(Debug)]
pub enum AppEvent {
    Crossterm(crossterm::event::Event),
    SessionBytes { id: SessionId, bytes: Vec<u8> },
    SessionExited { id: SessionId, code: i32 },
    RecipeExited { id: SessionId, code: i32 },
    Tick,
}
