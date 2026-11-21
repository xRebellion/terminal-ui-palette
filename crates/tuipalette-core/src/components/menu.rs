//! The list: rows, a selection that clamps at both ends, a window that keeps the
//! selection on screen, an icon column when the pane asks for one, an optional
//! scrollbar, and the matched characters of a query drawn differently from the
//! rest.
//!
//! The icon column is reserved here and filled by whoever supplies the rows —
//! the library never looks a glyph up (docs/PLAN.md, D11). The scrollbar is
//! drawn here too, from the same numbers the window is derived from, so the
//! pane still holds nothing but its rows and an index (docs/PLAN.md, 22).

use crate::action::Action;
use crate::component::{Component, Paint, Reaction};
use crate::spec::{Pane, ScrollbarStyle, Theme};
use crate::ui;
use crate::{Buffer, Modifier, Rect, Style};

/// One row: what is shown, what comes back, the glyph in front of it, and
/// text that is searchable but never drawn.
///
/// `search` is how a provider lends the matcher words the row should be
/// found by but not shown with — rofi's drun matches Name, GenericName,
/// Exec, Categories and Keywords while showing only the Name, and this is
/// the same split (docs/spec-reference.md, item protocol).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    pub display: String,
    pub value: String,
    pub icon: Option<String>,
    pub search: Option<String>,
    /// Shown by a `Detail` pane while this row is selected, never in the list.
    pub detail: Option<String>,
}

impl Item {
    /// A row that is its own value — what the document's `items` list means.
    pub fn line(text: impl Into<String>) -> Self {
        let text = text.into();
        Self {
            display: text.clone(),
            value: text,
            icon: None,
            search: None,
            detail: None,
        }
    }
}

/// A row that survived the matcher, with the characters to emphasise.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub item: Item,
    /// Character indices into `item.display`, ascending.
    pub positions: Vec<usize>,
}

impl Row {
    pub fn plain(item: Item) -> Self {
        Self {
            item,
            positions: Vec::new(),
        }
    }
}

/// The `Glyph` style's two glyphs: a filled thumb and the track it runs in.
/// Backgrounds are the default because that is what a terminal scrollbar looks
/// like (tmux, helix), but those cells are invisible to anything that reads the
/// screen back as text, so this style stays available for one.
const THUMB: &str = "\u{2588}";
const TRACK: &str = "\u{2591}";

/// A scrollbar needs a column to sit in and at least a couple of cells left
/// for the text itself.
const MIN_SCROLLBAR_WIDTH: u16 = 4;

pub struct Menu {
    id: String,
    title: String,
    rows: Vec<Row>,
    selected: usize,
    /// Width of the icon column in cells, if the pane declares one.
    icon_width: Option<u16>,
    frame: bool,
    /// Draw the thumb in the last column, and take one cell from the text.
    scrollbar: bool,
    /// How that bar is drawn when it is drawn at all — the geometry is the
    /// same either way, so this only decides what the cells hold.
    scrollbar_style: ScrollbarStyle,
}

impl Menu {
    pub fn new(pane: &Pane) -> Self {
        Self {
            id: pane.id.clone(),
            title: pane.title.clone(),
            rows: pane.items.iter().map(|text| Row::plain(Item::line(text))).collect(),
            selected: 0,
            icon_width: pane.icon_width.map(u16::from),
            frame: pane.frame,
            scrollbar: pane.scrollbar,
            scrollbar_style: pane.scrollbar_style(),
        }
    }

    /// Replace the rows, keeping the selection inside them. This is the hook the
    /// picker holds: each keystroke recomputes the rows.
    pub fn set_rows(&mut self, rows: Vec<Row>) {
        self.rows = rows;
        self.clamp();
    }

    pub fn set_items(&mut self, items: Vec<Item>) {
        self.set_rows(items.into_iter().map(Row::plain).collect());
    }

    pub fn rows(&self) -> &[Row] {
        &self.rows
    }

    pub fn selected(&self) -> Option<&Item> {
        self.rows.get(self.selected).map(|row| &row.item)
    }

    fn clamp(&mut self) {
        self.selected = self.selected.min(self.rows.len().saturating_sub(1));
    }

    /// First visible row for a window of `rows`: the selection never leaves it,
    /// and nothing has to be stored to keep it there.
    fn window(&self, rows: usize) -> usize {
        if rows == 0 || self.selected < rows {
            0
        } else {
            self.selected + 1 - rows
        }
    }

    /// Where the thumb goes in a track of `track` cells — `(top, length)` — or
    /// `None` when everything fits and there is nowhere to be.
    ///
    /// Both numbers are proportional to the list, so 12 rows in a 200-row list
    /// is a one-cell bar, and twelve of those in a 200-row list is a bar that
    /// covers a twelfth of the track. The position comes from the same window
    /// the rows are drawn from, which is what makes the bar agree with the
    /// screen instead of reporting a scroll offset the pane does not have.
    fn thumb(&self, track: usize) -> Option<(usize, usize)> {
        let total = self.rows.len();
        if track == 0 || total <= track {
            return None;
        }
        // The share of the track the window covers, rounded, never below one
        // cell: a bar that vanished would say "everything fits" while it does not.
        let length = ((track * track + total / 2) / total).clamp(1, track);
        let travel = track - length;
        let span = total - track;
        let top = (travel * self.window(track) + span / 2) / span;
        Some((top, length))
    }

