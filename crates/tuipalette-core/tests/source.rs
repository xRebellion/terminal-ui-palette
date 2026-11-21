//! Item 06's acceptance: the line protocol, a source that delivers before EOF,
//! and the picker that ranks and highlights what survives the query.

use std::io::Read;
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

use tuipalette_core::components::DetailPane;
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
    // The icon is one field, sanitised like the display: it is one cell slot,
    // and a control character in it would break the column.
    assert_eq!(remainder.icon.as_deref(), Some("c"));
    assert_eq!(remainder.search.as_deref(), Some("d"), "the fourth field is search text");
    assert_eq!(remainder.detail, None, "no fifth field, no detail");

    let preview = parse_line("a\tb\t\t\tfirst line of the preview").expect("a line");
    assert_eq!(preview.icon, None, "an empty field is an absent one");
    assert_eq!(preview.search, None);
    assert_eq!(preview.detail.as_deref(), Some("first line of the preview"));
}

#[test]
fn a_row_can_be_found_by_words_it_does_not_show() {
    // rofi's drun matches Name, GenericName, Exec, Categories and Keywords
    // while showing the Name; the fourth field is where those extra words go.
    let row = parse_line("Passwords and Keys\tseahorse\t\tGNOME Security keyring seahorse")
        .expect("a line");
    assert_eq!(row.display, "Passwords and Keys");
    assert_eq!(row.value, "seahorse");
    assert_eq!(row.icon, None, "an empty third field is no icon");
    assert_eq!(row.search.as_deref(), Some("GNOME Security keyring seahorse"));

    let plain = parse_line("firefox").expect("a line");
    assert_eq!(plain.search, None, "three fields stay three fields");
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

const PREVIEW: &str = r##"(
    window: (title: "t"),
    layout: (dir: Horizontal, sizes: ["fill", "20%"], children: [(pane: "list"), (pane: "peek")]),
    panes: [
        (id: "list", kind: Menu),
        (id: "peek", kind: Detail, from: "list"),
    ],
)"##;

