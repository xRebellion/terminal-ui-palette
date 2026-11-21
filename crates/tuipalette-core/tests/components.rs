//! Item 05's acceptance: what the built-ins paint, pinned as snapshots so a
//! visual change fails a test instead of needing eyes.
//!
//! These capture the buffer as text (`ui::screen`), one string per row, so a
//! diff is readable. Colour is checked separately, where colour is the point.
//!
//! Accepting a deliberate visual change:
//! `INSTA_UPDATE=always cargo test -p tuipalette-core --test components`

use tuipalette_core::components::{CounterPane, Item, Menu, Spacer};
use tuipalette_core::spec::{Format, ScrollbarStyle, Theme};
use tuipalette_core::{
    ui, Action, Buffer, Color, Component, Key, KeyCode, Mods, Outcome, Paint, Picker, Rect,
    Session, UiSpec,
};

/// The clipboard layout is the fzf picker's shape: one border around
/// everything, a prompt line, a header line, list on the left, the selected
/// entry on the right. Pinned because "close to the original" is a promise
/// someone will check against a screenshot.
#[test]
fn the_clipboard_layout_follows_the_selection() {
    let text = include_str!("../../../layouts/clipboard.ron");
    let mut picker = Picker::new(spec(text)).unwrap_or_else(|e| panic!("{e}"));
    picker.push(vec![
        Item {
            detail: Some("first entry, shown on the right\nsecond line of it".to_owned()),
            ..Item::line("13127  [[ binary data 18 KiB png 369x85 ]]")
        },
        Item {
            detail: Some("second entry".to_owned()),
            ..Item::line("13126  Short answer: only the three frontend screens")
        },
    ]);
    let area = Rect::new(0, 0, 64, 9);
    let mut buf = Buffer::empty(area);
    picker.render(area, &mut buf);
    insta::assert_snapshot!("clipboard_layout", ui::screen(&buf));

    picker.dispatch(Action::Down);
    let mut buf = Buffer::empty(area);
    picker.render(area, &mut buf);
    insta::assert_snapshot!("clipboard_layout_second_row", ui::screen(&buf));
}

fn spec(text: &str) -> UiSpec {
    UiSpec::parse(text, Format::Ron).unwrap_or_else(|e| panic!("fixture must parse: {e}"))
}

fn session(text: &str) -> Session {
    Session::from_spec(spec(text)).unwrap_or_else(|e| panic!("session must build: {e}"))
}

/// One session painted into a buffer, for the tests that have to look at a
/// cell rather than at the text.
fn buffered(session: &Session, width: u16, height: u16) -> Buffer {
    let area = Rect::new(0, 0, width, height);
    let mut buf = Buffer::empty(area);
    session.render(area, &mut buf);
    buf
}

/// The whole screen a session paints at `width`x`height`.
fn screen(session: &Session, width: u16, height: u16) -> String {
    ui::screen(&buffered(session, width, height))
}

/// One component, painted on its own.
fn painted(component: &dyn Component, width: u16, height: u16, focused: bool) -> String {
    let theme = Theme::default();
    let area = Rect::new(0, 0, width, height);
    let mut buf = Buffer::empty(area);
    component.render(
        &Paint {
            area,
            focused,
            theme: &theme,
        },
        &mut buf,
    );
    ui::screen(&buf)
}

fn press(session: &mut Session, times: usize, action: Action) {
    for _ in 0..times {
        session.dispatch(action.clone());
    }
}

// --- the query line ---------------------------------------------------------

const QUERY: &str = r##"(
    window: (title: "t"),
    layout: (pane: "q"),
    panes: [(id: "q", kind: Input, title: "search")],
)"##;

#[test]
fn a_query_line_accepts_typing_and_backspace() {
    let mut s = session(QUERY);
    for c in "kitty".chars() {
        s.dispatch(Action::Input(c));
    }
    assert_eq!(s.value().as_deref(), Some("kitty"));
    s.dispatch(Action::Backspace);
    assert_eq!(s.value().as_deref(), Some("kitt"));
    s.dispatch(Action::Input('\u{7}'));
    assert_eq!(s.value().as_deref(), Some("kitt"), "a control char is not text");
}

