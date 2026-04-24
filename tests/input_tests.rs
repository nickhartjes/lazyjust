use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use lazyrust::app::types::Mode;
use lazyrust::app::Action;
use lazyrust::input::handle_event;

fn key(c: char) -> Event {
    Event::Key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE))
}

#[test]
fn normal_mode_j_k_quit() {
    assert_eq!(
        handle_event(&key('j'), &Mode::Normal),
        Some(Action::CursorDown)
    );
    assert_eq!(
        handle_event(&key('k'), &Mode::Normal),
        Some(Action::CursorUp)
    );
    assert_eq!(
        handle_event(&key('q'), &Mode::Normal),
        Some(Action::RequestQuit)
    );
    assert_eq!(
        handle_event(&key('/'), &Mode::Normal),
        Some(Action::EnterFilter)
    );
    assert_eq!(
        handle_event(&key('?'), &Mode::Normal),
        Some(Action::OpenHelp)
    );
}

#[test]
fn filter_mode_typing_and_escape() {
    assert_eq!(
        handle_event(&key('a'), &Mode::FilterInput),
        Some(Action::FilterChar('a'))
    );
    let esc = Event::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
    assert_eq!(
        handle_event(&esc, &Mode::FilterInput),
        Some(Action::CancelFilter)
    );
    let bs = Event::Key(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE));
    assert_eq!(
        handle_event(&bs, &Mode::FilterInput),
        Some(Action::FilterBackspace)
    );
}

#[test]
fn split_resize_keys() {
    assert_eq!(
        handle_event(&key('>'), &Mode::Normal),
        Some(Action::GrowLeftPane)
    );
    assert_eq!(
        handle_event(&key('<'), &Mode::Normal),
        Some(Action::ShrinkLeftPane)
    );
    assert_eq!(
        handle_event(&key('='), &Mode::Normal),
        Some(Action::ResetSplit)
    );
}
