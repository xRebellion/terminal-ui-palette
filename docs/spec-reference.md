# terminal-ui-palette — spec reference

Everything a picker needs is in one RON document (JSON works too, same fields). This page is written to be sufficient on its own: a new picker should be writable from here without opening the source. The last checkable of item 11 is exactly that — `layouts/wifi.ron` was written from this page.

## The smallest useful document

```ron
(
    window: (title: "pick"),
    layout: (pane: "m"),
    panes: [(id: "m", kind: Menu, title: "items")],
)
```

Run it with `tuipalette --spec that.ron`, items arriving on stdin.

## Top level

| field | type | default | meaning |
|---|---|---|---|
| `window` | object | required | `title` — used as the terminal/window title |
| `theme` | object | built-in dark theme | colour tokens, below |
| `layout` | node | required | the pane tree, below |
| `panes` | list | `[]` | pane definitions, below |
| `keys` | map | `{}` | global key bindings, below |
| `host` | object | `{}` | a hint to the launcher, never used by the library |

Unknown fields are an error, at every level, in every object. A typo fails the document instead of silently parsing as a default.

### `host`

| field | type | default | meaning |
|---|---|---|---|
| `width` | integer | none | window width in cells |
| `height` | integer | none | window height in lines |
| `class` | string | none | the window class an adapter asks its terminal for, so a compositor rule can match it (use `tuipalette` for the shipped Hyprland rule) |

`tuipalette --print-geometry` prints `widthxheight`; `--print-class` prints `class`. Both exit 0 without opening a terminal, which is how `scripts/tuipalette-floating.sh` sizes its window.

## `theme`

| field | default | meaning |
|---|---|---|
| `accent` | `#89b4fa` | focus borders, the selected row's text |
| `selected_bg` | `#313244` | background of the selected row |
| `text` | `#cdd6f4` | ordinary text |
| `dim` | `#6c7086` | unfocused borders, status hints |
| `icon_placeholder` | `·` | fills an icon slot the provider left empty |

Colours are `#rrggbb` and nothing else — no names, no 256-colour indexes. `tuipalette --theme other.ron` replaces the document's theme; the file is the same RON object, a bare `(accent: "#ff0000", ...)`.

## `layout`

One object: either a **leaf** (`pane: "id"`) or a **split** (`dir` + `children`).

| field | type | default | meaning |
|---|---|---|---|
| `pane` | string | none | name of the pane this leaf shows |
| `dir` | `Vertical` \| `Horizontal` | none | split direction: `Vertical` stacks children top to bottom |
| `sizes` | list of size strings | all `fill` | one entry per child |
| `spacing` | integer | `0` | blank cells between children |
| `children` | list of nodes | `[]` | the children of a split |

### Size grammar

| form | meaning |
|---|---|
| `"3"` | exactly three cells |
| `"20%"` | 20% of the parent, rounded down |
| `"fill"` | shares what is left equally with other `fill` children |
| `"min:5"` | at least 5 cells |
| `"max:10"` | at most 10 cells |

Each size string maps onto one constraint of ratatui's layout solver — `N` → length, `N%` → percentage, `fill` → fill, `min:N`/`max:N` → a floor and a ceiling — and the solver distributes the cells: lengths and percentages take theirs first, `fill` children share what is left, and the last child absorbs the rounding. Omitting `sizes` entirely gives every child `fill`.

Order matters for focus: the focus ring follows the order the panes are **drawn**, i.e. depth-first through the tree, and `tab`/`shift-tab` walk that order.

## `panes`

| field | type | default | meaning |
|---|---|---|---|
| `id` | string | required | the name `layout.pane` refers to |
| `kind` | `Text` \| `Input` \| `Menu` \| `Status` \| `Spacer` | `Text` | what it draws |
| `title` | string | `""` | frame title; for `Input`, the prompt |
| `items` | list of strings | `[]` | for `Status`: the hints, joined with a gap |
| `icon_width` | integer | none | icon column width in cells; `None` means no column |
| `keys` | map | `{}` | bindings only for this pane |

### What each kind draws

