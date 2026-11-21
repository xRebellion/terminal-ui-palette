# 22 — A scrollbar, as a pane option

Last updated: 2026-11-18

The defect is a list you cannot place yourself in. `scripts/run-command.sh --list`
answers 4669 lines; the picker's window shows the selection and nothing else, so
"how much is left" and "how far in am I" were questions the screen did not answer.
fzf's `--scrollbar` is the reference: off by default (which is why the original
capture had none), one column, proportional thumb.

**Revised 2026-11-18:** the drawing was a glyph because a background is invisible to
a text snapshot; the user asked for the tinted-cell style as the default and a
configurable style, so both exist now and the invisibility problem moved into the
test. The geometry, the two tokens and the "off unless asked for" rule did not move.

## How modern terminal UIs draw one — the research

Everything below was read or run **on this machine on 2026-11-18** unless the line
says otherwise; three claims come from search results, not from source.

- **fzf 0.74.5** — `fzf --help` and `man fzf`: `--scrollbar=CHAR1[CHAR2]`, one
  character, default `│` (`:` under `--no-unicode`), and `--no-scrollbar` is a
  synonym for `--scrollbar=''`. Measured with `seq 1 200 | fzf --scrollbar` in a
  60x12 tmux pane and `capture-pane -e`: the column carries a glyph **only where
  the thumb is** — the thumb is `│` in the list's own colour (`38;5;59`) — and the
  track below and above it is blank cells. So fzf's "scrollbar character" is the
  thumb, and the track is nothing at all.
- **lazygit 0.66.2** — measured the same way: `▐` (U+2590 RIGHT HALF BLOCK) in
  the last column of a panel, `38;2;166;173;200`, five cells of a 23-row panel
  body; every other row of that column is the panel's `│` border. A **half**
  block, so the bar is half a cell wide and the track costs no extra cell.
- **tmux 3.8** — `man tmux`: `pane-scrollbars off|modal|on|auto-hide` — "a
  character based scrollbar appears on the left or right of each pane. A filled
  section of the scrollbar, known as the *slider*, represents the position and size
  of the visible part of the pane content." Its style is explicit: "The foreground
  colour is used for the slider, the **background** for the rest of the scrollbar.
  The *width* attribute sets the width of the scrollbar and the *pad* attribute the
  padding between the scrollbar and the pane." `on` narrows the pane by the bar's
  width; `modal` and `auto-hide` overlay it and hide it again after
  `pane-scrollbars-timeout`. Style elements `ScrollbarSlider`, `ScrollbarUp` and
  `ScrollbarDown` colour the slider and the cells above and below it.
- **helix** — `helix-term/src/ui/popup.rs` (upstream, read in full): one column at
  `inner.right() - 1`, drawn only `if !fits`; `scroll_height =
  win_height.pow(2).div_ceil(len)`, `scroll_line = (win_height - scroll_height) *
  scroll / max(1, len - win_height)` — the same two proportions this pane
  computes, rounded differently; glyph `▌`/`▐` (half block), and where the popup has no border the
  track takes the theme's **background** colour instead of a glyph
  (`cell.set_fg(scroll_style.bg)`).
- **ratatui 0.30 / ratatui-widgets 0.3.2** — the library this project draws into.
  `Scrollbar` is a glyph widget: `thumb_symbol` (default `█`, from
  `Set::DOUBLE_VERTICAL`), `track_symbol: Option<&str>` (default `║`; `None`
  leaves the cells untouched so a background can show through), optional
  `begin`/`end` arrows (`▲`/`▼`), each with its own `Style` — so even ratatui's
  own widget is built to be either glyphs or a styled column.
- **neovim 0.12.6** — no scrollbar in the TUI: no such option in the runtime docs;
  `gui.txt`'s thumb/trough is the Vim *GUI*'s scrollbar; `api-ui-events.txt` says
  a UI "could use a different font size per-window. Or reserve space around the
  border of the window for its own elements, such as scrollbars from the UI
  toolkit", and the position is exported (`win_viewport`) rather than drawn.
  `api.txt`: floating windows "don't support some widgets like scrollbar".
- **zellij** — no pane scrollbar; `zellij-org/zellij#4567` is an open feature
  request for one. *(Search result, not first-hand.)*
- **k9s / tview** — none in the library; `rivo/tview#950`, "Scroll bars for tables
  and textviews", asks for it. *(Search result, not first-hand.)*
- **nvim-scrollbar** — the plugin Neovim users reach for instead: a floating window
  of `virt_text` marks beside the buffer, hidden while the buffer fits, carrying
  diagnostics and git hunks next to the thumb. *(README read; its glyphs were not
  verified one by one — that part is memory.)*

