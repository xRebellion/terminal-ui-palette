//! The wire format, the geometry and the errors: this is the contract item 11
//! documents, so every claim here is one a user can rely on.

use ratatui_core::layout::Constraint;
use tuipalette_core::Rect;
use tuipalette_core::layout::{parse_constraint, resolve};
use tuipalette_core::spec::{Format, UiSpec};

fn fixture(format: Format) -> &'static str {
    match format {
        Format::Ron => include_str!("fixtures/launcher.ron"),
        Format::Json => include_str!("fixtures/launcher.json"),
    }
}

fn spec(text: &str) -> Result<UiSpec, String> {
    UiSpec::parse(text, Format::Ron).map_err(|e| e.to_string())
}

#[test]
fn both_formats_resolve_to_the_same_rects() {
    let expected = vec![
        ("query".to_owned(), Rect::new(0, 0, 40, 3)),
        ("results".to_owned(), Rect::new(0, 3, 40, 3)),
        ("preview".to_owned(), Rect::new(0, 6, 40, 3)),
        ("status".to_owned(), Rect::new(0, 9, 40, 1)),
    ];
    for format in [Format::Ron, Format::Json] {
        let parsed =
            UiSpec::parse(fixture(format), format).unwrap_or_else(|e| panic!("{format:?}: {e}"));
        let rects = resolve(&parsed.layout, Rect::new(0, 0, 40, 10)).unwrap();
        assert_eq!(rects, expected, "{format:?}");
    }
}

#[test]
fn horizontal_splits_differ_only_in_axis() {
    let parsed = spec(
        r#"(
            window: (title: "x"),
            layout: (dir: Horizontal, sizes: ["1", "fill"], children: [(pane: "a"), (pane: "b")]),
            panes: [(id: "a"), (id: "b")],
        )"#,
    )
    .unwrap();
    let rects = resolve(&parsed.layout, Rect::new(0, 0, 10, 4)).unwrap();
    assert_eq!(rects[0].1, Rect::new(0, 0, 1, 4));
    assert_eq!(rects[1].1, Rect::new(1, 0, 9, 4));
}

#[test]
fn spacing_separates_children_without_stealing_their_cells() {
    let parsed = spec(
        r#"(
            window: (title: "x"),
            layout: (dir: Vertical, spacing: 2, children: [(pane: "a"), (pane: "b")]),
            panes: [(id: "a"), (id: "b")],
        )"#,
    )
    .unwrap();
    let rects = resolve(&parsed.layout, Rect::new(0, 0, 10, 10)).unwrap();
    let (a, b) = (rects[0].1, rects[1].1);
    assert_eq!(a.y, 0);
    assert_eq!(a.height + b.height, 8, "10 rows minus two spacing rows");
    assert_eq!(a.y + a.height + 2, b.y);
}

#[test]
fn the_size_grammar_is_exactly_these_five() {
    assert_eq!(parse_constraint("3"), Some(Constraint::Length(3)));
    assert_eq!(parse_constraint("50%"), Some(Constraint::Percentage(50)));
    assert_eq!(parse_constraint("fill"), Some(Constraint::Fill(1)));
    assert_eq!(parse_constraint("min:5"), Some(Constraint::Min(5)));
    assert_eq!(parse_constraint("max:7"), Some(Constraint::Max(7)));
    assert_eq!(parse_constraint(" 1 "), Some(Constraint::Length(1)), "trimmed");
    for bad in ["", "wide", "150%", "%", "min:", "-1", "11px"] {
        assert_eq!(parse_constraint(bad), None, "`{bad}` must not parse");
    }
}

#[test]
fn a_ron_syntax_error_carries_a_position() {
    let error = spec(r#"(window: (title: "x""#).unwrap_err();
    assert!(error.contains("line 1, column"), "{error}");
}

#[test]
fn an_unknown_field_is_named_not_ignored() {
    let error = spec(
        r#"(
            window: (title: "x"),
            layout: (pane: "a"),
            panes: [(id: "a")],
            bogus: 1,
        )"#,
    )
    .unwrap_err();
    assert!(error.contains("bogus"), "{error}");
}

