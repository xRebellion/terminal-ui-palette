# 05 — Built-in components and snapshot tests

Detail for item 05 of `docs/PLAN.md`.

## Five components, one mapping

| `kind` | component | focuses | owns |
|---|---|---|---|
| `Input` | `InputLine` | yes | the text the user types, its cursor, what is handed back |
| `Menu` | `Menu` | yes | items, selection, scroll window, icon column |
| `Text` | `TextPane` | no | static lines in a frame |
| `Status` | `StatusLine` | no | one row of hints |
| `Spacer` | `Spacer` | no | nothing, deliberately |

`from_pane(pane)` is the only place that decides which is which, and `Session::from_spec` calls it for every pane. `Status` and `Spacer` are new wire names for `kind`; a document that does not mention them parses exactly as before.

## What the rendering primitives do

Core still does not depend on the `ratatui` facade, so the three things every built-in needs are here instead:

- `ui::frame(buf, area, title, focused, theme) -> Rect` — a rounded border in the accent colour when focused and the theme's dim colour when not, the title clipped, and the inside rect returned. Too small for a frame means an empty rect and nothing drawn, never a panic.
- `ui::put(buf, x, y, width, text, style)` — one line, clipped to a cell budget by `Buffer::set_stringn`, so wide glyphs cannot overflow into the next pane.
- `ui::screen(&Buffer) -> String` — the buffer read back as text with trailing blanks trimmed: what the snapshots capture.

## Theme gained a fourth token

`dim` (default `#6c7086`) is optional, so every document written before this item still loads. It is what an unfocused border and the status line are painted in. Item 11 documents the final surface; item 03's spec describes the three tokens it shipped with.

## The menu's rules

- **Clamp, do not wrap.** `up` at the top and `down` at the bottom stay put. A picker that wraps loses the sense of an end; a bound key can wrap if a document wants that later.
- **The window is derived, not stored.** `window(rows)` returns `selected + 1 - rows` once the selection passes the first page, so there is no offset to keep in sync when items stream in (item 06) or shrink.
- **The selection is clamped whenever the items change.** `set_items` is the hook item 06 will use; after it, `selected` is never past the end.
- **The icon column is reserved, never looked up.** `icon_width` on the pane allocates the cells; the glyph comes from the `Item` the caller supplies, and an absent glyph leaves the column blank rather than shifting the text.
- **Accept emits the value.** `Action::Accept` on a menu answers `Reaction::Emit(Action::Invoke(item.value))`, and `value()` returns the selected item's value, so the router's `Outcome::Invoke` carries both.

## Snapshots, and what they do not prove

The plan said snapshots against `TestBackend`. They capture `Buffer::empty(area)` instead: the component API takes a buffer, a `TestBackend` wrap adds a layer that changes no cell, and core already has the buffer type. The property that matters is kept — a visual change is a failing test.

Text cannot prove colour, so one test asserts the selected row's background cell by cell against the theme. And text cannot faithfully show a **double-width glyph**: a two-cell symbol occupies one buffer cell plus a shadow cell, so a snapshot reads one column narrow. That is pinned by its own geometry test — glyph in the first cell of its column, marker still on its column — and handed to item 15, which owns glyph width.

The launcher fixture at 60×14, three keystrokes into the query and three rows down:

```
╭─ search ─────────────────────────────────────────────────╮
│fir▏                                                      │
╰──────────────────────────────────────────────────────────╯
╭─ applications ───────────────────╮╭─ preview ────────────╮
│  firefox                         ││firefox               │
│  kitty                           ││internet browser      │
│  code                            ││130 MB                │
│› herdr                           ││                      │
│  spotify                         ││                      │
│  discord                         ││                      │
│  slack                           ││                      │
│  telegram                        ││                      │
╰──────────────────────────────────╯╰──────────────────────╯
enter run  tab next  esc quit
```

## Evidence

```
cargo test -p tuipalette-core --test components
  14 passed; 0 failed

cargo test --workspace
  tuipalette      unit        2 passed
  tuipalette      check       5 passed
  tuipalette-core components 14 passed
  tuipalette-core dispatch   14 passed
  tuipalette-core spec       16 passed
```

Snapshots live in `crates/tuipalette-core/tests/snapshots/` and are accepted with `INSTA_UPDATE=always cargo test -p tuipalette-core --test components` — no `cargo-insta` needed.

## Not in this item

Filtering as you type (06) — `InputLine` keeps the text and nothing reads it yet. Horizontal scrolling inside the query line: a value wider than the pane shows its end. And centring an icon inside its column, which is the width question item 15 inherits.
