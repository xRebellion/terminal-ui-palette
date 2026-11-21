//! Pictures over the cell grid, behind one trait so the protocol can change.
//!
//! The core crate never draws an image: a pane answers `Component::image`
//! with a path and a rect, and the draw loop hands that list to a `Graphics`
//! backend after the cells are drawn. A backend decides how the picture
//! reaches the screen — kitty's Unicode placeholders today; sixel or iTerm2
//! would be another impl of the same trait — and `None` (no backend) leaves
//! every pane's text fallback showing. Adding a protocol is one new file
//! here and one arm in `detect`; no pane and no document changes (D22,
//! docs/specs/25-image-pane.md).

use std::io::{self, Write};

use ratatui::buffer::Buffer;
use tuipalette_core::ImageRequest;

pub mod kitty;

/// One way of putting pictures on a terminal.
///
/// The loop calls `place` once per drawn frame, with the frame's buffer
/// *before* it is flushed, so a backend that works through the cell grid
/// (kitty placeholders) can write its cells into it, and one that works
/// beside it can write escapes to `out`. `clear` runs on every way out.
pub trait Graphics {
    /// Show exactly `images` this frame: anything placed earlier and not in
    /// the list must go. `out` is the terminal; `buf` is the frame about to
    /// be flushed.
    fn place(&mut self, images: &[ImageRequest], buf: &mut Buffer, out: &mut dyn Write) -> io::Result<()>;

    /// Forget every picture this backend put up. Called on exit, error and
    /// unwind; must be safe to call twice.
    fn clear(&mut self, out: &mut dyn Write) -> io::Result<()>;
}

/// Which backend a run uses. `Auto` asks the environment; `None` turns
/// pictures off, so every image pane shows its text.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Protocol {
    #[default]
    Auto,
    Kitty,
    None,
}

impl Protocol {
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "auto" => Some(Self::Auto),
            "kitty" => Some(Self::Kitty),
            "none" => Some(Self::None),
            _ => None,
        }
    }
}

/// The backend for `protocol`, or `None` when pictures are off or the
/// terminal is not known to draw them.
///
/// `Auto` reads the environment rather than querying the terminal: a query
/// would race the key reader for the reply on the same tty. `KITTY_WINDOW_ID`
/// is set by kitty for every child, and only by kitty; a wrong guess costs a
/// pane of placeholder glyphs, so the flag `--images none` is the way out.
pub fn detect(protocol: Protocol) -> Option<Box<dyn Graphics>> {
    let kitty = || Some(Box::new(kitty::Kitty::new()) as Box<dyn Graphics>);
    match protocol {
        Protocol::None => None,
        Protocol::Kitty => kitty(),
        Protocol::Auto => {
            let in_kitty = std::env::var_os("KITTY_WINDOW_ID").is_some_and(|id| !id.is_empty());
            // Inside tmux the escapes need wrapping that kitty::Kitty does not do.
            let in_tmux = std::env::var_os("TMUX").is_some();
            (in_kitty && !in_tmux).then(kitty).flatten()
        }
    }
}
