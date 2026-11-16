//! The drawing primitives the built-in components share.
//!
//! Three functions, because three is all a picker needs: a titled frame that
//! returns its inside, one clipped line of text, and the buffer read back as text
//! for snapshots.

use crate::spec::Theme;
use crate::{Buffer, Rect, Style};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

/// Draw a rounded frame with a title and return the area inside it.
///
/// A focused frame takes the accent colour, an unfocused one the theme's dim
/// token. When the area cannot hold a frame the returned rect is empty and
/// nothing is drawn — a component then has nothing to write into.
pub fn frame(buf: &mut Buffer, area: Rect, title: &str, focused: bool, theme: &Theme) -> Rect {
    if area.width < 2 || area.height < 2 {
        return Rect::new(area.x, area.y, 0, 0);
    }
    let style = Style::default().fg(if focused {
        theme.accent_color()
    } else {
        theme.dim_color()
    });
    let put = |buf: &mut Buffer, x: u16, y: u16, symbol: &str| {
        if let Some(cell) = buf.cell_mut((x, y)) {
            cell.set_symbol(symbol).set_style(style);
        }
    };
    let right = area.right() - 1;
    let bottom = area.bottom() - 1;
    for x in area.left() + 1..right {
        put(buf, x, area.top(), "─");
        put(buf, x, bottom, "─");
    }
    for y in area.top() + 1..bottom {
        put(buf, area.left(), y, "│");
        put(buf, right, y, "│");
    }
    put(buf, area.left(), area.top(), "╭");
    put(buf, right, area.top(), "╮");
    put(buf, area.left(), bottom, "╰");
    put(buf, right, bottom, "╯");

    let inner = Rect::new(area.left() + 1, area.top() + 1, area.width - 2, area.height - 2);
    if !title.is_empty() && inner.width > 4 {
        let room = (inner.width as usize).saturating_sub(2);
        let label: String = format!(" {title} ").chars().take(room).collect();
        buf.set_stringn(area.left() + 2, area.top(), &label, room, style);
    }
    inner
}

/// How many cells a string occupies. A wide (CJK, emoji) glyph counts two, which
/// is why nothing here measures a string with `chars().count()`.
pub fn width(text: &str) -> u16 {
    u16::try_from(text.width()).unwrap_or(u16::MAX)
}

/// Clip `text` to `width` cells and pad it with spaces to exactly that many.
///
/// Measures per **grapheme cluster**, not per `char`: an emoji built from a ZWJ
/// sequence is one cluster two cells wide, and cutting it between its parts would
/// leave half a glyph whose real width the terminal decides, not us.
pub fn fit(text: &str, width: u16) -> String {
    let mut out = String::new();
    let mut used = 0u16;
    for cluster in text.graphemes(true) {
        let w = u16::try_from(cluster.width()).unwrap_or(u16::MAX);
        if used + w > width {
            break;
        }
        used += w;
        out.push_str(cluster);
    }
    while used < width {
        out.push(' ');
        used += 1;
    }
    out
}

/// The tail of `text` that fits in `width` cells — a query is typed at its end,
/// so the end is the part worth showing.
pub fn tail(text: &str, width: u16) -> String {
    let total = self::width(text);
    if total <= width {
        return text.to_owned();
    }
    let mut drop = total - width;
    let mut out = String::new();
    for cluster in text.graphemes(true) {
        let w = u16::try_from(cluster.width()).unwrap_or(u16::MAX);
        if drop >= w {
            drop -= w;
            continue;
        }
        out.push_str(cluster);
    }
    out
}

/// One line of text, clipped to `width` cells and to the buffer's edge.
pub fn put(buf: &mut Buffer, x: u16, y: u16, width: u16, text: &str, style: Style) {
    if width == 0 {
        return;
    }
    buf.set_stringn(x, y, text, width as usize, style);
}

/// The buffer read back as text: one line per row, trailing blanks trimmed.
/// This is what the snapshot tests capture, so a visual change is a failing
/// test rather than something someone has to look at.
pub fn screen(buf: &Buffer) -> String {
    let area = *buf.area();
    let mut out = String::new();
    for y in area.top()..area.bottom() {
        let mut line = String::new();
        for x in area.left()..area.right() {
            line.push_str(buf.cell((x, y)).map(|cell| cell.symbol()).unwrap_or(" "));
        }
        out.push_str(line.trim_end());
        if y + 1 < area.bottom() {
            out.push('\n');
        }
    }
    out
}
