//! Item 06's acceptance: the line protocol, a source that delivers before EOF,
//! and the picker that ranks and highlights what survives the query.

use std::io::Read;
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

use tuipalette_core::matcher::SimpleMatcher;
use tuipalette_core::source::{parse_line, stream};
use tuipalette_core::spec::Format;
use tuipalette_core::{
    ui, Action, Buffer, Item, Key, KeyCode, Mods, Picker, Rect, Row, UiSpec,
};

// --- the line protocol (D12, D15) -------------------------------------------

#[test]
fn a_line_carries_display_value_and_icon() {
    let plain = parse_line("firefox").expect("a line");
    assert_eq!(plain.display, "firefox");
    assert_eq!(plain.value, "firefox", "no tab means the line is its own value");
    assert_eq!(plain.icon, None);

    let three = parse_line("Firefox\tfirefox.desktop\t🌐").expect("a line");
    assert_eq!(three.display, "Firefox");
    assert_eq!(three.value, "firefox.desktop");
    assert_eq!(three.icon.as_deref(), Some("🌐"));

    let two = parse_line("Firefox\tfirefox.desktop").expect("a line");
    assert_eq!(two.icon, None);

    let remainder = parse_line("a\tb\tc\td").expect("a line");
    assert_eq!(remainder.value, "b", "the value is the second field only");
    // The icon is the rest of the line, sanitised like the display: it is one
    // cell slot, and a control character in it would break the column.
    assert_eq!(remainder.icon.as_deref(), Some("c d"), "the icon takes the rest");
}

#[test]
fn a_line_sanitises_the_display_and_keeps_the_value_exact() {
    let item = parse_line("a\u{7}b\tvalue\u{7}kept").expect("a line");
    assert_eq!(item.display, "a b", "a control character becomes a space");
    assert_eq!(item.value, "value\u{7}kept", "the value is byte-exact");
}

#[test]
fn an_empty_line_is_not_an_item() {
    assert!(parse_line("").is_none());
    assert!(parse_line("\r").is_none());
    assert_eq!(parse_line("text\r").map(|i| i.display), Some("text".to_owned()));
}

// --- streaming --------------------------------------------------------------

/// A reader that hands over whatever the test sends, one line at a time, and
/// only reports EOF when the test closes the channel.
struct Paced {
    lines: Receiver<Option<String>>,
    pending: Vec<u8>,
}

impl Read for Paced {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.pending.is_empty() {
            match self.lines.recv() {
                Ok(Some(line)) => self.pending.extend_from_slice(line.as_bytes()),
                // None, or the test dropped the sender: end of stream.
                _ => return Ok(0),
            }
        }
        let take = self.pending.len().min(buf.len());
        buf[..take].copy_from_slice(&self.pending[..take]);
        self.pending.drain(..take);
        Ok(take)
    }
}

#[test]
fn an_item_arrives_before_the_source_ends() {
    let (sender, lines) = mpsc::channel();
    let source = stream(std::io::BufReader::new(Paced {
        lines,
        pending: Vec::new(),
    }));

    sender.send(Some("first\n".to_owned())).expect("send");
    let item = source
        .recv_timeout(Duration::from_secs(2))
        .expect("an item before EOF");
    assert_eq!(item.display, "first");

    sender.send(Some("second\n".to_owned())).expect("send");
    assert_eq!(source.recv_timeout(Duration::from_secs(2)).map(|i| i.display), Ok("second".to_owned()));
}

// --- the picker -------------------------------------------------------------

const PICKER: &str = r##"(
    window: (title: "t"),
    layout: (dir: Vertical, sizes: ["3", "fill"], children: [(pane: "query"), (pane: "list")]),
    panes: [
        (id: "query", kind: Input, title: "search"),
        (id: "list", kind: Menu, title: "matches"),
    ],
)"##;

fn picker(text: &str) -> Picker {
    let spec = UiSpec::parse(text, Format::Ron).unwrap_or_else(|e| panic!("{e}"));
    let picker = Picker::new(spec).unwrap_or_else(|e| panic!("{e}"));
    picker.with_matcher(Box::new(SimpleMatcher::new()))
}

fn items(names: &[&str]) -> Vec<Item> {
    names.iter().map(|name| Item::line(*name)).collect()
}

#[test]
fn the_picker_narrows_as_the_query_is_typed() {
    let mut picker = picker(PICKER);
    picker.push(items(&["firefox", "kitty", "firefly", "discord"]));
    assert_eq!(picker.rows().len(), 4, "an empty query shows everything");

    for c in "fir".chars() {
        picker.on_key(Key::new(KeyCode::Char(c), Mods::NONE));
    }
    let names: Vec<String> = picker.rows().into_iter().map(|row| row.item.display).collect();
    assert_eq!(
        names,
        vec!["firefox", "firefly"],
        "equal scores keep arrival order"
    );
    assert_eq!(picker.query(), "fir");

    picker.on_key(Key::new(KeyCode::Backspace, Mods::NONE));
    assert_eq!(picker.rows().len(), 2, "\"fi\" still matches both");
    // Delete lands on the same path as backspace.
    picker.on_key(Key::new(KeyCode::Delete, Mods::NONE));
    assert_eq!(picker.query(), "f", "delete trims the query too");
    picker.on_key(Key::new(KeyCode::Char('i'), Mods::NONE));
    picker.on_key(Key::new(KeyCode::Backspace, Mods::NONE));
    picker.on_key(Key::new(KeyCode::Backspace, Mods::NONE));
    assert_eq!(picker.rows().len(), 4, "and an empty query restores everything");
}

