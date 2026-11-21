//! Terminal keys in, our keys out. This is the only place that knows what a
//! terminal's key event looks like, and the only place that drops a release or a
//! repeat before the router sees it (docs/PLAN.md, D1).

use crossterm::event::{
    KeyCode as TermCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent,
    MouseEventKind,
};
use tuipalette_core::key::{self, Key, KeyCode, Mods, Phase};
use tuipalette_core::{Mouse, MouseKind};

/// One terminal mouse event as ours, or `None` for one the picker has no use
/// for: a release, a right or middle button, a sideways wheel. A drag is a
/// move — the pointer is still somewhere, and hover should follow it.
pub fn translate_mouse(event: MouseEvent) -> Option<Mouse> {
    let kind = match event.kind {
        MouseEventKind::Moved | MouseEventKind::Drag(_) => MouseKind::Moved,
        MouseEventKind::Down(MouseButton::Left) => MouseKind::Click,
        MouseEventKind::ScrollUp => MouseKind::ScrollUp,
        MouseEventKind::ScrollDown => MouseKind::ScrollDown,
        _ => return None,
    };
    Some(Mouse::new(kind, event.column, event.row))
}

/// One terminal event as a keypress, or `None` for an event that is not one.
pub fn translate(event: KeyEvent) -> Option<Key> {
    let phase = match event.kind {
        KeyEventKind::Press => Phase::Press,
        KeyEventKind::Repeat => Phase::Repeat,
        KeyEventKind::Release => Phase::Release,
    };
    let code = code(event.code)?;
    let mut mods = Mods {
        ctrl: event.modifiers.contains(KeyModifiers::CONTROL),
        alt: event.modifiers.contains(KeyModifiers::ALT),
        shift: event.modifiers.contains(KeyModifiers::SHIFT),
    };
    // BackTab is what a terminal sends for shift-tab.
    if matches!(event.code, TermCode::BackTab) {
        mods.shift = true;
    }
    key::on(phase, code, mods)
}

fn code(code: TermCode) -> Option<KeyCode> {
    let key = match code {
        // The character is kept as typed: the query should show "Firefox" when
        // the user typed it, while a bind compares the normalised form.
        TermCode::Char(c) => KeyCode::Char(c),
        TermCode::Enter => KeyCode::Enter,
        TermCode::Esc => KeyCode::Esc,
        TermCode::Tab | TermCode::BackTab => KeyCode::Tab,
        TermCode::Backspace => KeyCode::Backspace,
        TermCode::Delete => KeyCode::Delete,
        TermCode::Up => KeyCode::Up,
        TermCode::Down => KeyCode::Down,
        TermCode::Left => KeyCode::Left,
        TermCode::Right => KeyCode::Right,
        TermCode::Home => KeyCode::Home,
        TermCode::End => KeyCode::End,
        TermCode::PageUp => KeyCode::PageUp,
        TermCode::PageDown => KeyCode::PageDown,
        TermCode::F(n) if (1..=24).contains(&n) => KeyCode::F(n),
        _ => return None,
    };
    Some(key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn event(code: TermCode, mods: KeyModifiers, kind: KeyEventKind) -> KeyEvent {
        let mut event = KeyEvent::new(code, mods);
        event.kind = kind;
        event
    }

    #[test]
    fn a_press_becomes_a_key() {
        let pressed = event(TermCode::Char('j'), KeyModifiers::CONTROL, KeyEventKind::Press);
        assert_eq!(translate(pressed), Some(Key::new(KeyCode::Char('j'), Mods::CTRL)));
    }

    #[test]
    fn a_repeat_or_a_release_is_dropped() {
        let repeat = event(TermCode::Down, KeyModifiers::NONE, KeyEventKind::Repeat);
        assert_eq!(translate(repeat), None);
        let release = event(TermCode::Down, KeyModifiers::NONE, KeyEventKind::Release);
        assert_eq!(translate(release), None);
        let _ = Duration::from_secs(0);
    }

    #[test]
    fn an_upper_case_letter_is_shift_plus_the_letter() {
        let upper = event(TermCode::Char('J'), KeyModifiers::SHIFT, KeyEventKind::Press);
        let key = translate(upper).expect("a key");
        assert_eq!(key.normalised(), Key::new(KeyCode::Char('j'), Mods::SHIFT));
    }

    fn mouse(kind: MouseEventKind) -> MouseEvent {
        MouseEvent { kind, column: 7, row: 3, modifiers: KeyModifiers::NONE }
    }

    #[test]
    fn the_mouse_events_a_picker_uses_are_translated() {
        let at = |kind| Some(Mouse::new(kind, 7, 3));
        assert_eq!(translate_mouse(mouse(MouseEventKind::Moved)), at(MouseKind::Moved));
        assert_eq!(translate_mouse(mouse(MouseEventKind::Drag(MouseButton::Left))), at(MouseKind::Moved));
        assert_eq!(translate_mouse(mouse(MouseEventKind::Down(MouseButton::Left))), at(MouseKind::Click));
        assert_eq!(translate_mouse(mouse(MouseEventKind::ScrollUp)), at(MouseKind::ScrollUp));
        assert_eq!(translate_mouse(mouse(MouseEventKind::ScrollDown)), at(MouseKind::ScrollDown));
    }

    #[test]
    fn the_rest_of_the_mouse_is_dropped() {
        for kind in [
            MouseEventKind::Up(MouseButton::Left),
            MouseEventKind::Down(MouseButton::Right),
            MouseEventKind::Down(MouseButton::Middle),
            MouseEventKind::ScrollLeft,
            MouseEventKind::ScrollRight,
        ] {
            assert_eq!(translate_mouse(mouse(kind)), None, "{kind:?}");
        }
    }

    #[test]
    fn back_tab_is_shift_tab() {
        let back = event(TermCode::BackTab, KeyModifiers::SHIFT, KeyEventKind::Press);
        assert_eq!(
            translate(back),
            Some(Key::new(KeyCode::Tab, Mods::SHIFT))
        );
    }
}