The recurring idioms, each one present in at least two of the above:

1. **one column at the list's right edge**, inside the pane's own frame;
2. **thumb as a full or half block** (`█`, `▐`), or the same glyph recoloured
   (fzf), or a tinted cell with no glyph at all (tmux, helix without a border);
3. **a dimmed track** — a background, a dim glyph, or nothing at all — never as
   loud as the thumb;
4. **auto-hide when the content fits** (helix, fzf, tmux `auto-hide`);
5. the column is either **reserved** (tmux `on`, helix) or **overlaid**
   (tmux `modal`).

Which of them fits a ratatui-style cell grid: all of them, because a cell *is*
`{symbol, fg, bg, modifiers}` and every idiom above is one of those four fields.
The tinted cell costs one field and no glyph, which is why tmux and helix chose it
and why it is the default here; a glyph costs one `set_symbol`, which is the only
thing a text snapshot can see, which is why it stays as an option. Half-block
precision was **not** taken: the geometry here is whole cells, and the point of
this revision was that both styles draw the *same* geometry — a half-block style
would need a second, half-resolution geometry, so it is recorded as research
rather than implemented.

## The shape it took

`scrollbar: true` on a `Menu` pane. Default `false`, so no shipped document
changed shape and no existing snapshot moved.

- The bar is the pane's **last column** — the last cell inside the frame when the
  pane is framed — and the text gets one cell fewer. Nothing is reserved when the
  pane is narrower than four cells: a bar needs a column and the text needs a row.
- Shown only when `rows > visible rows`. A full list draws no bar and loses no
  column.
- Thumb length is the share of the track the window covers, rounded, never below
  one cell: `length = round(track² / total)`, clamped to `1..=track`.
- Thumb position is where the window sits in the list:
  `top = round((track - length) * start / (total - track))`, with
  `start = Menu::window(track)`. Twelve rows in a 200-row list is one cell; 120 of
  those is a bar covering a fifth of the track.

Two tokens, validated like the rest: `Theme.scrollbar` (thumb) and
`Theme.scrollbar_bg` (track). Colours are `#rrggbb` or the document is wrong.

## Two styles, one bar

`scrollbar_style: Background | Glyph` is a pane field; omitted means
`Background`. It decides what goes into the cells and nothing else — both styles
call the same `Menu::thumb(track)`, take the same column and paint the same rows.

| style | track cell | thumb cell | visible to |
|---|---|---|---|
| `Background` (default) | `scrollbar_bg` as the cell background, blank symbol | `scrollbar` as the cell background, blank symbol | a terminal, `capture-pane -e`, a buffer assertion |
| `Glyph` | `░` with `scrollbar_bg` as its foreground | `█` with `scrollbar` as its foreground | the above, plus a text snapshot and `capture-pane -p` |

- `Background` paints over the selection's background in that column on purpose:
  the bar has to stay readable on the row the cursor is on.
- The tokens keep their names and gain their second meaning: `scrollbar` is the
  thumb's background *or* its glyph's foreground, `scrollbar_bg` the track's
  background *or* the track glyph's foreground. Defaults unchanged (`#585b70`,
  `#45475a`) — surface-1 over whatever the terminal's own background is, one step
  above the selected row, which is what makes the track a *slight* tint.
- `scrollbar_style` without `scrollbar: true` is refused by name, and so is one on
  a non-`Menu` pane: a document that styles a bar it does not draw is a typo.

## How a colour is asserted

A snapshot reduces the buffer to text (`ui::screen`), which drops style — that is
the whole reason the first cut used glyphs. The tint is therefore asserted on the
buffer itself, one cell at a time, in `tests/components.rs`:

    let buf = buffered(&session(TINTED), 20, 5);           // the same render, kept
    let bg = |x, y| buf.cell((x, y)).unwrap().style().bg;  // Option<Color>

    assert_eq!(bg(18, 1), Some(theme.scrollbar_color()),    "thumb: the top cell of the track");
    assert_eq!(bg(18, 2), Some(theme.scrollbar_color()),    "thumb: two of the three track cells");
    assert_eq!(bg(18, 3), Some(theme.scrollbar_bg_color()), "track: the cell below is tinted");
    assert_eq!(bg(17, 1), Some(theme.selected_color()),     "the selected row keeps its background");
    assert_eq!(buf.cell((18, 1)).unwrap().symbol(), " ");   // and no glyph

