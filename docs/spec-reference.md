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
| `window` | object | required | `title` — used as the terminal/window title; `frame` — draw one box around the whole picker (default `false`); `mouse` — capture the mouse for hover, click and wheel (default `false`, see below); `padding: (x: N, y: N)` — blank cells between the edge (or the frame) and the layout, default none. With `frame: true` the title is drawn on the frame |
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
| `scrollbar` | `#585b70` | the scrollbar's thumb: its cell background, or `█`'s colour in the `Glyph` style |
| `hover_bg` | `#313244` | the row under the pointer, with `window: (mouse: true)`; quieter than `selected_bg`, because hovering is not choosing |
| `scrollbar_bg` | `#45475a` | the track behind it: the cell background of the column the thumb is not on, or `░`'s colour in `Glyph` |

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
| `kind` | `Text` \| `Input` \| `Menu` \| `Status` \| `Spacer` \| `Detail` \| `Counter` \| `Rule` | `Text` | what it draws |
| `wrap` | bool | `false` | for `Detail`: wrap long lines between words (inside a word only when it is wider than the pane) instead of cutting them |
| `title` | string | `""` | frame title; for an unframed `Input`, the prompt drawn in front of the query |
| `items` | list of strings | `[]` | for `Status`: the hints, joined with a gap; for `Text`: the lines |
| `icon_width` | integer | none | icon column width in cells; `None` means no column |
| `keys` | map | `{}` | bindings only for this pane |
| `from` | string | the first menu | for `Detail`: the `Menu` whose selection it follows |
| `frame` | bool | `true` | `false` draws the pane without its box |
| `scrollbar` | bool | `false` | for `Menu`: draw a scrollbar in the pane's last column, taking one cell from the text |
| `scrollbar_style` | `Background` \| `Glyph` | `Background` | for `Menu` with `scrollbar: true`: a tinted cell background, or the `░`/`█` glyph column |
| `format` | string | `{shown}/{total}` | for `Counter`: how the two numbers are written; `{shown}` and `{total}` are the only placeholders |
| `align` | `Left` \| `Center` \| `Right` | `Left` | for `Counter`: which edge of its rect the number hugs |
| `click` | `Select` \| `Accept` | `Select` | for `Menu` with `window: (mouse: true)`: what one click on a row does — select it (a second click accepts), or select and accept at once, like rofi |

### What each kind draws

| kind | draws | reacts to |
|---|---|---|
| `Text` | a framed block of static lines, from `items` | nothing |
| `Input` | the query line with a cursor; framed, or one bare row with `frame: false` | typing, `Backspace` |
| `Menu` | the item list: marker, optional icon slot, display, match highlight, a derived scroll window, and with `scrollbar: true` a proportional thumb in the last column, in the `scrollbar_style` the pane asks for | `Up`/`Down`, `Accept` |
| `Status` | one row of hints, no frame | nothing |
| `Spacer` | nothing, holding its cells | nothing |
| `Detail` | the selected row's `detail` text, line by line, clipped to the pane | nothing — the picker refills it when the menu's selection or rows change |
| `Rule` | a divider: one `dim` line across a wide rect or down a tall one, centred — a three-cell column is a blank, a line, a blank | nothing |
| `Counter` | `shown/total` — the rows the query left, over the rows that arrived — written in `format` and placed by `align`, on the pane's first row | nothing — the picker writes both numbers after every refilter |

The `Menu` clamps at both ends (no wrap), never scrolls past its window, and highlights the characters that matched the query.

### The mouse

`window: (mouse: true)` captures the mouse for the run. It is off by default because capture takes text selection away from the terminal — turn it on for a picker that owns its own window (the shipped `rofi-like` and `clipboard` do), not for one that runs inline in somebody's shell. With it on: moving the pointer tints the row under it with `hover_bg` without selecting it; a click selects the row and a click on the row already selected accepts it, so a double click accepts any row — or, with `click: Accept` on the `Menu`, one click selects and accepts, as rofi does; the wheel moves the selection three rows a notch; a click on another pane (the query) gives it focus. Without it, the picker ignores mouse events even if the terminal sends some. Capture is released on every exit, including an error or a panic.

### The scrollbar and the count

`scrollbar: true` on a `Menu` draws the thumb and track in the pane's last column —
inside the frame when the pane is framed — and gives the text one cell fewer. It
appears only when there are more rows than cells, and its size is the share of the
track the window covers, so twelve rows in a two-hundred-row list is a one-cell bar
in the right place. A document that does not ask for it keeps every column for text.

`scrollbar_style` decides how the same bar is painted, and never where it goes:

- `Background` (the default) tints the cells — `scrollbar_bg` behind the track,
  `scrollbar` behind the thumb, no glyph, so the column reads as a coloured stripe
  the way tmux's and helix's scrollbars do. It paints over the selected row's
  background in that one column, which is what keeps the bar legible.
- `Glyph` draws `░` and `█` in the same two colours as foregrounds. It is the
  style to pick when the screen is read back as text — a snapshot, a piped
  `capture-pane`, a terminal recorder — because a background colour is not text.

A `scrollbar_style` on a pane that does not also say `scrollbar: true`, or on a
pane that is not a `Menu`, is a spec error naming the field.

A `Counter` is a label the picker fills: it cannot hold focus, and its numbers are
not something a document writes. The default reads `3/49`;
`format: "{shown} of {total}"` reads `3 of 49`, and `align: Right` puts it against
the right edge of its pane, which is where fzf's info line puts it. `total` is every
item the provider sent, whether or not it is still on screen. A `format` containing
anything but those two placeholders is a spec error, and so are `format`/`align` on
a pane that is not a `Counter`. Give the pane room for the largest count it will
show (`250/250` is seven cells); a pane too narrow drops text from the *left* and
marks the cut with `…`, so the total is never the number that loses digits.

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

Default protocol, one item per line, up to five TAB-separated fields:

| field | required | notes |
|---|---|---|
| display | yes | what the row shows; control characters become spaces |
| value | no | what `Accept` returns, byte for byte; defaults to the display |
| icon | no | the glyph for the icon slot; the third field |
| search | no | text the matcher sees and the row never shows; the fourth field |
| detail | no | text a `Detail` pane shows while the row is selected; the fifth field |

A line without a tab is both display and value, which is what makes `printf 'a\nb\n' | tuipalette` a dmenu. An empty field is an absent one, so `name<TAB>exec<TAB><TAB>words` has no icon and searchable words.

The search field is how a provider lends the matcher words the row should be found by but not drawn with. rofi's drun does the same: it matches Name, GenericName, Exec, Categories and Keywords while showing only the Name, which is why typing `seahorse` there finds "Passwords and Keys". `scripts/apps.sh` emits those five fields in this slot.

Ranking (D20): a row whose display holds the query **literally** ranks above any scattered match, in four tiers — the display is the query, starts with it, has a word starting with it, contains it — and is highlighted where it matched. Then come the matcher's fuzzy hits, best score first. Last come rows found only through their value or search text: real hits, never highlighted, because positions index the display. Inside a tier the order the rows arrived in is kept, so a provider that sends its list most-used first (`scripts/usage.sh`) is ranked by usage *after* typing too. Smart case throughout: a query with an upper-case letter matches case exactly. `kn` therefore finds Ar**kn**ights before **K**o**n**sole.

`--in json` reads one object per line instead: `{"display": "…", "value": "…", "icon": "…", "search": "…", "detail": "…"}`, same rules. Expressing a detail longer than one line is the reason it exists: in the TAB protocol a newline would end the item.

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
