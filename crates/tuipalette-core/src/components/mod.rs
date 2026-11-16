//! The components a document can ask for by name.
//!
//! `kind: Input` and friends map to exactly these, and nothing else in the
//! library knows they exist — a picker's own component is just another
//! `Component` impl handed to `Session` (docs/PLAN.md, D5).

pub mod input;
pub mod menu;
pub mod spacer;
pub mod status;
pub mod text;

pub use input::InputLine;
pub use menu::{Item, Menu, Row};
pub use spacer::Spacer;
pub use status::StatusLine;
pub use text::TextPane;

use crate::component::Component;
use crate::spec::{Kind, Pane};

/// The built-in a pane asks for by `kind`.
pub fn from_pane(pane: &Pane) -> Box<dyn Component> {
    match pane.kind {
        Kind::Input => Box::new(InputLine::new(pane)),
        Kind::Menu => Box::new(Menu::new(pane)),
        Kind::Text => Box::new(TextPane::new(pane)),
        Kind::Status => Box::new(StatusLine::new(pane)),
        Kind::Spacer => Box::new(Spacer::new(pane)),
    }
}
