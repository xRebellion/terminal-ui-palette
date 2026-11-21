//! Print the top rows the picker ranks for a query, without a terminal.
//!
//!     scripts/apps.sh | scripts/usage.sh order 2 \
//!         | cargo run -q -p tuipalette-core --example rank -- kn 5
//!
//! Items are read in the line protocol (D12), so this is the same ordering a
//! real run shows. It exists to make ranking changes measurable (item 18).

use std::io::{self, BufRead};

use tuipalette_core::{parse_line, Format, Key, KeyCode, Mods, Picker, UiSpec};

const SPEC: &str = r#"(window: (title: "rank"), layout: (pane: "m"), panes: [(id: "m", kind: Menu)])"#;

fn main() {
    let mut args = std::env::args().skip(1);
    let query = args.next().unwrap_or_default();
    let top: usize = args.next().and_then(|n| n.parse().ok()).unwrap_or(5);
    let spec = UiSpec::parse(SPEC, Format::Ron).expect("the probe's own document");
    let mut picker = Picker::new(spec).expect("picker");
    let items = io::stdin()
        .lock()
        .lines()
        .map_while(Result::ok)
        .filter_map(|line| parse_line(&line));
    picker.push(items);
    // A document with no query pane still types (picker.rs), so the probe
    // types the query the way a user would.
    for c in query.chars() {
        picker.on_key(Key::new(KeyCode::Char(c), Mods::NONE));
    }
    for row in picker.rows().into_iter().take(top) {
        println!("{}", row.item.display);
    }
}
