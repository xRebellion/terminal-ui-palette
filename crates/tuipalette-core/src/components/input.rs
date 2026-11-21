//! The query line: what the user types, and nothing else. Filtering is the
//! source's job (item 06), so this component only keeps a string.

use crate::action::Action;
use crate::component::{Component, Paint, Reaction};
use crate::spec::Pane;
use crate::ui;
use crate::{Buffer, Style};

pub struct InputLine {
    id: String,
    title: String,
    value: String,
    frame: bool,
}

impl InputLine {
    pub fn new(pane: &Pane) -> Self {
        Self {
            id: pane.id.clone(),
            title: pane.title.clone(),
            value: String::new(),
            frame: pane.frame,
        }
    }

    pub fn text(&self) -> &str {
        &self.value
    }
}

impl Component for InputLine {
    fn id(&self) -> &str {
        &self.id
    }

    fn render(&self, paint: &Paint<'_>, buf: &mut Buffer) {
        let inner = if self.frame {
            ui::frame(buf, paint.area, &self.title, paint.focused, paint.theme)
        } else {
            paint.area
        };
        if inner.height == 0 {
            return;
        }
        let cursor = if paint.focused { "▏" } else { "" };
        let style = Style::default().fg(paint.theme.text_color());
        let mut x = inner.x;
        let mut room = inner.width;
        // Without a frame there is nowhere to hang the title, so it becomes the
        // prompt in front of the query — fzf's `>`. A framed pane keeps it on
        // the frame instead.
        if !self.frame && !self.title.is_empty() {
            let prompt = self.title.clone();
            let accent = Style::default().fg(paint.theme.accent_color());
            ui::put(buf, x, inner.y, room, &prompt, accent);
            let used = ui::width(&prompt);
            x += used;
            room = room.saturating_sub(used);
        }
        let shown = ui::tail(&format!("{}{cursor}", self.value), room);
        ui::put(buf, x, inner.y, room, &shown, style);
    }

    fn update(&mut self, action: &Action) -> Reaction {
        match action {
            Action::Input(c) if !c.is_control() => {
                self.value.push(*c);
                Reaction::Handled
            }
            Action::Backspace | Action::Delete => {
                if self.value.pop().is_some() {
                    Reaction::Handled
                } else {
                    Reaction::Ignored
                }
            }
            _ => Reaction::Ignored,
        }
    }

    fn value(&self) -> Option<String> {
        Some(self.value.clone())
    }
}