#[test]
fn a_query_line_shows_what_was_typed_with_a_cursor() {
    let mut s = session(QUERY);
    for c in "firefox".chars() {
        s.on_key(Key::new(KeyCode::Char(c), Mods::NONE));
    }
    insta::assert_snapshot!("query_line", screen(&s, 30, 3));
}

#[test]
fn a_long_query_shows_its_end() {
    let mut s = session(QUERY);
    for c in "a-very-long-query-that-cannot-fit".chars() {
        s.dispatch(Action::Input(c));
    }
    insta::assert_snapshot!("query_line_long", screen(&s, 20, 3));
}

// --- the menu ---------------------------------------------------------------

const MENU: &str = r##"(
    window: (title: "t"),
    layout: (pane: "m"),
    panes: [(id: "m", kind: Menu, title: "apps", items: [
        "one", "two", "three", "four", "five", "six", "seven", "eight",
    ])],
)"##;

#[test]
fn a_menu_clamps_at_both_ends() {
    let mut s = session(MENU);
    s.dispatch(Action::Up);
    assert_eq!(s.value().as_deref(), Some("one"), "up at the top stays put");
    press(&mut s, 20, Action::Down);
    assert_eq!(s.value().as_deref(), Some("eight"), "down at the end stays put");
}

#[test]
fn a_menu_accepts_its_value_and_emits_it() {
    let mut s = session(MENU);
    press(&mut s, 2, Action::Down);
    assert_eq!(
        s.dispatch(Action::Accept),
        Outcome::Invoke {
            command: "three".to_owned(),
            value: Some("three".to_owned()),
        }
    );
}

#[test]
fn a_menu_keeps_the_selection_inside_its_window() {
    let mut s = session(MENU);
    press(&mut s, 5, Action::Down);
    insta::assert_snapshot!("menu_scrolled", screen(&s, 24, 6));
}

#[test]
fn a_menu_reserves_an_icon_column_when_the_pane_asks() {
    let pane = &spec(
        r##"(
            window: (title: "t"),
            layout: (pane: "m"),
            panes: [(id: "m", kind: Menu, title: "apps", icon_width: 3)],
        )"##,
    )
    .panes[0];
    let mut menu = Menu::new(pane);
    menu.set_items(vec![
        Item {
            display: "firefox".to_owned(),
            value: "firefox".to_owned(),
            icon: Some("ff".to_owned()),
            search: None,
            detail: None,
          },
        Item::line("kitty"),
        Item {
            display: "code".to_owned(),
            value: "code".to_owned(),
            icon: Some(">_".to_owned()),
            search: None,
            detail: None,
          },
    ]);
    insta::assert_snapshot!("menu_icons", painted(&menu, 28, 6, true));
}

#[test]
fn a_wide_icon_does_not_move_the_columns() {
    // A double-width glyph occupies one buffer cell and one shadow cell. The
    // slot and the marker must stay on their columns either way; making the
    // glyph itself centre properly inside its column is item 15's job.
    let theme = Theme::default();
    let pane = &spec(
        r##"(
            window: (title: "t"),
            layout: (pane: "m"),
            panes: [(id: "m", kind: Menu, title: "apps", icon_width: 3)],
        )"##,
    )
    .panes[0];
    let mut menu = Menu::new(pane);
    menu.set_items(vec![Item {
        display: "firefox".to_owned(),
        value: "firefox".to_owned(),
        icon: Some("🌐".to_owned()),
        search: None,
        detail: None,
      }]);
    let area = Rect::new(0, 0, 24, 3);
    let mut buf = Buffer::empty(area);
    menu.render(
        &Paint {
            area,
            focused: true,
            theme: &theme,
        },
        &mut buf,
    );
    let cell = |x: u16| buf.cell((x, 1)).map(|c| c.symbol().to_owned()).unwrap_or_default();
    assert_eq!(cell(1), "🌐", "the glyph sits in the first cell of the column");
    assert_eq!(cell(1 + 3 + 1), "›", "and the marker keeps its column");
}

