//! `shown/total`: how many rows survived the query, out of how many the
//! provider sent.
//!
//! The pane holds two numbers and a way to write them; the picker is the only
//! thing that knows both, so it fills the numbers in after every refilter — the
//! same arrangement as `Detail`, where the picker owns the value a pane shows
//! (docs/PLAN.md, 23).

use crate::component::{Component, Paint};
use crate::spec::{Align, Pane};
use crate::ui;
use crate::{Buffer, Style};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// What a counter shows when the document does not say: fzf's info line.
const DEFAULT_FORMAT: &str = "{shown}/{total}";

pub struct CounterPane {
    id: String,
    title: String,
    /// `{shown}` and `{total}` are substituted; nothing else is interpreted.
    format: String,
    align: Align,
    /// Rows the menu is showing, and rows that arrived.
    shown: usize,
    total: usize,
    frame: bool,
}

impl CounterPane {
    pub fn new(pane: &Pane) -> Self {
        Self {
            id: pane.id.clone(),
            title: pane.title.clone(),
            format: pane.format.clone().unwrap_or_else(|| DEFAULT_FORMAT.to_owned()),
            align: pane.align.unwrap_or_default(),
            shown: 0,
            total: 0,
            frame: pane.frame,
        }
    }

    /// Replace both numbers. The picker calls this after every refilter, so the
    /// count is never one keystroke or one streamed row behind the list.
    pub fn set_counts(&mut self, shown: usize, total: usize) {
        self.shown = shown;
        self.total = total;
    }

    pub fn counts(&self) -> (usize, usize) {
        (self.shown, self.total)
    }

    /// The line this pane draws right now.
    pub fn text(&self) -> String {
        self.format
            .replace("{shown}", &self.shown.to_string())
            .replace("{total}", &self.total.to_string())
    }
}

/// `text` cut to `width` cells by dropping characters from the *left*, with
/// `…` marking the cut. The total is the right-hand number in every format
/// worth writing, and a count that loses its total's last digits (`250/25`)
/// is a wrong number rather than a short one.
fn fit_from_the_right(text: &str, width: u16) -> String {
    let width = usize::from(width);
    if text.width() <= width {
        return text.to_owned();
    }
    if width == 0 {
        return String::new();
    }
    let mut kept = Vec::new();
    let mut used = 1; // the ellipsis
    for c in text.chars().rev() {
        let w = c.width().unwrap_or(0);
        if used + w > width {
            break;
        }
        used += w;
        kept.push(c);
    }
    std::iter::once('\u{2026}').chain(kept.into_iter().rev()).collect()
}

impl Component for CounterPane {
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
        let text = fit_from_the_right(&self.text(), inner.width);
        let width = ui::width(&text);
        let x = match self.align {
            Align::Left => inner.x,
            Align::Center => inner.x + (inner.width - width) / 2,
            Align::Right => inner.right() - width,
        };
        let style = Style::default().fg(paint.theme.text_color());
        ui::put(buf, x, inner.y, width, &text, style);
    }
}
