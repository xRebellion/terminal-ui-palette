//! The terminal side of tuipalette: a crossterm backend, key decoding and the
//! draw loop. Everything that knows a terminal exists lives here, which is what
//! keeps `tuipalette-core` testable headless (docs/PLAN.md, D1).

pub mod input;
pub mod run;

pub use run::{run, Exit, Options};

/// How many lines the terminal has, for a default inline height.
pub fn terminal_rows() -> Option<u16> {
    crossterm::terminal::size().ok().map(|(_, rows)| rows)
}
