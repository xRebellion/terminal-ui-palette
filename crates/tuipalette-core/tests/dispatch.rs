//! Item 04's acceptance: the focus ring, bubbling, emit, and the key map's
//! override and unset rules — all without a terminal.
//!
//! Run with `cargo test -p tuipalette-core -- dispatch`.

use std::cell::RefCell;
use std::rc::Rc;

use tuipalette_core::components::InputLine;
use tuipalette_core::key;
use tuipalette_core::spec::Format;
use tuipalette_core::{
    Action, Buffer, Component, Key, KeyCode, Mods, Outcome, Paint, Phase, Reaction, Rect, Session,
    Style, UiSpec,
};

// --- a component that does exactly what a test needs -------------------------

type Log = Rc<RefCell<Vec<Action>>>;

struct Spy {
    id: &'static str,
    focusable: bool,
    value: Option<&'static str>,
    log: Log,
    react: fn(&Action) -> Reaction,
}

impl Spy {
    fn new(id: &'static str, react: fn(&Action) -> Reaction) -> Self {
        Self {
            id,
            focusable: true,
            value: None,
            log: Rc::new(RefCell::new(Vec::new())),
            react,
        }
    }

    fn label(id: &'static str) -> Self {
        Self {
            focusable: false,
            ..Self::new(id, ignore)
        }
    }

    fn value(mut self, value: &'static str) -> Self {
        self.value = Some(value);
        self
    }

    fn boxed(self) -> Box<dyn Component> {
        Box::new(self)
    }

    fn seen(&self) -> Vec<Action> {
        self.log.borrow().clone()
    }
}

impl Component for Spy {
    fn id(&self) -> &str {
        self.id
    }

    fn focusable(&self) -> bool {
        self.focusable
    }

    fn render(&self, paint: &Paint<'_>, buf: &mut Buffer) {
        let label = if paint.focused {
            format!("{}*", self.id)
        } else {
            self.id.to_owned()
        };
        buf.set_string(paint.area.x, paint.area.y, label, Style::default());
    }

    fn update(&mut self, action: &Action) -> Reaction {
        self.log.borrow_mut().push(action.clone());
        (self.react)(action)
    }

    fn value(&self) -> Option<String> {
        self.value.map(str::to_owned)
    }
}

fn ignore(_: &Action) -> Reaction {
    Reaction::Ignored
}

fn handle(_: &Action) -> Reaction {
    Reaction::Handled
}

fn emit_run(_: &Action) -> Reaction {
    Reaction::Emit(Action::Invoke("run:firefox".to_owned()))
}

fn emit_focus_next(_: &Action) -> Reaction {
    Reaction::Emit(Action::FocusNext)
}

// --- fixtures ---------------------------------------------------------------

const THREE: &str = r##"(
    window: (title: "t"),
    layout: (dir: Vertical, sizes: ["1", "1", "1"], children: [(pane: "a"), (pane: "b"), (pane: "c")]),
    panes: [(id: "a"), (id: "b"), (id: "c")],
)"##;

fn spec(text: &str) -> UiSpec {
    UiSpec::parse(text, Format::Ron).unwrap_or_else(|e| panic!("fixture must parse: {e}"))
}

fn session(components: Vec<Box<dyn Component>>) -> Session {
    match Session::new(spec(THREE), components) {
        Ok(session) => session,
        Err(e) => panic!("session must build: {e}"),
    }
}

fn key(code: KeyCode, mods: Mods) -> Key {
    Key::new(code, mods)
}

// --- the focus ring ---------------------------------------------------------

#[test]
fn focus_next_skips_panes_that_do_not_take_focus() {
    let mut s = session(vec![
        Spy::new("a", ignore).boxed(),
        Spy::label("b").boxed(),
        Spy::new("c", ignore).boxed(),
    ]);
    assert_eq!(s.focus(), Some("a"), "first focusable in draw order");
    s.dispatch(Action::FocusNext);
    assert_eq!(s.focus(), Some("c"), "b is a label and is stepped over");
    s.dispatch(Action::FocusNext);
    assert_eq!(s.focus(), Some("a"), "the ring wraps");
    s.dispatch(Action::FocusPrev);
    assert_eq!(s.focus(), Some("c"), "and wraps the other way");
}

// --- bubbling ---------------------------------------------------------------

#[test]
fn an_action_the_focused_pane_ignores_bubbles_once_and_stops() {
    let a = Rc::new(Spy::new("a", ignore));
    let b = Rc::new(Spy::new("b", handle));
    let c = Rc::new(Spy::new("c", handle));
    let mut s = Session::new(
        spec(THREE),
        vec![
            Box::new(Shared::new(a.clone())),
            Box::new(Shared::new(b.clone())),
            Box::new(Shared::new(c.clone())),
        ],
    )
    .expect("session");

    s.dispatch(Action::Down);
    assert_eq!(a.seen(), vec![Action::Down], "the focused pane sees it first");
    assert_eq!(b.seen(), vec![Action::Down], "it bubbles to the next pane");
    assert_eq!(c.seen(), Vec::<Action>::new(), "and stops at the handler");
}

