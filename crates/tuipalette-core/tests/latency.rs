//! Item 09's second half: what one keystroke costs inside the library.
//!
//! The terminal's flush is the host's cost and is not measured here; this is
//! query → re-rank → draw into a buffer, which is everything the picker owns.

use std::time::Instant;

use tuipalette_core::spec::Format;
use tuipalette_core::{Buffer, Item, Key, KeyCode, Mods, Picker, Rect, UiSpec};

const DOC: &str = r##"(
    window: (title: "bench"),
    layout: (dir: Vertical, sizes: ["3", "fill", "1"], children: [
        (pane: "q"), (pane: "m"), (pane: "s"),
    ]),
    panes: [
        (id: "q", kind: Input, title: "query"),
        (id: "m", kind: Menu, title: "items"),
        (id: "s", kind: Status, items: ["enter run"]),
    ],
)"##;

fn picker() -> Picker {
    let spec = UiSpec::parse(DOC, Format::Ron).expect("the document is valid");
    let mut picker = Picker::new(spec).expect("the document is valid");
    let items: Vec<Item> = (0..1000).map(|i| Item::line(format!("item-{i:04}"))).collect();
    picker.push(items);
    picker
}

#[test]
fn a_keystroke_costs_far_less_than_a_frame() {
    let mut picker = picker();
    let area = Rect::new(0, 0, 60, 20);
    let mut buf = Buffer::empty(area);
    let keys = [
        Key::new(KeyCode::Char('i'), Mods::NONE),
        Key::new(KeyCode::Char('t'), Mods::NONE),
        Key::new(KeyCode::Down, Mods::NONE),
        Key::new(KeyCode::Up, Mods::NONE),
    ];
    let mut samples = Vec::new();
    for round in 0..40 {
        let key = keys[round % keys.len()];
        let start = Instant::now();
        picker.on_key(key);
        picker.render(area, &mut buf);
        samples.push(start.elapsed());
    }
    samples.sort();
    let median = samples[samples.len() / 2];
    eprintln!("key → repaint median: {:.3} ms (p95 {:.3} ms, 1000 items)",
        median.as_secs_f64() * 1000.0,
        samples[samples.len() * 19 / 20].as_secs_f64() * 1000.0);
    assert!(median.as_secs_f64() * 1000.0 < 5.0, "a keystroke took {median:?}");
}