#[test]
fn a_menu_clamps_its_selection_when_the_items_shrink() {
    let pane = &spec(MENU).panes[0];
    let mut menu = Menu::new(pane);
    for _ in 0..7 {
        menu.update(&Action::Down);
    }
    assert_eq!(menu.selected().map(|i| i.display.as_str()), Some("eight"));
    menu.set_items(vec![Item::line("solo")]);
    assert_eq!(menu.selected().map(|i| i.display.as_str()), Some("solo"));
}

#[test]
fn the_selected_row_is_painted_with_the_selected_colour() {
    let theme = Theme::default();
    let pane = &spec(MENU).panes[0];
    let mut menu = Menu::new(pane);
    menu.update(&Action::Down);
    let area = Rect::new(0, 0, 20, 5);
    let mut buf = Buffer::empty(area);
    menu.render(
        &Paint {
            area,
            focused: true,
            theme: &theme,
        },
        &mut buf,
    );
    let row = Rect::new(1, 2, 18, 1);
    for x in row.left()..row.right() {
        assert_eq!(
            buf.cell((x, 2)).and_then(|cell| cell.style().bg),
            Some(theme.selected_color()),
            "row background at x={x}"
        );
    }
}

// --- the scrollbar (22) -----------------------------------------------------

/// Five rows in a three-row window, asking for the glyph style: `█` and `░`
/// are what a snapshot can see, which is why these two snapshots still use it.
const SCROLLED: &str = r##"(
    window: (title: "t"),
    layout: (pane: "m"),
    panes: [(id: "m", kind: Menu, title: "apps", scrollbar: true, scrollbar_style: Glyph, items: [
        "one", "two", "three", "four", "five",
    ])],
)"##;

/// The same five rows with the style left out — the default a document gets.
const TINTED: &str = r##"(
    window: (title: "t"),
    layout: (pane: "m"),
    panes: [(id: "m", kind: Menu, title: "apps", scrollbar: true, items: [
        "one", "two", "three", "four", "five",
    ])],
)"##;

/// Twelve rows in a pane whose window holds three of them, with the scrollbar
/// style spelled out — or left to the default (docs/specs/22-scrollbar.md).
fn twelve(style: Option<ScrollbarStyle>) -> Session {
    let style = style
        .map(|style| format!(", scrollbar_style: {}", style.name()))
        .unwrap_or_default();
    session(&format!(
        r##"(
    window: (title: "t"),
    layout: (pane: "m"),
    panes: [(id: "m", kind: Menu, scrollbar: true{style}, items: [
        "01", "02", "03", "04", "05", "06", "07", "08", "09", "10", "11", "12",
    ])],
)"##
    ))
}

/// The screen rows holding the thumb, read back as text.
fn thumb_rows(text: &str) -> Vec<usize> {
    text.lines()
        .enumerate()
        .filter(|(_, line)| line.contains('\u{2588}'))
        .map(|(row, _)| row)
        .collect()
}

/// The rows of column `x` whose cell background is `bg`. The tinted style has
/// no glyph to look for, so its bar is read back from the cells themselves —
/// the one observation a text snapshot cannot make.
fn bg_rows(buf: &Buffer, x: u16, bg: Color) -> Vec<usize> {
    (0..buf.area().height)
        .filter(|row| buf.cell((x, *row)).and_then(|cell| cell.style().bg) == Some(bg))
        .map(usize::from)
        .collect()
}

/// Five rows in a three-row window: a two-cell thumb at the top says "there is
/// more below", and at the end the same bar sits at the bottom.
#[test]
fn a_scrollbar_shows_where_a_list_is() {
    let mut s = session(SCROLLED);
    insta::assert_snapshot!("menu_scrollbar", screen(&s, 20, 5));
    press(&mut s, 4, Action::Down);
    insta::assert_snapshot!("menu_scrollbar_end", screen(&s, 20, 5));
}

