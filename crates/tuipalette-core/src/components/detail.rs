//! The selected row's `detail` text in a frame — a preview that follows the
//! menu's selection.
//!
//! The pane holds no reference to the menu: the picker copies the selected
//! item's detail into it whenever the selection or the rows change, which keeps
//! components ignorant of each other (docs/PLAN.md, D5).

use crate::component::{Component, Paint};
use crate::spec::Pane;
use crate::ui;
use crate::{Buffer, Style};

pub struct DetailPane {
    id: String,
    title: String,
    /// The menu this preview follows, when the document names one.
    from: Option<String>,
    /// What to show; `None` until the picker hands the first selection over.
    text: Option<String>,
    frame: bool,
}

impl DetailPane {
    pub fn new(pane: &Pane) -> Self {
        Self {
            id: pane.id.clone(),
            title: pane.title.clone(),
            from: pane.from.clone(),
            text: None,
            frame: pane.frame,
        }
    }

    /// The menu this pane follows, or `None` to mean "the first menu".
    pub fn from(&self) -> Option<&str> {
        self.from.as_deref()
    }

    /// Replace the preview. The text is data: it is drawn line by line, clipped
    /// to the pane, and nothing in it is interpreted.
    pub fn set_text(&mut self, text: Option<String>) {
        self.text = text;
    }

    pub fn text(&self) -> Option<&str> {
        self.text.as_deref()
    }
}

impl Component for DetailPane {
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
        if inner.width == 0 || inner.height == 0 {
            return;
        }
        let Some(text) = self.text.as_deref() else {
            return;
        };
        let style = Style::default().fg(paint.theme.text_color());
        for (row, line) in text.lines().take(inner.height as usize).enumerate() {
            ui::put(buf, inner.x, inner.y + row as u16, inner.width, line, style);
        }
    }
}
