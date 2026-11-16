//! Keys from the document, resolved per pane over the global map.
//!
//! The merge is the whole point: a pane overrides the global map where it says
//! something, and an explicit `None` clears an inherited bind instead of falling
//! through to it, so "this pane does not answer to ctrl-j" is writable
//! (docs/PLAN.md, D8).

use std::collections::BTreeMap;

use crate::action::Action;
use crate::error::SpecError;
use crate::key::Key;
use crate::spec::UiSpec;

#[derive(Debug, Clone, Default)]
pub struct Keymap {
    global: BTreeMap<Key, Option<Action>>,
    panes: BTreeMap<String, BTreeMap<Key, Option<Action>>>,
}

impl Keymap {
    /// Parse every key string in the document. A key that is not a key is
    /// refused here, with its path, rather than binding nothing at runtime.
    pub fn from_spec(spec: &UiSpec) -> Result<Self, SpecError> {
        let global = parse_map(&spec.keys, "keys")?;
        let mut panes = BTreeMap::new();
        for (i, pane) in spec.panes.iter().enumerate() {
            let path = format!("panes[{i}].keys");
            panes.insert(pane.id.clone(), parse_map(&pane.keys, &path)?);
        }
        Ok(Self { global, panes })
    }

    /// The bind that applies while `pane` has focus.
    pub fn resolve(&self, pane: &str, key: Key) -> Option<&Action> {
        if let Some(bind) = self.panes.get(pane).and_then(|map| map.get(&key)) {
            return bind.as_ref();
        }
        self.global.get(&key).and_then(Option::as_ref)
    }

    /// Everything the pane answers to, inherited binds included — what a help
    /// footer or the reference doc enumerates.
    pub fn effective(&self, pane: &str) -> BTreeMap<Key, &Action> {
        let mut out: BTreeMap<Key, &Action> = BTreeMap::new();
        for (key, action) in &self.global {
            if let Some(action) = action {
                out.insert(*key, action);
            }
        }
        if let Some(own) = self.panes.get(pane) {
            for (key, action) in own {
                match action {
                    Some(action) => {
                        out.insert(*key, action);
                    }
                    None => {
                        out.remove(key);
                    }
                }
            }
        }
        out
    }
}

fn parse_map(
    map: &BTreeMap<String, Option<Action>>,
    path: &str,
) -> Result<BTreeMap<Key, Option<Action>>, SpecError> {
    let mut out = BTreeMap::new();
    for (text, action) in map {
        let key = Key::parse(text)
            .map_err(|detail| SpecError::at_path(format!("{path}[\"{text}\"]"), detail))?;
        out.insert(key, action.clone());
    }
    Ok(out)
}
