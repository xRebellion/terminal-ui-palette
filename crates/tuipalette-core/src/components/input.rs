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
}

impl InputLine {
    pub fn new(pane: &Pane) -> Self {
        Self {
            id: pane.id.clone(),
            title: pane.title.clone(),
            value: String::new(),
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
        let inner = ui::frame(buf, paint.area, &self.title, paint.focused, paint.theme);
        if inner.height == 0 {
            return;
        }
        let cursor = if paint.focused { "▏" } else { "" };
        let shown = ui::tail(&format!("{}{cursor}", self.value), inner.width);
        ui::put(
            buf,
            inner.x,
            inner.y,
            inner.width,
            &shown,
            Style::default().fg(paint.theme.text_color()),
        );
    }

    fn update(&mut self, action: &Action) -> Reaction {
        match action {
            Action::Input(c) if !c.is_control() => {
                self.value.push(*c);
                Reaction::Handled
            }
            Action::Backspace => {
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
