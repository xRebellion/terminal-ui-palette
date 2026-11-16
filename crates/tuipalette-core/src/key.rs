//! Keys the way the spec spells them, and the one rule about events: only a
//! press acts.
//!
//! The library never sees a terminal's key type. `Key` is ours: built from a
//! document string (`"ctrl-j"`) or from whatever a host decoded, so adding a
//! front-end means spelling modifiers the way this file does and nothing more.

use std::fmt;

/// A key once its modifiers are stripped.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum KeyCode {
    Char(char),
    Esc,
    Enter,
    Tab,
    Backspace,
    Delete,
    Space,
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
    F(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Mods {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
}

impl Mods {
    pub const NONE: Mods = Mods {
        ctrl: false,
        alt: false,
        shift: false,
    };
    pub const CTRL: Mods = Mods {
        ctrl: true,
        alt: false,
        shift: false,
    };
    pub const ALT: Mods = Mods {
        ctrl: false,
        alt: true,
        shift: false,
    };
    pub const SHIFT: Mods = Mods {
        ctrl: false,
        alt: false,
        shift: true,
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Key {
    pub code: KeyCode,
    pub mods: Mods,
}

/// What a key event is: a hold repeats, a release ends it, and neither is a
/// new keypress. A picker that acts on repeats toggles twice, so the phase is
/// checked once, here (docs/PLAN.md, 04).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Press,
    Repeat,
    Release,
}

/// The only constructor a host should use for input: returns the key on a
/// press and `None` otherwise.
pub fn on(phase: Phase, code: KeyCode, mods: Mods) -> Option<Key> {
    if phase == Phase::Press {
        Some(Key { code, mods })
    } else {
        None
    }
}

impl Key {
    pub const fn new(code: KeyCode, mods: Mods) -> Self {
        Self { code, mods }
    }

    /// `"ctrl-j"`, `"esc"`, `"shift-pageup"`, `"f5"`. Anything else is refused
    /// with a sentence saying what was expected.
    ///
    /// Named keys are spelled as one word — `pageup`, not `page-up` — because
    /// `-` separates modifiers.
    pub fn parse(text: &str) -> Result<Self, String> {
        if text.trim().is_empty() {
            return Err("empty key".to_owned());
        }
        let mut parts: Vec<&str> = text.split('-').collect();
        if parts.iter().any(|part| part.is_empty()) {
            return Err(format!(
                "`{text}` has an empty part — expected a modifier and a key, e.g. `ctrl-j`"
            ));
        }
        let base = parts.pop().expect("one part remains");
        let mut mods = Mods::NONE;
        for part in parts {
            match part.to_ascii_lowercase().as_str() {
                "ctrl" | "control" => mods.ctrl = true,
                "alt" | "meta" | "option" => mods.alt = true,
                "shift" => mods.shift = true,
                other => {
                    return Err(format!(
                        "`{other}` is not a modifier — expected ctrl, alt or shift in `{text}`"
                    ));
                }
            }
        }
        let mut code = parse_code(base)?;
        // A capital letter means shift was held, so `J` and `shift-j` are one
        // bind instead of two that only one terminal reports.
        if let KeyCode::Char(c) = code {
            if c.is_ascii_uppercase() {
                mods.shift = true;
                code = KeyCode::Char(c.to_ascii_lowercase());
            }
        }
        Ok(Key { code, mods })
    }
}

fn parse_code(text: &str) -> Result<KeyCode, String> {
    let lower = text.to_ascii_lowercase();
    let named = match lower.as_str() {
        "esc" | "escape" => Some(KeyCode::Esc),
        "enter" | "return" => Some(KeyCode::Enter),
        "tab" => Some(KeyCode::Tab),
        "backspace" => Some(KeyCode::Backspace),
        "delete" | "del" => Some(KeyCode::Delete),
        "space" => Some(KeyCode::Space),
        "up" => Some(KeyCode::Up),
        "down" => Some(KeyCode::Down),
        "left" => Some(KeyCode::Left),
        "right" => Some(KeyCode::Right),
        "home" => Some(KeyCode::Home),
        "end" => Some(KeyCode::End),
        "pageup" | "pgup" => Some(KeyCode::PageUp),
        "pagedown" | "pgdn" => Some(KeyCode::PageDown),
        _ => None,
    };
    if let Some(code) = named {
        return Ok(code);
    }
    if let Some(digits) = lower.strip_prefix('f') {
        if let Ok(n) = digits.parse::<u8>() {
            if (1..=24).contains(&n) {
                return Ok(KeyCode::F(n));
            }
            return Err(format!("`f{n}` is not a function key — the range is f1 to f24"));
        }
    }
    let mut chars = text.chars();
    match (chars.next(), chars.next()) {
        (Some(' '), None) => Ok(KeyCode::Space),
        (Some(c), None) => Ok(KeyCode::Char(c)),
        _ => Err(format!(
            "`{text}` is not a key — one character, or a name such as esc, enter, tab,              backspace, delete, space, up, down, left, right, home, end, pageup, pagedown, f1..f24"
        )),
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.mods.ctrl {
            f.write_str("ctrl-")?;
        }
        if self.mods.alt {
            f.write_str("alt-")?;
        }
        if self.mods.shift {
            f.write_str("shift-")?;
        }
        let code = match self.code {
            KeyCode::Char(c) => return write!(f, "{c}"),
            KeyCode::Esc => "esc",
            KeyCode::Enter => "enter",
            KeyCode::Tab => "tab",
            KeyCode::Backspace => "backspace",
            KeyCode::Delete => "delete",
            KeyCode::Space => "space",
            KeyCode::Up => "up",
            KeyCode::Down => "down",
            KeyCode::Left => "left",
            KeyCode::Right => "right",
            KeyCode::Home => "home",
            KeyCode::End => "end",
            KeyCode::PageUp => "pageup",
            KeyCode::PageDown => "pagedown",
            KeyCode::F(n) => return write!(f, "f{n}"),
        };
        f.write_str(code)
    }
}
