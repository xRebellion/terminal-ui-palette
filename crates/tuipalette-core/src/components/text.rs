//! Static lines in a frame — a preview pane, a help page, a log tail.

use crate::component::{Component, Paint};
use crate::spec::Pane;
use crate::ui;
use crate::{Buffer, Style};

pub struct TextPane {
    id: String,
    title: String,
    lines: Vec<String>,
    frame: bool,
}

impl TextPane {
    pub fn new(pane: &Pane) -> Self {
        Self {
            id: pane.id.clone(),
            title: pane.title.clone(),
            lines: pane.items.clone(),
            frame: pane.frame,
        }
    }
}

impl Component for TextPane {
    fn id(&self) -> &str {
        &self.id
    }

    fn focusable(&self) -> bool {
        false
    }

    fn render(&self, paint: &Paint<'_>, buf: &mut Buffer) {
        let inner = if self.frame {
            ui::frame(buf, paint.area, &self.title, paint.focused, paint.theme)
        } else {
            paint.area
        };
        let style = Style::default().fg(paint.theme.text_color());
        for (row, line) in self.lines.iter().take(inner.height as usize).enumerate() {
            ui::put(buf, inner.x, inner.y + row as u16, inner.width, line, style);
        }
    }
}