| kind | draws | reacts to |
|---|---|---|
| `Text` | a framed block of static lines, from `items` | nothing |
| `Input` | the query line with a cursor, framed | typing, `Backspace` |
| `Menu` | the item list: marker, optional icon slot, display, match highlight, a derived scroll window | `Up`/`Down`, `Accept` |
| `Status` | one row of hints, no frame | nothing |
| `Spacer` | nothing, holding its cells | nothing |

The `Menu` clamps at both ends (no wrap), never scrolls past its window, and highlights the characters that matched the query.

## `keys`

A map from key spelling to action. Values are either an action name or `None`.

```ron
keys: {
    "esc": Some(Quit),
    "ctrl-j": None,        // clears an inherited binding
    "f5": Some(FocusNext),
}
```

### Key spellings

`mod-key`, with any of `ctrl`/`control`, `alt`/`meta`/`option`, `shift` in front, in any order. The key is one of: a single character (`a`, `1`, `/`), or `esc`/`escape`, `enter`/`return`, `tab`, `backspace`, `delete`/`del`, `space`, `up`, `down`, `left`, `right`, `home`, `end`, `pageup`/`pgup`, `pagedown`/`pgdn`, `insert`, `f1`…`f24`. A named key is case-insensitive; a bare capital means shift (`K` is `shift-k`).

### Actions

| action | what the session does |
|---|---|
| `Quit` | exit, code 130 |
| `Accept` | take the focused pane's value, exit 0 |
| `Up` / `Down` | move the menu selection |
| `Backspace` | drop the last character of the query |
| `Delete` | same for now: the query has no cursor to move, so it trims the end |
| `FocusNext` / `FocusPrev` | next/previous pane in draw order |

`Input(char)` and `Invoke(value)` exist at runtime only; a document cannot name them.

### Merge rules

Three layers, later wins: the **built-in default map**, then the document's global `keys`, then the pane's own `keys`. A key mapped to `None` at any layer removes it for that scope — that is how a pane takes `enter` away from the global binding. Typing always falls through to the focused `Input` when no binding claims the character.

Built-in defaults (present even when the document says nothing):

| key | action |
|---|---|
| `up`, `ctrl-p`, `ctrl-k` | `Up` |
| `down`, `ctrl-n`, `ctrl-j` | `Down` |
| `enter` | `Accept` |
| `tab` | `FocusNext` |
| `shift-tab` | `FocusPrev` |
| `esc` | `Quit` |

## Items on stdin

Default protocol, one item per line, up to three TAB-separated fields:

| field | required | notes |
|---|---|---|
| display | yes | what the row shows; control characters become spaces |
| value | no | what `Accept` returns, byte for byte; defaults to the display |
| icon | no | the glyph for the icon slot; everything after the second tab |

A line without a tab is both display and value, which is what makes `printf 'a\nb\n' | tuipalette` a dmenu.

`--in json` reads one object per line instead: `{"display": "…", "value": "…", "icon": "…"}`, same rules.

Items are read on a thread as they arrive, so a slow provider fills the list progressively rather than holding the first frame back.

## Output

One line on stdout: the chosen value (plus the query first with `--print-query`). `--out json` prints `{ "query": "…", "value": "…" }`. `--out-file path` writes the same value to a file, which is how a floating window — whose stdout is the terminal it draws on — hands its answer back.

Exit codes: `0` accepted, `130` cancelled, `1` the document is wrong (with the path and line), `2` unusable host or flags.

## Command line

