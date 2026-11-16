//! The list: items, a selection that clamps at both ends, a window that keeps
//! the selection on screen, and an icon column when the pane asks for one.
//!
//! The icon column is reserved here and filled by whoever supplies the items —
//! the library never looks a glyph up (docs/PLAN.md, D11).

use crate::action::Action;
use crate::component::{Component, Paint, Reaction};
use crate::spec::Pane;
use crate::ui;
use crate::{Buffer, Modifier, Rect, Style};

/// One row: what is shown, what comes back, and the glyph in front of it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    pub display: String,
    pub value: String,
    pub icon: Option<String>,
}

impl Item {
    /// A row that is its own value — what the document's `items` list means.
    pub fn line(text: impl Into<String>) -> Self {
        let text = text.into();
        Self {
            display: text.clone(),
            value: text,
            icon: None,
        }
    }
}

pub struct Menu {
    id: String,
    title: String,
    items: Vec<Item>,
    selected: usize,
    /// Width of the icon column, if the pane declares one.
    icon_width: Option<u16>,
}

impl Menu {
    pub fn new(pane: &Pane) -> Self {
        Self {
            id: pane.id.clone(),
            title: pane.title.clone(),
            items: pane.items.iter().map(Item::line).collect(),
            selected: 0,
            icon_width: pane.icon_width.map(u16::from),
        }
    }

    /// Replace the rows, keeping the selection inside them — the hook a
    /// streaming source will hold (item 06).
    pub fn set_items(&mut self, items: Vec<Item>) {
        self.items = items;
        self.clamp();
    }

    pub fn items(&self) -> &[Item] {
        &self.items
    }

    pub fn selected(&self) -> Option<&Item> {
        self.items.get(self.selected)
    }

    fn clamp(&mut self) {
        self.selected = self.selected.min(self.items.len().saturating_sub(1));
    }

    /// First visible row for a window of `rows`: the selection never leaves it.
    fn window(&self, rows: usize) -> usize {
        if rows == 0 || self.selected < rows {
            0
        } else {
            self.selected + 1 - rows
        }
    }
}

impl Component for Menu {
    fn id(&self) -> &str {
        &self.id
    }

    fn render(&self, paint: &Paint<'_>, buf: &mut Buffer) {
        let inner = ui::frame(buf, paint.area, &self.title, paint.focused, paint.theme);
        if inner.height == 0 || inner.width == 0 {
            return;
        }
        let rows = inner.height as usize;
        let start = self.window(rows);
        for (row, item) in self.items.iter().skip(start).take(rows).enumerate() {
            let index = start + row;
            let selected = index == self.selected;
            let accent = Style::default().fg(paint.theme.accent_color());
            let plain = Style::default().fg(paint.theme.text_color());
            let y = inner.y + row as u16;
            if selected {
                buf.set_style(
                    Rect::new(inner.x, y, inner.width, 1),
                    Style::default().bg(paint.theme.selected_color()),
                );
            }
            let mut x = inner.x;
            let mut room = inner.width;
            if let Some(width) = self.icon_width {
                // The glyph only, clipped to the column: padding it by
                // characters would push a wide glyph past the column and shift
                // the whole row.
                ui::put(
                    buf,
                    x,
                    y,
                    width,
                    item.icon.as_deref().unwrap_or(""),
                    if selected { accent } else { plain },
                );
                x = x.saturating_add(width + 1);
                room = room.saturating_sub(width + 1);
            }
            let marker = if selected { "› " } else { "  " };
            let style = if selected {
                accent.add_modifier(Modifier::BOLD)
            } else {
                plain
            };
            ui::put(buf, x, y, room, &format!("{marker}{}", item.display), style);
        }
    }

    fn value(&self) -> Option<String> {
        self.selected().map(|item| item.value.clone())
    }

    fn update(&mut self, action: &Action) -> Reaction {
        match action {
            Action::Down => {
                if self.selected + 1 < self.items.len() {
                    self.selected += 1;
                }
                Reaction::Handled
            }
            Action::Up => {
                self.selected = self.selected.saturating_sub(1);
                Reaction::Handled
            }
            Action::Accept => match self.selected() {
                Some(item) => Reaction::Emit(Action::Invoke(item.value.clone())),
                None => Reaction::Handled,
            },
            _ => Reaction::Ignored,
        }
    }
}
