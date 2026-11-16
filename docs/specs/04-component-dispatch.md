# 04 — Component trait, focus ring, action router, key maps

Detail for item 04 of `docs/PLAN.md`.

## The component

```rust
pub trait Component {
    fn id(&self) -> &str;
    fn focusable(&self) -> bool { true }
    fn render(&self, paint: &Paint<'_>, buf: &mut Buffer);
    fn update(&mut self, _action: &Action) -> Reaction { Reaction::Ignored }
    fn value(&self) -> Option<String> { None }
}

pub struct Paint<'a> { pub area: Rect, pub focused: bool, pub theme: &'a Theme }

pub enum Reaction { Ignored, Handled, Emit(Action) }
```

Rendering takes `&self`, so a component can be drawn from a shared reference and a snapshot test never needs to own it. `Paint` carries the three things every component wants and none of the things it does not: where it is, whether it has focus, and the theme. There is deliberately no width setting, no terminal handle and no frame.

## The router, in order

1. `Quit` → `Outcome::Quit`, `FocusNext`/`FocusPrev` → move the ring. Global, so a bind cannot be shadowed by a component than wants the key for something else.
2. The focused component's `update`, then every other component once in declaration order, stopping at the first `Handled` or `Emit`. Bubble order is fixed, so behaviour does not depend on which pane happens to hold focus.
3. `Emit(Action::Invoke(command))` becomes `Outcome::Invoke { command, value }` — the emitting component's `value()` travels with it, because the pane that accepts is not always the pane with focus.
4. Any other `Emit` is dispatched again from the top, up to 8 deep, so a component can move focus or ask for a global action without reaching for the session.

Keys that no bind claims fall through to the focused component as `Action::Input(char)` when they are an unmodified character — typing works without a bind per letter. Everything else is absorbed.

## Only a press acts

Hosts decode key events; `key::on(phase, code, mods) -> Option<Key>` is the only constructor they should use, and it returns `None` for `Repeat` and `Release`. Without that, holding a key re-fires a toggle and a release fires a bind a second time. The rule lives in core so it is testable without a terminal; a host that builds `Key` by hand loses it.

## Key spelling

Modifiers are `ctrl`, `alt` and `shift`, spelled before the key and separated by `-`: `ctrl-j`, `alt-enter`, `ctrl-alt-shift-x`. Named keys are one word — `pageup`, not `page-up` — because the hyphen is the separator. `Display` prints the canonical spelling, so a key can be round-tripped.

A capital letter means shift: `J` and `shift-j` are one bind, stored as `shift` + `j`. Terminals disagree about whether an uppercase character arrives as `shift+j` or as `J`, and this is where that disagreement is resolved once.

A key string that is not a key is a spec error at load, with its path — never a bind that silently never fires:

```
/tmp/badkey.ron:  keys["ctrl-"]: `ctrl-` has an empty part — expected a modifier and a key, e.g. `ctrl-j`
/tmp/badkey2.ron: panes[0].keys["hyper-x"]: `hyper` is not a modifier — expected ctrl, alt or shift in `hyper-x`
```

## Key-map merge (D8)


Resolve order for a key while `pane` has focus: the pane's own map, then the global map. A pane entry of `Some(action)` wins; `None` clears, and does *not* fall through to the global bind. `effective(pane)` returns the merged view with cleared binds removed — what a help footer or the reference doc would print.

## The focus ring

Built from `layout::pane_order` — the draw order, not the order `panes` lists them — keeping only components that answer `focusable()`. Labels are stepped over, the ring wraps both ways, and a pane that is not drawn is not on it. Bubbling still reaches non-focusable components, because knowing an action and taking focus are different questions.

## Plan correction

The checkable on the plan line was `cargo test -p tuipalette-core -- dispatch`. A bare word there is a *name* filter, not a file filter, so it ran one test whose name happened to contain "dispatch". The suite is now run as `cargo test -p tuipalette-core --test dispatch`, and the plan line says so.

## Evidence

```
cargo test -p tuipalette-core --test dispatch
  14 passed; 0 failed

cargo test --workspace
  tuipalette      unit        2 passed
  tuipalette      check       5 passed
  tuipalette-core dispatch   14 passed
  tuipalette-core spec       16 passed
```

The 14 cover: focus skipping a label, wrapping both ways, bubbling stopping at the first handler, emit carrying the emitting pane's value, an emitted action re-dispatched from the top, quit never reaching a component, pane override, explicit `None` unset, `effective` merging, press-only, typing fallthrough, a bad key naming its path, key round-tripping through `Display`, and focus reaching the render pass.

## Not in this item

A `focusable` field on `Pane` — today the component decides, which is enough until someone needs to disable a pane from the document. Building components from panes (`from_pane`) is item 05's, and border chrome follows it.