/// The default style says the same thing as `menu_scrollbar` with no glyph at
/// all: five rows in a three-row track is a two-cell thumb at the top, painted
/// as `scrollbar` cells with `scrollbar_bg` cells below them.
#[test]
fn a_background_scrollbar_tints_the_last_column() {
    let theme = Theme::default();
    let buf = buffered(&session(TINTED), 20, 5);
    let bg = |x: u16, y: u16| {
        buf.cell((x, y))
            .unwrap_or_else(|| panic!("no cell ({x},{y})"))
            .style()
            .bg
    };

    // 20 cells wide and framed: the last inner column is x=18, the track y=1..3.
    assert_eq!(bg(18, 1), Some(theme.scrollbar_color()), "thumb: the top cell of the track");
    assert_eq!(bg(18, 2), Some(theme.scrollbar_color()), "thumb: two of the three track cells");
    assert_eq!(bg(18, 3), Some(theme.scrollbar_bg_color()), "track: the cell below is tinted");
    assert_eq!(bg(17, 1), Some(theme.selected_color()), "the selected row keeps its background");
    for y in 1..4 {
        assert_eq!(
            buf.cell((18, y)).map(|cell| cell.symbol()),
            Some(" "),
            "a background bar writes no glyph at y={y}"
        );
    }
    assert!(!ui::screen(&buf).contains('\u{2588}'), "no thumb glyph anywhere");
}

/// One bar in two styles: the same column, the same thumb, from the same
/// window. Twelve rows in a three-row track is a one-cell thumb, and both
/// styles put it on the same row at every step the selection makes.
#[test]
fn both_scrollbar_styles_draw_the_same_geometry() {
    let theme = Theme::default();
    let mut glyph = twelve(Some(ScrollbarStyle::Glyph));
    let mut tinted = twelve(None);
    for step in 0..12 {
        let marked = thumb_rows(&screen(&glyph, 18, 5));
        let painted = bg_rows(&buffered(&tinted, 18, 5), 16, theme.scrollbar_color());
        assert_eq!(painted, marked, "step {step}: the tinted thumb is not where the glyph is");
        glyph.dispatch(Action::Down);
        tinted.dispatch(Action::Down);
    }
}

/// The bar takes the last column only when it has something to say: five rows
/// in seven cells draws no bar and no column is taken from the text.
#[test]
fn no_scrollbar_when_every_row_fits() {
    let s = session(SCROLLED);
    let text = screen(&s, 20, 9);
    assert!(!text.contains('\u{2588}') && !text.contains('\u{2591}'), "{text}");
}

/// The thumb is the selection's place in the list, and it only ever moves one
/// way while the selection does. Twelve rows in three: a one-cell thumb that
/// starts on the first row of the track and ends on the last.
#[test]
fn the_scrollbar_thumb_follows_the_selection() {
    let mut s = twelve(Some(ScrollbarStyle::Glyph));
    let mut tops = Vec::new();
    for _ in 0..12 {
        let text = screen(&s, 18, 5);
        let rows = thumb_rows(&text);
        assert_eq!(rows.len(), 1, "a 12-row list in three rows is a one-cell thumb:\n{text}");
        tops.push(rows[0]);
        s.dispatch(Action::Down);
    }
    assert_eq!(tops[0], 1, "the thumb starts on the first row inside the frame: {tops:?}");
    assert_eq!(tops[11], 3, "and ends on the last: {tops:?}");
    assert!(tops.contains(&2), "it passes through the middle: {tops:?}");
    assert!(tops.windows(2).all(|pair| pair[1] >= pair[0]), "the thumb never goes backwards: {tops:?}");
}

// --- the counter (23) -------------------------------------------------------

const COUNTED: &str = r##"(
    window: (title: "t"),
    layout: (dir: Vertical, sizes: ["1", "fill", "1"], children: [
        (pane: "q"), (pane: "m"), (pane: "c"),
    ]),
    panes: [
        (id: "q", kind: Input, frame: false),
        (id: "m", kind: Menu, frame: false),
        (id: "c", kind: Counter, align: Right, frame: false),
    ],
)"##;

