//! A divider: one line across its rect, in the theme's `dim` colour.
//!
//! Horizontal when the rect is wider than it is tall, vertical otherwise, and
//! centred on the other axis — so a three-cell-wide column gives a vertical line
//! with a blank cell on each side, which is how a document spaces two panes
//! apart without a frame around either.

use crate::component::{Component, Paint};
use crate::spec::Pane;
use crate::{Buffer, Style};

pub struct Rule {
    id: String,
}

impl Rule {
    pub fn new(pane: &Pane) -> Self {
        Self { id: pane.id.clone() }
    }
}

impl Component for Rule {
    fn id(&self) -> &str {
        &self.id
    }

    fn focusable(&self) -> bool {
        false
    }

    fn render(&self, paint: &Paint<'_>, buf: &mut Buffer) {
        let area = paint.area;
        if area.width == 0 || area.height == 0 {
            return;
        }
        let style = Style::default().fg(paint.theme.dim_color());
        let mut put = |x: u16, y: u16, symbol: &str| {
            if let Some(cell) = buf.cell_mut((x, y)) {
                cell.set_symbol(symbol).set_style(style);
            }
        };
        if area.height > area.width {
            let x = area.x + area.width / 2;
            for y in area.top()..area.bottom() {
                put(x, y, "\u{2502}");
            }
        } else {
            let y = area.y + area.height / 2;
            for x in area.left()..area.right() {
                put(x, y, "\u{2500}");
            }
        }
    }
}
