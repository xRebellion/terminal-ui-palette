//! One row of hints: no frame, no focus, dim so it reads as furniture.

use crate::component::{Component, Paint};
use crate::spec::Pane;
use crate::ui;
use crate::{Buffer, Style};

pub struct StatusLine {
    id: String,
    hints: Vec<String>,
}

impl StatusLine {
    pub fn new(pane: &Pane) -> Self {
        Self {
            id: pane.id.clone(),
            hints: pane.items.clone(),
        }
    }
}

impl Component for StatusLine {
    fn id(&self) -> &str {
        &self.id
    }

    fn focusable(&self) -> bool {
        false
    }

    fn render(&self, paint: &Paint<'_>, buf: &mut Buffer) {
        if paint.area.height == 0 {
            return;
        }
        ui::put(
            buf,
            paint.area.x,
            paint.area.y,
            paint.area.width,
            &self.hints.join("  "),
            Style::default().fg(paint.theme.dim_color()),
        );
    }
}
