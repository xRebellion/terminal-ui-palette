//! Actions and the key map.
//!
//! A component never sees a `KeyEvent`: everything it can do is one flat
//! `Action`, so the same action can come from a key today and from a mouse
//! click or a script later.

use std::collections::BTreeMap;

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub enum Action {
    Quit,
    FocusNext,
    FocusPrev,
    Up,
    Down,
    Accept,
    Backspace,
    Input(char),
    /// Emitted by a component: an app-level command with a payload.
    Invoke(String),
}

/// One bound key. `action: None` is an explicit unset (D8): it stops the
/// lookup, it does not fall through to the global map.
#[derive(Debug, Clone)]
pub struct Bind {
    pub code: KeyCode,
    pub mods: KeyModifiers,
    pub action: Option<Action>,
}

#[derive(Debug, Default)]
pub struct KeyMaps {
    pub global: Vec<Bind>,
    pub per_pane: Vec<(String, Vec<Bind>)>,
}

impl KeyMaps {
    pub fn build(
        global: &BTreeMap<String, Option<Action>>,
        panes: &[(String, BTreeMap<String, Option<Action>>)],
    ) -> Self {
        Self {
            global: binds(global),
            per_pane: panes
                .iter()
                .map(|(id, map)| (id.clone(), binds(map)))
                .collect(),
        }
    }

    pub fn resolve(&self, pane: &str, key: KeyEvent) -> Option<Action> {
        if key.kind != KeyEventKind::Press {
            return None;
        }
        if let Some((_, list)) = self.per_pane.iter().find(|(id, _)| id == pane)
            && let Some(b) = list.iter().find(|b| matches(b, key))
        {
            return b.action.clone();
        }
        if let Some(b) = self.global.iter().find(|b| matches(b, key)) {
            return b.action.clone();
        }
        if key.modifiers.is_empty()
            && let KeyCode::Char(c) = key.code
        {
            return Some(Action::Input(c));
        }
        None
    }
}

fn matches(b: &Bind, key: KeyEvent) -> bool {
    b.code == key.code && b.mods == key.modifiers
}

fn binds(map: &BTreeMap<String, Option<Action>>) -> Vec<Bind> {
    map.iter()
        .filter_map(|(k, action)| {
            let (code, mods) = parse_key(k)?;
            Some(Bind {
                code,
                mods,
                action: action.clone(),
            })
        })
        .collect()
}

/// `"esc"`, `"tab"`, `"ctrl-j"`, `"alt-x"`, `"space"`, `"home"`, a single char…
pub fn parse_key(spec: &str) -> Option<(KeyCode, KeyModifiers)> {
    let lower = spec.trim().to_ascii_lowercase();
    let parts: Vec<&str> = lower.split('-').collect();
    let (mods_part, key_part) = parts.split_at(parts.len().saturating_sub(1));
    let mut mods = KeyModifiers::empty();
    for m in mods_part {
        mods |= match *m {
            "ctrl" | "c" => KeyModifiers::CONTROL,
            "alt" | "m" => KeyModifiers::ALT,
            "shift" | "s" => KeyModifiers::SHIFT,
            "super" | "meta" => KeyModifiers::SUPER,
            _ => return None,
        };
    }
    let code = match *key_part.first()? {
        "enter" | "return" => KeyCode::Enter,
        "esc" | "escape" => KeyCode::Esc,
        "tab" => KeyCode::Tab,
        "backtab" => KeyCode::BackTab,
        "backspace" => KeyCode::Backspace,
        "space" => KeyCode::Char(' '),
        "up" => KeyCode::Up,
        "down" => KeyCode::Down,
        "left" => KeyCode::Left,
        "right" => KeyCode::Right,
        "home" => KeyCode::Home,
        "end" => KeyCode::End,
        other => {
            let mut chars = other.chars();
            let c = chars.next()?;
            if chars.next().is_some() {
                return None;
            }
            KeyCode::Char(c)
        }
    };
    Some((code, mods))
}
