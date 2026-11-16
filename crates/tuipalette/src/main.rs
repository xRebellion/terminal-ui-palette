//! The binary. Item 07 owns the CLI contract; for now this exists so the three
//! crates are linked, built and tested as one workspace.

use tuipalette_core::Rect;

fn main() {
    let area = Rect::new(0, 0, 80, 24);
    eprintln!(
        "tuipalette {} (core {} + tui {}) — scaffold only, see docs/PLAN.md",
        env!("CARGO_PKG_VERSION"),
        tuipalette_core::VERSION,
        tuipalette_tui::VERSION
    );
    eprintln!("root area would be {area:?}");
}
