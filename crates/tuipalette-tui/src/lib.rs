//! The terminal backend: event loop, `Buffer` rendering, viewports, icon widths.
//!
//! Everything that knows a terminal exists lives here, so `tuipalette-core`
//! stays headless and `TestBackend` keeps working for the whole draw path.

pub use ratatui;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
