//! What a pane is made of: a box that draws itself and answers an action.
//!
//! A component owns its state and replies with a `Reaction` instead of touching
//! anything global, so focus, key maps and quitting stay in `Session` where they
//! can be tested without a terminal. Extending the library means writing one of
//! these and recompiling — there is no plugin ABI (docs/PLAN.md, D5).

use std::any::Any;

use crate::action::Action;
use crate::spec::Theme;
use crate::{Buffer, Rect};

/// What a component did with an action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reaction {
    /// Not mine — the router keeps looking.
    Ignored,
    /// Mine, and finished.
    Handled,
    /// Mine, and it asks the app for more. `Emit(Action::Invoke(name))` is the
    /// command path; any other action is dispatched again from the top, which is
    /// how a component can move focus.
    Emit(Action),
}

/// Everything a component needs to draw itself: where, whether it has focus, and
/// the theme to take colours from.
#[derive(Clone, Copy)]
pub struct Paint<'a> {
    pub area: Rect,
    pub focused: bool,
    pub theme: &'a Theme,
}

/// A pane's behaviour.
///
/// `Any` is a supertrait so the picker can reach a built-in by type
/// (`Session::pane_mut::<Menu>`); it changes nothing for an implementor.
pub trait Component: Any {
    /// Must match a pane id in the spec.
    fn id(&self) -> &str;

    /// Whether the focus ring may land here. A label answers no.
    fn focusable(&self) -> bool {
        true
    }

    /// Draw into `buf`. Never a `Frame`: a component that cannot paint into a
    /// plain buffer cannot be tested without a terminal (D1).
    fn render(&self, paint: &Paint<'_>, buf: &mut Buffer);

    fn update(&mut self, _action: &Action) -> Reaction {
        Reaction::Ignored
    }

    /// What the picker hands back when this pane is the one that accepts.
    fn value(&self) -> Option<String> {
        None
    }
}