    /// Paint the bar once its geometry is decided.
    ///
    /// Both styles write the same cells — the pane's last column, with
    /// `length` of the track's `rows` lit — and differ only in what goes into
    /// them: a glyph, or a tinted background under a cell nobody writes a glyph
    /// into. The tinted style paints over the selection's background, which is
    /// what keeps the bar visible on the row the selection is on.
    fn draw_bar(
        &self,
        buf: &mut Buffer,
        inner: Rect,
        rows: usize,
        top: usize,
        length: usize,
        theme: &Theme,
    ) {
        let x = inner.right() - 1;
        match self.scrollbar_style {
            ScrollbarStyle::Background => {
                // `set_style`, not a glyph: the cell keeps its blank symbol and
                // only its background changes. That is the one thing a text
                // snapshot cannot show and a cell-style assertion can.
                buf.set_style(
                    Rect::new(x, inner.y, 1, rows as u16),
                    Style::default().bg(theme.scrollbar_bg_color()),
                );
                buf.set_style(
                    Rect::new(x, inner.y + top as u16, 1, length as u16),
                    Style::default().bg(theme.scrollbar_color()),
                );
            }
            ScrollbarStyle::Glyph => {
                let track = Style::default().fg(theme.scrollbar_bg_color());
                let thumb = Style::default().fg(theme.scrollbar_color());
                for row in 0..rows as u16 {
                    ui::put(buf, x, inner.y + row, 1, TRACK, track);
                }
                for row in 0..length as u16 {
                    ui::put(buf, x, inner.y + top as u16 + row, 1, THUMB, thumb);
                }
            }
        }
    }
}

impl Component for Menu {
    fn id(&self) -> &str {
        &self.id
    }

    fn render(&self, paint: &Paint<'_>, buf: &mut Buffer) {
        let inner = if self.frame {
            ui::frame(buf, paint.area, &self.title, paint.focused, paint.theme)
        } else {
            paint.area
        };
        if inner.height == 0 || inner.width == 0 {
            return;
        }
        let rows = inner.height as usize;
        let start = self.window(rows);
        // The bar is the pane's last column, so the text is one cell narrower.
        let bar = if self.scrollbar && inner.width >= MIN_SCROLLBAR_WIDTH {
            self.thumb(rows)
        } else {
            None
        };
        let text_width = inner.width - u16::from(bar.is_some());
        for (row, data) in self.rows.iter().skip(start).take(rows).enumerate() {
            let selected = start + row == self.selected;
            let y = inner.y + row as u16;
            if selected {
                buf.set_style(
                    Rect::new(inner.x, y, inner.width, 1),
                    Style::default().bg(paint.theme.selected_color()),
                );
            }
            let plain = Style::default().fg(if selected {
                paint.theme.accent_color()
            } else {
                paint.theme.text_color()
            });
            let marked = plain.add_modifier(Modifier::BOLD);

            let mut x = inner.x;
            let mut room = text_width;
            if let Some(width) = self.icon_width {
                // An item without a glyph still gets a slot: the theme's
                // placeholder keeps the text column where the other rows put it.
                let glyph = data.item.icon.as_deref().filter(|icon| !icon.is_empty());
                let mut slot = ui::fit(glyph.unwrap_or(&paint.theme.icon_placeholder), width);
                if glyph.is_some() && slot.trim().is_empty() {
                    // Too wide for the slot: a half glyph would be worse than none.
                    slot = ui::fit(&paint.theme.icon_placeholder, width);
                }
                ui::put(buf, x, y, width, &slot, plain);
                x += width + 1;
                room = room.saturating_sub(width + 1);
            }
            let marker = if selected { "› " } else { "  " };
            ui::put(buf, x, y, room, marker, plain);
            x += ui::width(marker);
            room = room.saturating_sub(ui::width(marker));
            write_display(buf, x, y, room, data, marked, plain);
        }
        if let Some((top, length)) = bar {
            self.draw_bar(buf, inner, rows, top, length, paint.theme);
        }
    }

    fn value(&self) -> Option<String> {
        self.selected().map(|item| item.value.clone())
    }

    fn update(&mut self, action: &Action) -> Reaction {
        match action {
            Action::Down => {
                if self.selected + 1 < self.rows.len() {
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

/// Write the display string, switching to the emphasised style on the characters
/// the matcher reported. Positions are ascending, so one pass with a run buffer
/// is enough.
fn write_display(
    buf: &mut Buffer,
    mut x: u16,
    y: u16,
    room: u16,
    row: &Row,
    marked: Style,
    plain: Style,
) {
    let mut left = room;
    let mut run = String::new();
    let mut run_hit = false;
    let mut started = false;

    for (index, ch) in row.item.display.chars().enumerate() {
        if left == 0 {
            break;
        }
        let hit = row.positions.binary_search(&index).is_ok();
        if started && hit != run_hit {
            x = flush(buf, x, y, &mut left, &mut run, run_hit, marked, plain);
        }
        run.push(ch);
        run_hit = hit;
        started = true;
    }
    if started {
        flush(buf, x, y, &mut left, &mut run, run_hit, marked, plain);
    }
}

fn flush(
    buf: &mut Buffer,
    x: u16,
    y: u16,
    left: &mut u16,
    run: &mut String,
    hit: bool,
    marked: Style,
    plain: Style,
) -> u16 {
    ui::put(buf, x, y, *left, run, if hit { marked } else { plain });
    let used = ui::width(run);
    *left = left.saturating_sub(used);
    run.clear();
    x.saturating_add(used)
}