fn typed(picker: &mut Picker, text: &str) {
    for c in text.chars() {
        picker.on_key(Key::new(KeyCode::Char(c), Mods::NONE));
    }
}

fn counted(picker: &Picker, id: &str) -> String {
    picker
        .session()
        .pane::<CounterPane>(id)
        .unwrap_or_else(|| panic!("no counter pane {id}"))
        .text()
}

/// Typed query, four rows in, two left: `2/4`, hard against the right edge.
#[test]
fn the_counter_shows_shown_over_total() {
    let mut picker = Picker::new(spec(COUNTED)).expect("picker must build");
    picker.push(vec![
        Item::line("alpha"),
        Item::line("beta"),
        Item::line("delta"),
        Item::line("gamma"),
    ]);
    typed(&mut picker, "ta");
    let area = Rect::new(0, 0, 24, 6);
    let mut buf = Buffer::empty(area);
    picker.render(area, &mut buf);
    insta::assert_snapshot!("counter", ui::screen(&buf));
}

/// Both numbers are the picker's, so both move: a keystroke narrows the list,
/// and a row arriving widens the total behind the same query.
#[test]
fn the_counter_follows_typing_and_streaming_rows() {
    let mut picker = Picker::new(spec(COUNTED)).expect("picker must build");
    assert_eq!(counted(&picker, "c"), "0/0", "nothing has arrived yet");
    picker.push(vec![
        Item::line("alpha"),
        Item::line("beta"),
        Item::line("delta"),
        Item::line("gamma"),
    ]);
    assert_eq!(counted(&picker, "c"), "4/4");
    typed(&mut picker, "ta");
    assert_eq!(counted(&picker, "c"), "2/4", "beta and delta contain ta");
    picker.push(vec![Item::line("tangent"), Item::line("velvet")]);
    assert_eq!(counted(&picker, "c"), "3/6", "the new row matches, so both numbers moved");
}

/// A document can write the numbers into a sentence and put them anywhere in
/// the rect; nothing but `{shown}` and `{total}` is interpreted.
#[test]
fn a_counter_takes_its_wording_and_its_edge_from_the_document() {
    let cases = [
        ("Left", 0usize),
        ("Center", 7),
        ("Right", 14),
    ];
    for (edge, column) in cases {
        let pane = &spec(&format!(
            "(window: (title: \"t\"), layout: (pane: \"c\"), panes: [(id: \"c\", kind: Counter, frame: false, format: \"{{shown}} of {{total}}\", align: {edge})])"
        ))
        .panes[0];
        let mut counter = CounterPane::new(pane);
        counter.set_counts(3, 49);
        let text = painted(&counter, 21, 1, false);
        assert_eq!(text.trim(), "3 of 49");
        assert_eq!(text.find('3'), Some(column), "align: {edge}");
    }
}

/// The clipboard's header row is fzf's info line: the label on the left and
/// the count against the right edge of the same row.
#[test]
fn the_clipboard_header_shows_the_count_beside_its_label() {
    let text = include_str!("../../../layouts/clipboard.ron");
    let mut picker = Picker::new(spec(text)).expect("the shipped layout must build");
    picker.push(vec![Item::line("first entry"), Item::line("second entry")]);
    let area = Rect::new(0, 0, 64, 9);
    let mut buf = Buffer::empty(area);
    picker.render(area, &mut buf);
    let header = ui::screen(&buf);
    let row = header.lines().nth(2).unwrap_or_default().to_owned();
    assert!(row.contains("Clipboard History"), "{row:?}");
    // Right-aligned in its own pane, so it is the last thing before the border.
    let before_the_border = row.trim_end_matches(['│', ' ']);
    assert!(before_the_border.ends_with("2/2"), "{row:?}");
}

