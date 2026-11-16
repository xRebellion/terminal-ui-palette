//! Spec, layout resolution and the component model.
//!
//! This crate never touches a terminal: it turns a RON (or JSON) document into
//! `Rect`s and drives `Component`s through `Action`s. Rendering into a real or
//! fake terminal lives in `tuipalette-tui`, which is what keeps every component
//! testable headless (docs/PLAN.md, D1).

pub mod action;
pub mod error;
pub mod layout;
pub mod spec;

/// The one ratatui type allowed across this boundary — re-inventing geometry
/// would buy nothing (docs/PLAN.md, Scope).
pub use ratatui_core::layout::Rect;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
