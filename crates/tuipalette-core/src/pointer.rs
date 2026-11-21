//! The mouse, as the library sees it: four things a pointer can do, at a cell.
//!
//! Like `Key`, this is the core's own type — the terminal crate translates
//! crossterm's events into it — so a component that answers the mouse is still
//! testable without a terminal (docs/PLAN.md, D1 and 21).

/// What the pointer did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseKind {
    /// It moved, with or without a button held.
    Moved,
    /// The primary button went down.
    Click,
    /// The wheel turned away from the user.
    ScrollUp,
    /// The wheel turned towards the user.
    ScrollDown,
}

/// One pointer event at a cell, 0-based in the same coordinates the frame is
/// drawn in — so an inline viewport's rows are the terminal's rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mouse {
    pub kind: MouseKind,
    pub column: u16,
    pub row: u16,
}

impl Mouse {
    pub fn new(kind: MouseKind, column: u16, row: u16) -> Self {
        Self { kind, column, row }
    }

    /// The cell, as ratatui's geometry type, for `Rect::contains`.
    pub fn position(&self) -> crate::Position {
        crate::Position::new(self.column, self.row)
    }
}