/// A full history is the case that matters: cliphist keeps 250 entries here,
/// and the header must show every digit of both numbers at the window size
/// the layout asks for (106 wide, so the left column is 41 cells).
#[test]
fn the_clipboard_count_keeps_every_digit_of_a_full_history() {
    let text = include_str!("../../../layouts/clipboard.ron");
    for (rows, query, expected) in [(250, "", "250/250"), (250, "1", "133/250"), (1000, "", "1000/1000")] {
        let mut picker = Picker::new(spec(text)).expect("the shipped layout must build");
        picker.push((1..=rows).map(|n| Item::line(&n.to_string())));
        typed(&mut picker, query);
        let area = Rect::new(0, 0, 106, 26);
        let mut buf = Buffer::empty(area);
        picker.render(area, &mut buf);
        let screen = ui::screen(&buf);
        let row = screen.lines().nth(2).unwrap_or_default();
        assert!(row.contains("Clipboard History"), "{row:?}");
        // The outer border, then the left column, then the preview's frame.
        let left = row.split('│').nth(1).unwrap_or_default().trim_end();
        assert!(left.ends_with(&format!(" {expected}")), "{rows} rows, query {query:?}: {row:?}");
    }
}

/// When a counter really has no room, it gives up the left of its text, not
/// the right: `…/250` is short, `250/25` would be wrong.
#[test]
fn a_counter_too_narrow_for_its_text_keeps_the_total() {
    let pane = &spec(
        "(window: (title: \"t\"), layout: (pane: \"c\"), panes: [(id: \"c\", kind: Counter, frame: false, align: Right)])",
    )
    .panes[0];
    let mut counter = CounterPane::new(pane);
    counter.set_counts(250, 250);
    assert_eq!(painted(&counter, 7, 1, false), "250/250");
    assert_eq!(painted(&counter, 6, 1, false), "\u{2026}0/250");
    assert_eq!(painted(&counter, 1, 1, false), "\u{2026}");
}

// --- status and spacer ------------------------------------------------------

#[test]
fn a_status_line_joins_its_hints() {
    let s = session(
        r##"(
            window: (title: "t"),
            layout: (pane: "s"),
            panes: [(id: "s", kind: Status, items: ["enter run", "tab next", "esc quit"])],
        )"##,
    );
    insta::assert_snapshot!("status_line", screen(&s, 40, 1));
}

#[test]
fn a_spacer_draws_nothing() {
    let pane = &spec(
        r##"(
            window: (title: "t"),
            layout: (pane: "gap"),
            panes: [(id: "gap", kind: Spacer)],
        )"##,
    )
    .panes[0];
    let spacer = Spacer::new(pane);
    assert_eq!(painted(&spacer, 12, 3, true).trim(), "");
}

// --- the whole thing --------------------------------------------------------

const LAUNCHER: &str = r##"(
    window: (title: "launcher"),
    theme: (accent: "#89b4fa", selected_bg: "#313244", text: "#cdd6f4", dim: "#6c7086"),
    layout: (dir: Vertical, sizes: ["3", "fill", "1"], children: [
        (pane: "query"),
        (dir: Horizontal, sizes: ["60%", "fill"], children: [(pane: "results"), (pane: "preview")]),
        (pane: "status"),
    ]),
    panes: [
        (id: "query", kind: Input, title: "search"),
        (id: "results", kind: Menu, title: "applications", items: [
            "firefox", "kitty", "code", "herdr", "spotify", "discord",
            "slack", "telegram", "obsidian", "mpv", "zathura", "thunar",
        ]),
        (id: "preview", kind: Text, title: "preview", items: [
            "firefox",
            "internet browser",
            "130 MB",
        ]),
        (id: "status", kind: Status, items: ["enter run", "tab next", "esc quit"]),
    ],
)"##;

#[test]
fn the_launcher_renders() {
    let mut s = session(LAUNCHER);
    for c in "fir".chars() {
        s.on_key(Key::new(KeyCode::Char(c), Mods::NONE));
    }
    press(&mut s, 3, Action::Down);
    insta::assert_snapshot!("launcher", screen(&s, 60, 14));
}