| flag | meaning |
|---|---|
| `--spec <file>` | the document (required) |
| `--format ron\|json` | how to read it; inferred from the extension by default |
| `--check` | validate and describe, then exit — `1` if the document is wrong |
| `--inline` | draw in the invoking terminal instead of the alternate screen (opt-in; without it the picker takes the whole screen) |
| `--height <n>` | with `--inline`, take `n` lines at the bottom |
| `--prompt <text>` | replace the query pane's title |
| `--theme <file>` | read the theme from this file instead |
| `--icons` / `--no-icons` | force/remove a two-cell icon column in menu panes |
| `--no-sort` | keep the provider's order instead of ranking (rofi's `-no-sort`) |
| `--accept-query` | Enter with no match answers with the typed text (rofi's custom input) |
| `--message <text>` | replace the status row with this line (rofi's `-mesg`) |
| `--in line\|json` | stdin protocol |
| `--out line\|json` | stdout shape |
| `--print-query` | print the query before the value |
| `--print-geometry` / `--print-class` | print the host hint and exit |
| `--out-file <path>` | write the value there instead of stdout |

Environment: `TUIPALETTE_TIMING=1` prints the startup phases to stderr; `TUIPALETTE_DEBUG_KEYS=1` prints each decoded key.

## Writing a new picker

1. **Decide the shape.** A launcher is `Input` over `Menu` over `Status`. A dmenu is `Input` over `Menu`. A chooser with a preview is a horizontal split: `Menu` left, `Text` right.
2. **Write the document.** Copy the nearest shipped layout (`layouts/`) and change the panes, the sizes and the tokens.
3. **Write the provider.** A script that prints the protocol on stdout; keep the lookup out of the library. `printf 'display\tvalue\ticon'`.
4. **Check it.** `tuipalette --check --spec your.ron` prints `ok — N panes, M bindings`, or the field path that is wrong.
5. **Run it.** `your-provider | tuipalette --spec your.ron`, or `--inline --height 8` from a terminal.

## Adding a `Component`

A component renders into a plain buffer and answers actions with a `Reaction`; it never sees the terminal, so it is testable headless (D1, D5). The trait, in full:

```rust
use tuipalette_core::ui;
use tuipalette_core::{Action, Buffer, Component, Paint, Reaction, Style};

pub struct Gauge {
    id: String,
    value: u8,
}

impl Component for Gauge {
    /// Must match a pane id in the document.
    fn id(&self) -> &str { &self.id }

    /// May the focus ring land here? Labels answer false.
    fn focusable(&self) -> bool { true }

    /// Draw into `buf`, clipped to `paint.area`. `ui::frame` returns the inside,
    /// `ui::put` writes one clipped line, `ui::fit`/`ui::tail` pad and truncate by cell width.
    fn render(&self, paint: &Paint<'_>, buf: &mut Buffer) {
        let inner = ui::frame(buf, paint.area, "gauge", paint.focused, paint.theme);
        let style = Style::default().fg(paint.theme.accent_color());
        ui::put(buf, inner.x, inner.y, inner.width, &self.value.to_string(), style);
    }

    /// `Ignored` lets the router keep looking; `Handled` stops it; `Emit(action)`
    /// re-dispatches from the top (that is how a component moves focus or asks
    /// the app for something via `Action::Invoke`).
    fn update(&mut self, action: &Action) -> Reaction {
        match action {
            Action::Down => { self.value = self.value.saturating_sub(1); Reaction::Handled }
            _ => Reaction::Ignored,
        }
    }

    /// What `Accept` hands back when this pane has focus.
    fn value(&self) -> Option<String> { None }
}
```

Register it in `components::from_pane` next to the built-ins, and give the document a way to
name it — a new `Kind` variant if it is a new shape, or a `Pane` field with
`#[serde(default)]` if the existing kinds just need more data. Keep `deny_unknown_fields` on, so a
typo stays an error.

Because `Component: Any`, the picker can reach a built-in by type with
`session.pane_mut::<Menu>()` — that is how `Picker` fills a menu without a trait object cast.

Test it the way the built-ins are tested: render into `Buffer::empty(area)`, read the text
back with `ui::screen`, assert the cells you care about, and pin the text with a
snapshot.

## A worked example, written from this page

`layouts/wifi.ron` and `scripts/wifi.sh` are a picker built from this reference alone:
a query line, a network list, two hints, monokai tokens, a 50x12 card. The provider prints
the three-field protocol and knows nothing about the library.

```
$ tuipalette --check --spec layouts/wifi.ron
layouts/wifi.ron: ok — 3 panes, 2 bindings
$ scripts/wifi.sh --fake | tuipalette --spec layouts/wifi.ron   # typed "cafe", enter
cafe
```
