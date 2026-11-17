//! Item 05's acceptance: what the built-ins paint, pinned as snapshots so a
//! visual change fails a test instead of needing eyes.
//!
//! These capture the buffer as text (`ui::screen`), one string per row, so a
//! diff is readable. Colour is checked separately, where colour is the point.
//!
//! Accepting a deliberate visual change:
//! `INSTA_UPDATE=always cargo test -p tuipalette-core --test components`

use tuipalette_core::components::{Item, Menu, Spacer};
use tuipalette_core::spec::{Format, Theme};
use tuipalette_core::{
    ui, Action, Buffer, Component, Key, KeyCode, Mods, Outcome, Paint, Rect, Session, UiSpec,
};

fn spec(text: &str) -> UiSpec {
    UiSpec::parse(text, Format::Ron).unwrap_or_else(|e| panic!("fixture must parse: {e}"))
}

fn session(text: &str) -> Session {
    Session::from_spec(spec(text)).unwrap_or_else(|e| panic!("session must build: {e}"))
}

/// The whole screen a session paints at `width`x`height`.
fn screen(session: &Session, width: u16, height: u16) -> String {
    let area = Rect::new(0, 0, width, height);
    let mut buf = Buffer::empty(area);
    session.render(area, &mut buf);
    ui::screen(&buf)
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
          },
        Item::line("kitty"),
        Item {
            display: "code".to_owned(),
            value: "code".to_owned(),
            icon: Some(">_".to_owned()),
            search: None,
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
            Item { display: "firefox".to_owned(), value: "firefox".to_owned(), icon: icon.map(str::to_owned), search: None },
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
      }]);
    let area = Rect::new(0, 0, 20, 3);
    let mut buf = Buffer::empty(area);
    let theme = Theme::default();
    menu.render(&Paint { area, focused: true, theme: &theme }, &mut buf);
    let cell = buf.cell((1, 1)).map(|c| c.symbol().to_owned()).unwrap_or_default();
    assert_eq!(cell, theme.icon_placeholder, "a glyph that cannot fit must not be split");
}
