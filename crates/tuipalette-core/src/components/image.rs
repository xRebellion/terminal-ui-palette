//! A preview that shows the selected row's picture where a host can draw one,
//! and the row's `detail` text where it cannot.
//!
//! The pane never draws a picture itself: it paints its fallback text like a
//! `Detail` pane, then answers `Component::image` with the path and the
//! rect inside its frame. Whether that rect is then covered by an image is the
//! host's business (`tuipalette-tui`'s `Graphics` backends), which is what
//! keeps this crate free of terminal protocols (D1, D22,
//! docs/specs/25-image-pane.md).

use crate::component::{Component, ImageRequest, Paint};
use crate::components::DetailPane;
use crate::spec::Pane;
use crate::ui;
use crate::{Buffer, Rect};

pub struct ImagePane {
    /// The fallback: the same text, frame and wrapping a `Detail` pane has.
    text: DetailPane,
    frame: bool,
    image: Option<String>,
}

impl ImagePane {
    pub fn new(pane: &Pane) -> Self {
        Self {
            text: DetailPane::new(pane),
            frame: pane.frame,
            image: None,
        }
    }

    /// The menu this pane follows, or `None` to mean "the first menu".
    pub fn from(&self) -> Option<&str> {
        self.text.from()
    }

    /// The selected row's picture and its text, both replaced together so a
    /// row without a picture never shows the previous row's.
    pub fn set(&mut self, image: Option<String>, text: Option<String>) {
        self.image = image;
        self.text.set_text(text);
    }

    pub fn image_path(&self) -> Option<&str> {
        self.image.as_deref()
    }

    pub fn text(&self) -> Option<&str> {
        self.text.text()
    }
}

impl Component for ImagePane {
    fn id(&self) -> &str {
        self.text.id()
    }

    fn focusable(&self) -> bool {
        false
    }

    fn render(&self, paint: &Paint<'_>, buf: &mut Buffer) {
        self.text.render(paint, buf);
    }

    fn image(&self, area: Rect) -> Option<ImageRequest> {
        let path = self.image.clone()?;
        let rect = if self.frame { ui::inner(area) } else { area };
        (rect.width > 0 && rect.height > 0).then_some(ImageRequest { rect, path })
    }
}