/// A component that records into a log owned by the test.
struct Shared(Rc<Spy>);

impl Shared {
    fn new(spy: Rc<Spy>) -> Self {
        Self(spy)
    }
}

impl Component for Shared {
    fn id(&self) -> &str {
        self.0.id()
    }

    fn focusable(&self) -> bool {
        self.0.focusable()
    }

    fn render(&self, paint: &Paint<'_>, buf: &mut Buffer) {
        self.0.render(paint, buf)
    }

    fn update(&mut self, action: &Action) -> Reaction {
        self.0.log.borrow_mut().push(action.clone());
        (self.0.react)(action)
    }

    fn value(&self) -> Option<String> {
        self.0.value.clone().map(str::to_owned)
    }
}

// --- emit -------------------------------------------------------------------

#[test]
fn a_component_emits_a_command_that_reaches_the_app_with_its_value() {
    let mut s = session(vec![
        Spy::new("a", ignore).boxed(),
        Spy::new("b", emit_run).value("firefox").boxed(),
        Spy::new("c", ignore).boxed(),
    ]);
    assert_eq!(
        s.dispatch(Action::Down),
        Outcome::Invoke {
            command: "run:firefox".to_owned(),
            value: Some("firefox".to_owned()),
        },
        "the emitting pane's value travels with the command"
    );
}

#[test]
fn an_emitted_action_is_dispatched_again_from_the_top() {
    let mut s = session(vec![
        Spy::new("a", emit_focus_next).boxed(),
        Spy::new("b", ignore).boxed(),
        Spy::new("c", ignore).boxed(),
    ]);
    s.dispatch(Action::Down);
    assert_eq!(s.focus(), Some("b"), "the emitted FocusNext was routed");
}

#[test]
fn quit_is_global_and_never_reaches_a_component() {
    let a = Rc::new(Spy::new("a", handle));
    let mut s = Session::new(
        spec(THREE),
        vec![Box::new(Shared::new(a.clone()))],
    )
    .expect("session");
    assert_eq!(s.dispatch(Action::Quit), Outcome::Quit);
    assert_eq!(a.seen(), Vec::<Action>::new());
}

// --- key maps (D8) ----------------------------------------------------------

const OVERRIDE: &str = r##"(
    window: (title: "t"),
    layout: (pane: "a"),
    panes: [
        (id: "a", keys: { "down": Some(Accept), "ctrl-j": None }),
        (id: "b"),
    ],
    keys: { "down": Some(Up), "ctrl-j": Some(Down), "esc": Some(Quit) },
)"##;

#[test]
fn a_pane_bind_overrides_the_global_one() {
    let s = Session::new(spec(OVERRIDE), vec![Spy::new("a", ignore).boxed()]).expect("session");
    let down = key(KeyCode::Down, Mods::NONE);
    assert_eq!(s.keymap().resolve("a", down), Some(&Action::Accept));
    assert_eq!(s.keymap().resolve("b", down), Some(&Action::Up));
}

#[test]
fn an_explicit_none_clears_an_inherited_bind() {
    let s = Session::new(spec(OVERRIDE), vec![Spy::new("a", ignore).boxed()]).expect("session");
    let ctrl_j = key(KeyCode::Char('j'), Mods::CTRL);
    assert_eq!(
        s.keymap().resolve("a", ctrl_j),
        None,
        "the pane says no and does not fall through"
    );
    assert_eq!(s.keymap().resolve("b", ctrl_j), Some(&Action::Down));
}

#[test]
fn effective_binds_are_the_global_ones_minus_what_the_pane_cleared() {
    let s = Session::new(spec(OVERRIDE), vec![Spy::new("a", ignore).boxed()]).expect("session");
    let binds = s.keymap().effective("a");
    let down = key(KeyCode::Down, Mods::NONE);
    assert_eq!(binds.get(&down), Some(&&Action::Accept));
    assert!(!binds.contains_key(&key(KeyCode::Char('j'), Mods::CTRL)));
    assert_eq!(binds.get(&key(KeyCode::Esc, Mods::NONE)), Some(&&Action::Quit));
}

#[test]
fn only_a_press_acts() {
    let down = KeyCode::Down;
    assert_eq!(key::on(Phase::Press, down, Mods::NONE), Some(Key::new(down, Mods::NONE)));
    assert_eq!(key::on(Phase::Repeat, down, Mods::NONE), None, "a hold must not re-fire");
    assert_eq!(key::on(Phase::Release, down, Mods::NONE), None);
}

#[test]
fn typing_reaches_the_focused_component_with_no_bind_for_it() {
    let a = Rc::new(Spy::new("a", handle));
    let mut s = Session::new(spec(THREE), vec![Box::new(Shared::new(a.clone()))]).expect("session");
    s.on_key(key(KeyCode::Char('f'), Mods::NONE));
    assert_eq!(a.seen(), vec![Action::Input('f')]);
    s.on_key(key(KeyCode::Char('f'), Mods::CTRL));
    assert_eq!(a.seen(), vec![Action::Input('f')], "ctrl-f is not typing");
}

