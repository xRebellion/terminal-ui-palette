//! Where items come from: lines on stdin, read as they arrive.
//!
//! The protocol is one line per item, up to five TAB-separated fields —
//! `display`, `value`, `icon`, `search`, `detail` — with trailing fields
//! optional. A line with no tab is its own value, which is what makes
//! `printf 'a\nb\n' | tuipalette` a dmenu (docs/PLAN.md, D12). The fourth
//! field is matched but never drawn: rofi's drun finds "Passwords and Keys"
//! by its `seahorse` executable, and this is where such words belong. The
//! fifth is shown by a `Detail` pane while the row is selected, and it is
//! one line here — a multi-line preview needs `--in json`, where the
//! newline is escaped instead of ending the item (D16).

use std::io::{BufRead, BufReader};
use std::sync::mpsc::{self, Receiver};

use crate::components::Item;

/// One line of the protocol, or `None` for a line that carries nothing.
///
/// The display is sanitised — a control character becomes a space, because a
/// menu row is one row — while the value is kept byte for byte. The icon takes
/// the third field (D15), the search text the fourth, and the detail text the
/// fifth; a field left empty is an absent one.
pub fn parse_line(line: &str) -> Option<Item> {
    let line = line.strip_suffix('\r').unwrap_or(line);
    if line.is_empty() {
        return None;
    }
    let mut fields = line.splitn(5, '\t');
    let display = sanitise(fields.next().unwrap_or_default());
    let value = fields.next();
    let icon = fields.next();
    let search = fields.next();
    let detail = fields.next();
    let display = match (display.is_empty(), value) {
        (true, Some(value)) if !value.is_empty() => sanitise(value),
        _ => display,
    };
    let value = match value {
        Some(value) => value.to_owned(),
        None => display.clone(),
    };
    if display.is_empty() && value.is_empty() {
        return None;
    }
    let icon = icon.filter(|icon| !icon.is_empty()).map(sanitise);
    let search = search.filter(|search| !search.is_empty()).map(sanitise);
    let detail = detail.filter(|detail| !detail.is_empty()).map(sanitise);
    Some(Item {
        display,
        value,
        icon,
        search,
        detail,
    })
}

fn sanitise(text: &str) -> String {
    text.chars()
        .map(|ch| if ch.is_control() { ' ' } else { ch })
        .collect()
}

/// Read `reader` on a thread and hand items over as they arrive — the picker
/// keeps drawing while a slow provider is still producing.
pub fn stream<R: BufRead + Send + 'static>(reader: R) -> Receiver<Item> {
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        for line in reader.lines() {
            let Ok(line) = line else { break };
            let Some(item) = parse_line(&line) else {
                continue;
            };
            if sender.send(item).is_err() {
                break;
            }
        }
    });
    receiver
}

/// Lines from this process's stdin.
pub fn stdin() -> Receiver<Item> {
    stream(BufReader::new(std::io::stdin()))
}

/// One line of `--in json`: either a bare string or an object with any of
/// `display`, `value`, `icon`, `search` and `detail` (docs/PLAN.md, D16 — the
/// opt-in path for values that a TAB-separated line cannot carry, newlines
/// included, which is what a multi-line detail needs).
pub fn parse_json_line(line: &str) -> Option<Item> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }
    if let Ok(text) = serde_json::from_str::<String>(line) {
        return Some(Item::line(text));
    }
    let object: JsonItem = serde_json::from_str(line).ok()?;
    let display = object.display.or_else(|| object.value.clone())?;
    let value = object.value.unwrap_or_else(|| display.clone());
    Some(Item {
        display,
        value,
        icon: object.icon.filter(|icon| !icon.is_empty()),
        search: object.search.filter(|search| !search.is_empty()),
        detail: object.detail.filter(|detail| !detail.is_empty()),
    })
}

#[derive(serde::Deserialize)]
struct JsonItem {
    display: Option<String>,
    value: Option<String>,
    icon: Option<String>,
    search: Option<String>,
    detail: Option<String>,
}

/// `stream` for the JSON protocol.
pub fn stream_json<R: BufRead + Send + 'static>(reader: R) -> Receiver<Item> {
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        for line in reader.lines() {
            let Ok(line) = line else { break };
            let Some(item) = parse_json_line(&line) else {
                continue;
            };
            if sender.send(item).is_err() {
                break;
            }
        }
    });
    receiver
}
