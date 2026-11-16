//! Cells with nothing in them, so a layout can push two panes apart.

use crate::component::{Component, Paint};
use crate::spec::Pane;
use crate::Buffer;

pub struct Spacer {
    id: String,
}

impl Spacer {
    pub fn new(pane: &Pane) -> Self {
        Self {
            id: pane.id.clone(),
        }
    }
}

impl Component for Spacer {
    fn id(&self) -> &str {
        &self.id
    }

    fn focusable(&self) -> bool {
        false
    }

    fn render(&self, _paint: &Paint<'_>, _buf: &mut Buffer) {}
}