const NO_KEYS: &str = r##"(
    window: (title: "t"),
    layout: (pane: "a"),
    panes: [(id: "a")],
)"##;

#[test]
fn a_document_with_no_keys_still_has_the_defaults() {
    let s = Session::new(spec(NO_KEYS), vec![Spy::new("a", ignore).boxed()]).expect("session");
    let control = s.keymap();
    assert_eq!(control.resolve("a", key(KeyCode::Down, Mods::NONE)), Some(&Action::Down));
    assert_eq!(control.resolve("a", key(KeyCode::Up, Mods::NONE)), Some(&Action::Up));
    assert_eq!(control.resolve("a", key(KeyCode::Enter, Mods::NONE)), Some(&Action::Accept));
    assert_eq!(control.resolve("a", key(KeyCode::Esc, Mods::NONE)), Some(&Action::Quit));
    assert_eq!(control.resolve("a", key(KeyCode::Tab, Mods::NONE)), Some(&Action::FocusNext));
    assert_eq!(
        control.resolve("a", key(KeyCode::Tab, Mods::SHIFT)),
        Some(&Action::FocusPrev)
    );
}

#[test]
fn a_document_can_clear_a_default() {
    let text = r##"(
        window: (title: "t"),
        layout: (pane: "a"),
        panes: [(id: "a")],
        keys: { "esc": None },
    )"##;
    let s = Session::new(spec(text), vec![Spy::new("a", ignore).boxed()]).expect("session");
    assert_eq!(s.keymap().resolve("a", key(KeyCode::Esc, Mods::NONE)), None);
    assert_eq!(
        s.keymap().resolve("a", key(KeyCode::Down, Mods::NONE)),
        Some(&Action::Down),
        "clearing one default leaves the rest"
    );
}

#[test]
fn a_key_string_that_is_not_a_key_names_its_path() {
    let err = UiSpec::parse(
        r##"(
            window: (title: "t"),
            layout: (pane: "a"),
            panes: [(id: "a")],
            keys: { "ctrl-": Some(Quit) },
        )"##,
        Format::Ron,
    )
    .expect_err("ctrl- is not a key")
    .to_string();
    assert!(err.contains("keys[\"ctrl-\"]"), "{err}");
    assert!(err.contains("empty part"), "{err}");
}

#[test]
fn keys_round_trip_through_their_spelling() {
    for spelling in [
        "ctrl-j",
        "esc",
        "shift-pageup",
        "alt-enter",
        "f5",
        "ctrl-alt-shift-x",
    ] {
        let key = Key::parse(spelling).expect(spelling);
        assert_eq!(key.to_string(), spelling);
    }
    assert_eq!(
        Key::parse("J").expect("J").to_string(),
        "shift-j",
        "a capital letter is shift plus the letter"
    );
    assert!(Key::parse("ctrl-shift").expect_err("not a key").contains("not a key"));
}

// --- render -----------------------------------------------------------------

#[test]
fn each_pane_is_painted_in_its_own_rect_and_focus_is_passed_down() {
    let s = session(vec![
        Spy::new("a", ignore).boxed(),
        Spy::new("b", ignore).boxed(),
        Spy::new("c", ignore).boxed(),
    ]);
    let area = Rect::new(0, 0, 12, 3);
    let mut buf = Buffer::empty(area);
    s.render(area, &mut buf);

    let row = |y: u16| {
        buf.cell((0, y))
            .map(|cell| cell.symbol().to_owned())
            .unwrap_or_default()
    };
    assert_eq!(row(0), "a", "first pane in the first row");
    assert_eq!(row(1), "b");
    assert_eq!(row(2), "c");
    assert_eq!(
        buf.cell((1, 0)).map(|cell| cell.symbol().to_owned()).as_deref(),
        Some("*"),
        "the focused pane is told it has focus"
    );
}

#[test]
fn two_components_cannot_claim_one_pane() {
    let err = match Session::new(
        spec(THREE),
        vec![Spy::new("a", ignore).boxed(), Spy::new("a", ignore).boxed()],
    ) {
        Ok(_) => panic!("duplicate ids must be refused"),
        Err(e) => e.to_string(),
    };
    assert!(err.contains("two components claim pane"), "{err}");
}

#[test]
fn delete_reaches_the_query_like_backspace() {
    // The query has no cursor movement yet, so Delete drops the last
    // character; the point is that the key is decoded, routed and answered
    // instead of vanishing.
    let spec = spec(
        r#"(window: (title: "t"), layout: (pane: "q"), panes: [(id: "q", kind: Input, title: "q")])"#,
    );
    let mut session = Session::from_spec(spec).expect("the document is valid");
    session.dispatch(Action::Input('a'));
    session.dispatch(Action::Input('b'));
    let outcome = session.on_key(Key::new(KeyCode::Delete, Mods::NONE));
    assert_eq!(outcome, Outcome::Continue);
    let query = session.pane::<InputLine>("q").and_then(|input| input.value());
    assert_eq!(query, Some("a".to_owned()));
}