`tuipalette_core::Buffer` is `ratatui_core::buffer::Buffer`, so `cell((x, y))`
returns the `Cell` the frame would have flushed, `Cell::style()` its resolved
`Style`, and `.bg` the `Option<Color>` the terminal backend turns into SGR —
`Some(Color::Rgb(88, 91, 112))` for the thumb (`#585b70`) and
`Some(Color::Rgb(69, 71, 90))` for the track (`#45475a`), which is exactly what
the live run shows on the wire. It is the idiom the selected-row test already used,
so nothing new was invented to check a colour. `bg_rows()` turns the same reading
into a row list, which is how the two-styles test compares a tinted bar against a
glyph one row for row.

## Two places the item's wording and the code did not line up

1. **"a test that the thumb tracks the selection and not just the offset."** There
   is no stored offset to track: `Menu::window` derives the first visible row from
   the selection, so the pane has one number, not two, and the bar has to be drawn
   from it or it would disagree with the rows on screen. The test therefore drives
   the *selection* down twelve rows in a three-row window and pins the thumb's row
   at the first, through the middle and at the last. A thumb placed at the
   selection's fraction of the list instead (offset 0, bar halfway down) was
   considered and rejected: it would claim a position the pane is not showing.
   **This revision changes nothing here** — the tinted style is computed from the
   same window, and the two-styles test pins that it lands on the same row.
2. **`scrollbar_bg` was a glyph, and is a background by default now.** The track
   used to be `░` and the thumb `█`, because a bar drawn only as background colours
   is invisible to everything that reads the screen back as text — `ui::screen` for
   snapshots, `tmux capture-pane`, `pty-check.py`. The user asked for the tinted
   cell anyway, which is what tmux and helix draw and what the eye reads as a
   scrollbar, so both exist: `Background` is the default, `Glyph` keeps the old
   drawing, and the invisibility problem is solved in the *test* instead of in the
   drawing. The token keeps the name the plan gave it.

## What was checked

    cargo test -p tuipalette-core --test components      # 26 passed, 12 snapshots
    cargo test --workspace                               # 100 passed, 0 failed
    for f in layouts/*.ron; do ./target/release/tuipalette --check --spec $f; done
                                                         # four ok

- `components__menu_scrollbar.snap` — five rows in a three-row window, `Glyph`
  style: a two-cell thumb at the top of the track.
- `components__menu_scrollbar_end.snap` — the same document after four `Down`s: the
  thumb at the bottom, the track above it, `three/four/five` in the text column.
- `a_background_scrollbar_tints_the_last_column` — the same five rows with the style
  left out: the thumb's two cells at `x=18, y=1..2` carry
  `Color::Rgb(88, 91, 112)`, the track cell at `y=3` carries `Rgb(69, 71, 90)`, the
  cell to their left still carries the selected row's background, every bar cell is
  a blank symbol, and no `█` is anywhere in the text.
- `both_scrollbar_styles_draw_the_same_geometry` — twelve rows in a three-row track,
  both styles, twelve selections: `thumb_rows(glyph) == bg_rows(tinted)` at every
  step, one cell, rows 1→3.
- `the_scrollbar_thumb_follows_the_selection` — the `Glyph` style, rows
  1,1,1,1,1,2,2,2,2,3,3,3 as the selection moves, never backwards, never off the
  track.
- `no_scrollbar_when_every_row_fits` — five rows in seven cells: no `█`, no `░`.
- `a_scrollbar_style_belongs_to_a_bar_that_is_drawn` — the default is
  `Background`, `Glyph` parses, a style with no `scrollbar: true` is refused at
  `panes[0].scrollbar_style` and a style on a `Text` pane likewise.
- A live run, both styles side by side, 200 rows in a 60x14 `tmux` pane
  (`seq 1 200 | tuipalette --spec <doc>`, the documents a copy of `layouts/dmenu.ron`
  with `scrollbar: true` added). One character per screen row, `T` = thumb,
  `_` = track, `.` = nothing:

      tinted start : ....T_______..
      glyph  start : ....T_______..
      tinted +100D : ....___T____..
      glyph  +100D : ....___T____..

  so both styles put the thumb on the same row before and after a hundred `Down`s,
  and the rows behind it moved from `1..8` to `94..101`. `capture-pane -p` shows
  the tinted bar as nothing but spaces; `capture-pane -e` shows it as
  `^[[48;2;88;91;112m ` on the thumb row and `^[[48;2;69;71;90m ` on the track —
  1 thumb cell and 7 track cells, exactly the numbers above. The same pane in the
  `Glyph` style reads `█` and `░` in plain text.

## What it costs

One comparison and two integer divisions per frame, per menu pane with the option
on, plus one `Buffer::set_style` per style instead of one `set_stringn` per row
with the glyphs — the tinted style is the cheaper of the two to draw. The pane
still holds only its rows and its selection: the bar is derived, not stored, which
is what keeps a resized pane and a filtered list correct without a scroll state to
invalidate.