/// A detail pane shows the selected row's `detail`, and follows the
/// selection: it is a pane, not a second list.
#[test]
fn a_detail_pane_follows_the_selection() {
    let mut picker = picker(PREVIEW);
    picker.push(vec![
        Item {
            detail: Some("first preview".to_owned()),
            ..Item::line("first")
        },
        Item {
            detail: Some("second preview\nover two lines".to_owned()),
            ..Item::line("second")
        },
    ]);
    let shown = |picker: &Picker| {
        picker
            .session()
            .pane::<DetailPane>("peek")
            .and_then(|pane| pane.text().map(str::to_owned))
    };
    assert_eq!(shown(&picker).as_deref(), Some("first preview"), "the first row is selected");
    picker.dispatch(Action::Down);
    assert_eq!(shown(&picker).as_deref(), Some("second preview\nover two lines"));
    picker.dispatch(Action::Up);
    assert_eq!(shown(&picker).as_deref(), Some("first preview"));
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

// --- ranking (18) -----------------------------------------------------------

/// The default (nucleo) picker over `names`, in this arrival order, with
/// `query` typed: the displays it shows, best first.
fn ranked(names: &[&str], query: &str) -> Vec<String> {
    let spec = UiSpec::parse(PICKER, Format::Ron).expect("spec");
    let mut picker = Picker::new(spec).expect("picker");
    picker.push(items(names));
    for c in query.chars() {
        picker.on_key(Key::new(KeyCode::Char(c), Mods::NONE));
    }
    picker.rows().into_iter().map(|row| row.item.display).collect()
}

/// The user's report: `kn` led with Konsole (a scattered K…n with word-start
/// bonuses) over Arknights, which holds `kn` literally. A literal hit now
/// outranks every scattered one, whatever the scorer's number says.
#[test]
fn a_literal_hit_outranks_a_scattered_one() {
    let names = ["Konsole", "KDE Connect Indicator", "Arknights", "CachyOS Kernel Manager"];
    let rows = ranked(&names, "kn");
    assert_eq!(rows[0], "Arknights", "{rows:?}");
    assert_eq!(rows.len(), 4, "the scattered rows still match: {rows:?}");
    let rows = ranked(&names, "kni");
    assert_eq!(rows[0], "Arknights", "{rows:?}");
}

/// Literal hits order among themselves by where they hit — the whole name,
/// its start, a word's start, anywhere — and only then by arrival.
#[test]
fn literal_hits_rank_exact_then_prefix_then_word_then_inside() {
    let names = ["Bookmarks", "Ark Manager", "Arknights", "Ark", "Spark"];
    assert_eq!(
        ranked(&names, "ark"),
        vec!["Ark", "Ark Manager", "Arknights", "Bookmarks", "Spark"],
    );
    let names = ["Bookmark Ark", "Spark"];
    assert_eq!(ranked(&names, "ark"), vec!["Bookmark Ark", "Spark"], "a word start beats inside a word");
}

/// Inside a tier the provider's order is kept. That is how usage ranks typed
/// results without the library knowing about it: `usage.sh` hands the list
/// over most-recent first.
#[test]
fn equally_good_hits_keep_the_providers_order() {
    let recent_first = ["Arknights", "Arknights: Endfield", "Konsole"];
    assert_eq!(ranked(&recent_first, "kn")[..2], ["Arknights", "Arknights: Endfield"]);
    let other_way = ["Arknights: Endfield", "Arknights", "Konsole"];
    assert_eq!(ranked(&other_way, "kn")[..2], ["Arknights: Endfield", "Arknights"]);
}

/// A literal row is highlighted where it matched literally, not where the
/// fuzzy scorer would have put its characters.
#[test]
fn a_literal_row_highlights_the_run_it_matched() {
    let spec = UiSpec::parse(PICKER, Format::Ron).expect("spec");
    let mut picker = Picker::new(spec).expect("picker");
    picker.push(items(&["Arknights"]));
    for c in "kn".chars() {
        picker.on_key(Key::new(KeyCode::Char(c), Mods::NONE));
    }
    assert_eq!(picker.rows()[0].positions, vec![2, 3]);
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

/// A row can be found by words it does not show — rofi's drun finds "Passwords
/// and Keys" by its `seahorse` executable. Such a hit is real and ranks below
/// every display match, and it has no highlight: positions index the display.
#[test]
fn a_hit_on_hidden_words_is_a_real_but_shallower_match() {
    let mut picker = picker(PICKER);
    picker.push(vec![
        Item {
            display: "Passwords and Keys".to_owned(),
            value: "seahorse".to_owned(),
            icon: None,
            search: Some("GNOME Security keyring seahorse".to_owned()),
            detail: None,
            image: None,
        },
        Item::line("SeaMonkey"),
    ]);
    for ch in "sea".chars() {
        picker.on_key(Key::new(KeyCode::Char(ch), Mods::NONE));
    }
    let rows = picker.rows();
    let names: Vec<&str> = rows.iter().map(|row| row.item.display.as_str()).collect();
    assert_eq!(names, vec!["SeaMonkey", "Passwords and Keys"], "a display match leads");
    assert!(!rows[0].positions.is_empty(), "the display hit is highlighted");
    assert!(rows[1].positions.is_empty(), "a hidden hit has nothing to highlight");
    assert_eq!(rows[1].item.value, "seahorse", "the value still comes back");
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

/// `image` comes through `--in json` as a path, and an empty one is absent.
#[test]
fn a_json_item_carries_an_image_path() {
    let item = tuipalette_core::parse_json_line(r#"{"display": "shot", "value": "7", "image": "/run/a.png"}"#).expect("item");
    assert_eq!(item.image.as_deref(), Some("/run/a.png"));
    let none = tuipalette_core::parse_json_line(r#"{"display": "x", "image": ""}"#).expect("item");
    assert_eq!(none.image, None);
}
