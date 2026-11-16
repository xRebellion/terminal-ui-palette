//! Spec, layout resolution and the component model.
//!
//! This crate never touches a terminal: it turns a RON (or JSON) document into
//! `Rect`s and drives `Component`s through `Action`s. Rendering into a real or
//! fake terminal lives in `tuipalette-tui`, which is what keeps every component
//! testable headless (docs/PLAN.md, D1).

pub mod action;
pub mod component;
pub mod error;
pub mod key;
pub mod keymap;
pub mod layout;
pub mod session;
pub mod spec;

// The ratatui-core types allowed across this boundary — geometry, a cell grid,
// and styling. Re-inventing them would buy nothing; the `ratatui` facade and
// `crossterm` stay out of this crate entirely (docs/PLAN.md, D1).
pub use ratatui_core::buffer::Buffer;
pub use ratatui_core::layout::Rect;
pub use ratatui_core::style::{Color, Modifier, Style};

pub use action::Action;
pub use component::{Component, Paint, Reaction};
pub use error::SpecError;
pub use key::{Key, KeyCode, Mods, Phase};
pub use keymap::Keymap;
pub use session::{Outcome, Session};
pub use spec::{Format, UiSpec};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