#[test]
fn focus_moves_between_the_query_and_the_menu_only() {
    let mut s = session(LAUNCHER);
    assert_eq!(s.focus(), Some("query"));
    s.dispatch(Action::FocusNext);
    assert_eq!(s.focus(), Some("results"), "preview and status are labels");
    s.dispatch(Action::FocusNext);
    assert_eq!(s.focus(), Some("query"), "and the ring wraps");
}

/// The four ways an icon slot can be filled, rendered side by side. The text
/// column must not move: a native-width glyph, a double-width emoji, an absent
/// glyph (the theme's placeholder steps in) and no slot at all.
#[test]
fn the_icon_slot_keeps_the_text_column_in_all_four_cases() {
    let cases: [(&str, Option<&str>, Option<u8>); 4] = [
        ("native", Some("◆"), Some(3)),
        ("wide", Some("🎉"), Some(3)),
        ("absent", None, Some(3)),
        ("off", Some("◆"), None),
    ];
    let mut matrix = String::new();
    let mut columns = Vec::new();
    for (name, icon, icon_width) in cases {
        let pane = &spec(&format!(
            "(window: (title: \"t\"), layout: (pane: \"m\"), panes: [(id: \"m\", kind: Menu, title: \"apps\"{})])",
            icon_width.map(|w| format!(", icon_width: {w}")).unwrap_or_default()
        ))
        .panes[0];
        let mut menu = Menu::new(pane);
        menu.set_items(vec![
            Item { display: "firefox".to_owned(), value: "firefox".to_owned(), icon: icon.map(str::to_owned), search: None, detail: None },
            Item::line("kitty"),
        ]);
        let area = Rect::new(0, 0, 24, 4);
        let mut buf = Buffer::empty(area);
        let theme = Theme::default();
        menu.render(
            &Paint { area, focused: true, theme: &theme },
            &mut buf,
        );
        let text = ui::screen(&buf);
        // Where "firefox" starts is the column the layout has to keep fixed.
        // Measured on buffer cells, not on the extracted text: the cell after
        // a double-width glyph holds no symbol of its own, and `screen` turns
        // that continuation cell into a space.
        let column = (0..area.width)
            .find(|&x| buf.cell((x, 1)).is_some_and(|cell| cell.symbol() == "f"))
            .expect("the first row carries the display text") as i32;
        columns.push((name, column, icon_width));
        matrix.push_str(&format!("-- {name} --\n{text}\n"));
    }
    let with_slot: Vec<_> = columns.iter().filter(|(_, _, w)| w.is_some()).collect();
    for (name, column, _) in &with_slot {
        assert_eq!(
            *column, with_slot[0].1,
            "case {name} moved the text column"
        );
    }
    let off = columns.iter().find(|(name, _, _)| *name == "off").unwrap();
    assert_eq!(off.1, with_slot[0].1 - 4, "--no-icons should drop the slot and its gap");
    insta::assert_snapshot!("icon_matrix", matrix);
}

/// A glyph wider than its slot is dropped rather than cut in half — the theme's
/// placeholder takes the cell, so the row stays legible and aligned.
#[test]
fn a_glyph_too_wide_for_its_slot_falls_back_to_the_placeholder() {
    let pane = &spec(
        r##"(window: (title: "t"), layout: (pane: "m"), panes: [(id: "m", kind: Menu, title: "apps", icon_width: 1)])"##,
    )
    .panes[0];
    let mut menu = Menu::new(pane);
    menu.set_items(vec![Item {
        display: "firefox".to_owned(),
        value: "firefox".to_owned(),
        icon: Some("🎉".to_owned()),
        search: None,
        detail: None,
      }]);
    let area = Rect::new(0, 0, 20, 3);
    let mut buf = Buffer::empty(area);
    let theme = Theme::default();
    menu.render(&Paint { area, focused: true, theme: &theme }, &mut buf);
    let cell = buf.cell((1, 1)).map(|c| c.symbol().to_owned()).unwrap_or_default();
    assert_eq!(cell, theme.icon_placeholder, "a glyph that cannot fit must not be split");
}