#[test]
fn a_typo_inside_a_node_is_named_too() {
    // Why a node is one struct with dir/children rather than an enum of
    // variants: serde cannot deny unknown fields inside a variant, so
    // `childrens` would silently become an empty split.
    let error = spec(
        r#"(
            window: (title: "x"),
            layout: (dir: Vertical, childrens: [(pane: "a")]),
            panes: [(id: "a")],
        )"#,
    )
    .unwrap_err();
    assert!(error.contains("childrens"), "{error}");
}

#[test]
fn an_undeclared_pane_is_reported_with_its_path() {
    let error = spec(
        r#"(
            window: (title: "x"),
            layout: (dir: Vertical, children: [(pane: "ghost")]),
            panes: [],
        )"#,
    )
    .unwrap_err();
    assert!(error.contains("layout.children[0].pane"), "{error}");
    assert!(error.contains("ghost"), "{error}");
}

#[test]
fn a_pane_may_only_be_drawn_once() {
    let error = spec(
        r#"(
            window: (title: "x"),
            layout: (dir: Vertical, children: [(pane: "a"), (pane: "a")]),
            panes: [(id: "a")],
        )"#,
    )
    .unwrap_err();
    assert!(error.contains("more than once"), "{error}");
}

#[test]
fn a_duplicate_pane_id_is_reported_at_its_index() {
    let error = spec(
        r#"(
            window: (title: "x"),
            layout: (pane: "a"),
            panes: [(id: "a"), (id: "a")],
        )"#,
    )
    .unwrap_err();
    assert!(error.contains("panes[1].id"), "{error}");
}

#[test]
fn sizes_must_match_children() {
    let error = spec(
        r#"(
            window: (title: "x"),
            layout: (dir: Vertical, sizes: ["1"], children: [(pane: "a"), (pane: "b")]),
            panes: [(id: "a"), (id: "b")],
        )"#,
    )
    .unwrap_err();
    assert!(error.contains("2 children"), "{error}");
}

#[test]
fn a_bad_size_names_its_index() {
    let error = spec(
        r#"(
            window: (title: "x"),
            layout: (dir: Vertical, sizes: ["1", "wide"], children: [(pane: "a"), (pane: "b")]),
            panes: [(id: "a"), (id: "b")],
        )"#,
    )
    .unwrap_err();
    assert!(error.contains("layout.sizes[1]"), "{error}");
    assert!(error.contains("wide"), "{error}");
}

#[test]
fn a_leaf_refuses_split_fields() {
    let error = spec(
        r#"(
            window: (title: "x"),
            layout: (pane: "a", dir: Vertical),
            panes: [(id: "a")],
        )"#,
    )
    .unwrap_err();
    assert!(error.contains("leaf node takes only"), "{error}");
}

#[test]
fn a_split_needs_a_direction() {
    let error = spec(
        r#"(
            window: (title: "x"),
            layout: (children: [(pane: "a")]),
            panes: [(id: "a")],
        )"#,
    )
    .unwrap_err();
    assert!(error.contains("needs `dir`"), "{error}");
}

#[test]
fn colours_are_hex_or_nothing() {
    let error = spec(
        r##"(
            window: (title: "x"),
            theme: (accent: "blue"),
            layout: (pane: "a"),
            panes: [(id: "a")],
        )"##,
    )
    .unwrap_err();
    assert!(error.contains("theme.accent"), "{error}");
    assert!(
        spec(
            r##"(
                window: (title: "x"),
                theme: (accent: "#89b4fa"),
                layout: (pane: "a"),
                panes: [(id: "a")],
            )"##,
        )
        .is_ok()
    );
}

#[test]
fn an_icon_column_of_zero_is_refused() {
    const RD: &str = r#"(
            window: (title: "x"),
            layout: (pane: "a"),
            panes: [(id: "a", icon_width: 0)],
        )"#;
    let error = spec(RD).unwrap_err();
    assert!(error.contains("icon_width"), "{error}");
}