#[test]
fn a_document_without_a_query_pane_still_types() {
    let mut picker = picker(
        r##"(
            window: (title: "dmenu"),
            layout: (pane: "list"),
            panes: [(id: "list", kind: Menu)],
        )"##,
    );
    picker.push(items(&["firefox", "herdr", "zathura"]));
    for c in "her".chars() {
        picker.on_key(Key::new(KeyCode::Char(c), Mods::NONE));
    }
    assert_eq!(picker.query(), "her");
    let names: Vec<String> = picker.rows().into_iter().map(|row| row.item.display).collect();
    assert_eq!(names, vec!["herdr"]);
    picker.on_key(Key::new(KeyCode::Backspace, Mods::NONE));
    assert_eq!(picker.query(), "he");
    assert_eq!(picker.rows().len(), 1, "\"he\" still only finds herdr");
}

#[test]
fn a_row_carries_the_characters_to_highlight() {
    let mut picker = picker(PICKER);
    picker.push(items(&["firefox"]));
    for c in "fox".chars() {
        picker.on_key(Key::new(KeyCode::Char(c), Mods::NONE));
    }
    let row: Row = picker.rows().pop().expect("one row");
    assert_eq!(row.positions, vec![4, 5, 6]);
}

#[test]
fn the_picker_drains_a_live_source() {
    let mut picker = picker(PICKER);
    let (sender, source) = mpsc::channel();
    assert!(!picker.drain(&source));
    sender.send(Item::line("late")).expect("send");
    std::thread::sleep(Duration::from_millis(20));
    assert!(picker.drain(&source), "something arrived");
    assert_eq!(picker.rows().len(), 1);
}

#[test]
fn a_nucleo_picker_fuzzes_where_the_simple_one_does_not() {
    let names = ["firefox", "kitty"];
    let spec = UiSpec::parse(PICKER, Format::Ron).expect("spec");

    let mut fuzzy = Picker::new(spec.clone()).expect("picker");
    fuzzy.push(items(&names));
    fuzzy.on_key(Key::new(KeyCode::Char('f'), Mods::NONE));
    fuzzy.on_key(Key::new(KeyCode::Char('f'), Mods::NONE));
    assert_eq!(fuzzy.rows().len(), 1, "nucleo matches f...f");

    let mut plain = Picker::new(spec).expect("picker").with_matcher(Box::new(SimpleMatcher::new()));
    plain.push(items(&names));
    plain.on_key(Key::new(KeyCode::Char('f'), Mods::NONE));
    plain.on_key(Key::new(KeyCode::Char('f'), Mods::NONE));
    assert_eq!(plain.rows().len(), 0, "the fallback does not");
}

#[test]
fn the_highlighted_rows_render() {
    let mut picker = picker(PICKER);
    picker.push(items(&["firefox", "firefly", "kitty"]));
    for c in "fir".chars() {
        picker.on_key(Key::new(KeyCode::Char(c), Mods::NONE));
    }
    picker.dispatch(Action::FocusNext);
    let area = Rect::new(0, 0, 30, 8);
    let mut buf = Buffer::empty(area);
    picker.render(area, &mut buf);
    insta::assert_snapshot!("picker_highlighted", ui::screen(&buf));
}

/// `-no-sort`: filtering and highlighting still happen, ranking does not, so a
/// caller whose rows already carry meaning keeps them where it put them.
#[test]
fn no_sort_keeps_the_provider_order() {
    let mut unsorted = picker(PICKER).no_sort(true);
    unsorted.push(items(&["zulu", "alpha", "mike"]));
    let names: Vec<String> = unsorted.rows().into_iter().map(|row| row.item.display).collect();
    assert_eq!(names, vec!["zulu", "alpha", "mike"], "arrival order survives");
    unsorted.on_key(Key::new(KeyCode::Char('m'), Mods::NONE));
    let filtered: Vec<String> = unsorted.rows().into_iter().map(|row| row.item.display).collect();
    assert_eq!(filtered, vec!["mike"], "filtering still applies");
}

/// `--accept-query`: rofi's custom input. Enter on an empty list answers with
/// what was typed; without the flag it stays silent, as a fixed-row menu wants.
#[test]
fn accept_query_answers_with_the_typed_text() {
    let mut custom = picker(PICKER).accept_query(true);
    custom.push(items(&["firefox", "kitty"]));
    for c in "no-such-app".chars() {
        custom.on_key(Key::new(KeyCode::Char(c), Mods::NONE));
    }
    assert!(custom.rows().is_empty(), "nothing matches");
    let outcome = custom.on_key(Key::new(KeyCode::Enter, Mods::NONE));
    assert_eq!(
        outcome,
        tuipalette_core::Outcome::Invoke {
            command: "no-such-app".to_owned(),
            value: Some("no-such-app".to_owned())
        }
    );

    let mut silent = picker(PICKER);
    silent.push(items(&["firefox"]));
    for c in "nope".chars() {
        silent.on_key(Key::new(KeyCode::Char(c), Mods::NONE));
    }
    assert_eq!(silent.on_key(Key::new(KeyCode::Enter, Mods::NONE)), tuipalette_core::Outcome::Continue);
}
