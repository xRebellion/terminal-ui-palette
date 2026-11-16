//! What a component is here: a box that draws itself and reacts to an action.
//! It owns its state, knows nothing about focus policy or key handling, and
//! answers with a `Reaction` instead of mutating anything global.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph, Widget};

use crate::action::Action;
use crate::spec::{Kind, Pane, Theme};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reaction {
    Ignored,
    Handled,
    Emit(Action),
}

pub trait Component {
    fn id(&self) -> &str;
    fn focusable(&self) -> bool {
        true
    }
    fn render(&self, area: Rect, buf: &mut Buffer, focused: bool, theme: &Theme);
    fn update(&mut self, _action: &Action) -> Reaction {
        Reaction::Ignored
    }
    fn value(&self) -> Option<String> {
        None
    }
}

pub fn from_pane(pane: &Pane) -> Box<dyn Component> {
    match pane.kind {
        Kind::Input => Box::new(InputLine::new(pane)),
        Kind::Menu => Box::new(Menu::new(pane)),
        Kind::Text => Box::new(TextPane::new(pane)),
    }
}

pub fn frame_block(title: &str, focused: bool, theme: &Theme) -> Block<'static> {
    let style = if focused {
        Style::default().fg(theme.accent_color())
    } else {
        Style::default().fg(Color::DarkGray)
    };
    Block::bordered()
        .title(Line::from(format!(" {title} ")))
        .border_style(style)
}

pub struct TextPane {
    id: String,
    title: String,
    lines: Vec<String>,
}

impl TextPane {
    pub fn new(pane: &Pane) -> Self {
        Self {
            id: pane.id.clone(),
            title: pane.title.clone(),
            lines: pane.items.clone(),
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
    fn render(&self, area: Rect, buf: &mut Buffer, focused: bool, theme: &Theme) {
        let text: Vec<Line> = self.lines.iter().map(|l| Line::from(l.as_str())).collect();
        Paragraph::new(text)
            .block(frame_block(&self.title, focused, theme))
            .render(area, buf);
    }
}

pub struct InputLine {
    id: String,
    title: String,
    pub value: String,
}

impl InputLine {
    pub fn new(pane: &Pane) -> Self {
        Self {
            id: pane.id.clone(),
            title: pane.title.clone(),
            value: String::new(),
        }
    }
}

impl Component for InputLine {
    fn id(&self) -> &str {
        &self.id
    }
    fn render(&self, area: Rect, buf: &mut Buffer, focused: bool, theme: &Theme) {
        let block = frame_block(&self.title, focused, theme);
        let inner = block.inner(area);
        block.render(area, buf);
        let cursor = if focused { "▏" } else { "" };
        Paragraph::new(Line::from(format!("{}{}", self.value, cursor)))
            .style(Style::default().fg(theme.text_color()))
            .render(inner, buf);
    }
    fn update(&mut self, action: &Action) -> Reaction {
        match action {
            Action::Input(c) => {
                self.value.push(*c);
                Reaction::Handled
            }
            Action::Backspace => {
                self.value.pop();
                Reaction::Handled
            }
            _ => Reaction::Ignored,
        }
    }
    fn value(&self) -> Option<String> {
        Some(self.value.clone())
    }
}

pub struct Menu {
    id: String,
    title: String,
    items: Vec<String>,
    pub selected: usize,
}

impl Menu {
    pub fn new(pane: &Pane) -> Self {
        Self {
            id: pane.id.clone(),
            title: pane.title.clone(),
            items: pane.items.clone(),
            selected: 0,
        }
    }
}

impl Component for Menu {
    fn id(&self) -> &str {
        &self.id
    }
    fn render(&self, area: Rect, buf: &mut Buffer, focused: bool, theme: &Theme) {
        let block = frame_block(&self.title, focused, theme);
        let inner = block.inner(area);
        block.render(area, buf);

        let rows = inner.height as usize;
        let width = inner.width as usize;
        if rows == 0 || width == 0 || self.items.is_empty() {
            return;
        }
        let start = (self.selected + 1).saturating_sub(rows);
        for (row, (idx, item)) in self
            .items
            .iter()
            .enumerate()
            .skip(start)
            .take(rows)
            .enumerate()
        {
            let selected = idx == self.selected;
            let marker = if selected { "› " } else { "  " };
            let mut line = format!("{marker}{item}");
            if line.chars().count() > width {
                line = line.chars().take(width).collect();
            }
            let padded = format!("{line:<width$}");
            let style = if selected && focused {
                Style::default()
                    .bg(theme.selected_color())
                    .fg(theme.accent_color())
                    .add_modifier(Modifier::BOLD)
            } else if selected {
                Style::default().fg(theme.accent_color())
            } else {
                Style::default().fg(theme.text_color())
            };
            buf.set_string(inner.x, inner.y + row as u16, padded, style);
        }
    }
    fn update(&mut self, action: &Action) -> Reaction {
        match action {
            Action::Up => {
                self.selected = self.selected.saturating_sub(1);
                Reaction::Handled
            }
            Action::Down => {
                if self.selected + 1 < self.items.len() {
                    self.selected += 1;
                }
                Reaction::Handled
            }
            Action::Accept => match self.items.get(self.selected) {
                Some(item) => Reaction::Emit(Action::Invoke(format!("run:{item}"))),
                None => Reaction::Handled,
            },
            _ => Reaction::Ignored,
        }
    }
    fn value(&self) -> Option<String> {
        self.items.get(self.selected).cloned()
    }
}
