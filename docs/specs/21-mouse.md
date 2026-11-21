# 21 — Mouse: hover, click and wheel

Last updated: 2026-11-21

Before this item every crossterm mouse event fell through `tuipalette-tui`'s
key path and was dropped. A picker that a user drives with a pointer needs three
things the plan named: a hit test against the rects the layout produced, a way
for a component to answer a pointer event, and capture enabled and released
around the run on every exit path.

## The opt-in

`window: (mouse: true)`, default `false`. Capture takes text selection away
from the terminal, so a picker that captures by default breaks select-to-copy
for every caller — `dmenu.ron` and `wifi.ron` stay off. The two documents that
own a floating window of their own, `rofi-like.ron` and `clipboard.ron`, turn
it on: in a window that exists only for the picker nobody loses a selection.

The opt-in is enforced twice: the terminal crate enables capture only for such a
document, and `Session::on_mouse` ignores every event for a document without
it, so a mouse sequence that reaches the tty anyway (another program enabled
reporting, or bytes were written to it) never moves anything.

## The shape

- **`tuipalette_core::Mouse`** (`pointer.rs`) — `Moved`, `Click`,
  `ScrollUp`, `ScrollDown` at a 0-based cell. The core's own type, like
  `Key`, so every behaviour is tested headless (D1). `tuipalette-tui/src/input.rs`
  translates crossterm's events: a drag is a move (hover follows it); a release,
  the right and middle buttons and the sideways wheel are dropped.
- **`Component::on_mouse(&Mouse, area) -> Reaction`** — default `Ignored`. Every
  pane gets the event *with its own rect*, so the pane under the pointer acts and
  the others can drop a stale hover. Additive: an existing component compiles
  unchanged.
- **`Session::on_mouse(area, mouse)`** — a click first moves focus to the pane
  under it (when that pane can hold focus: clicking the query pane after Tab
  puts typing back there), then offers the event. An `Emit(Invoke)` is an accept,
  exactly as from Enter.
- **One rect source.** `Session::rects(area)` is what both `render` and the hit
  test read, including the one-cell shift of `window.frame` (`ui::inner` is the
  frame's interior without drawing it), so a click lands where the eye sees.
- **The drawn area.** The run loop keeps the `Rect` the last frame was drawn in
  and hands it to `Picker::on_mouse`; that is what makes an inline viewport,
  which does not start at row 0, hit-test correctly.

## The menu

- **Hover** — the row under the pointer is tinted with the new theme token
  `hover_bg` (default `#313244`, quieter than `selected_bg`). Hovering never
  changes the selection, so Enter still accepts what the marker shows. A
  refilter forgets the hover; the next move sets it again.
- **Click** is a per-pane choice, `click: Select | Accept` on a `Menu`
  (added 2026-11-21 at the user's request — they were used to rofi's single
  click):
  - `Select` (the default): a click selects the row under the pointer, and a
    **click on the selected row accepts it**. A double click therefore accepts
    any row, with no timer to tune and nothing terminal-specific; the cost is
    that a single click on the already-selected row also accepts.
  - `Accept`: one click selects and accepts — rofi with
    `me-select-entry: "MousePrimary"` and `me-accept-entry: "!MousePrimary"`,
    which is the user's `~/.config/rofi/config.rasi`. `rofi-like.ron` and
    `clipboard.ron` use it.
  - The default stays `Select` so a stray click never launches anything in a
    document that did not ask. `click` on a pane that is not a `Menu`, or in a
    document without `window: (mouse: true)`, is a spec error naming the field.
- **Wheel** moves the selection three rows a notch (`WHEEL_STEP`), clamped at
  both ends, only while the pointer is over the list body. The window follows
  the selection as it does for keys, so the next click hits the rows now drawn.
- Clicks on the frame, on the empty space under a short list, or between panes
  hit nothing.

## Capture lifetime

`MouseCapture` in `run.rs` enables capture when constructed and disables it in
`Drop`: a normal exit, an `?` on an I/O error, and a panic unwinding through
`run` all release it. No profile sets `panic = "abort"`; if one ever does,
capture — and raw mode, which has the same exposure today — needs a panic hook.

## What was checked

    cargo test --workspace        # 123 passed, 0 failed (121 before `click`)

- `tests/components.rs`: `hovering_highlights_the_row_under_the_pointer`
  (cell background `hover_bg` on the hovered row, `selected_bg` kept on the
  selection, plain elsewhere, hover dropped over the query),
  `a_click_selects_and_a_second_click_accepts` (`Invoke` with `row 03`),
  `click_accept_takes_the_row_in_one_click`, `a_click_off_the_rows_selects_nothing` (frame, short-list blank, query),
  `the_wheel_scrolls_and_a_click_hits_the_scrolled_rows` (10 notches → 30, up →
  27, click on the top drawn row → 23, clamps at 199, ignored over the query),
  `a_click_moves_focus_to_the_pane_under_it`,
  `a_document_without_the_mouse_ignores_it`,
  `a_framed_window_shifts_the_hit_test_with_the_picture`.
- `tuipalette-tui` unit tests: the four kinds translated, the rest dropped.
- The real binary in a 60x16 pty, SGR sequences (1-based) into
  `layouts/rofi-like.ron` over `seq -f 'row %03g' 1 200`:

  | keys | result |
  |---|---|
  | `ESC[<35;10;8M` (move) then Enter | `row 001` — hover drew `48;2;49;50;68` (`hover_bg`) and did not select |
  | `ESC[<0;10;7M`, release, `ESC[<0;10;7M` | `row 003` — click selected, second click accepted (`click: Select`) |
  | `ESC[<0;10;7M` once, `click: Accept` (the shipped layout) | `row 003`, and `ESC[?1000l` written |
  | `ESC[<0;10;7M` then Enter, `click: Select` | `row 003` — the click selected without accepting |
  | 10 × `ESC[<65;10;8M` (wheel down) then Enter | `row 031` |
  | Tab, click on the query, `150`, Enter | `row 150` — the click gave the query focus back |
  | wheel then Escape | `exit=130`; `ESC[?1000l` written — capture released |
  | the same clicks with `--inline --height 14` | `row 003` |
  | `layouts/dmenu.ron` (no opt-in) | no `ESC[?1000h` written; Down Enter → `b` |
  | clicks into a document without `mouse: true`, then Enter | `row 001` — ignored |

## Not checked

A real pointer in kitty on the keybind (the pty proves the byte path; a live
session would prove kitty sends what crossterm expects — SGR 1006 is kitty's
default reporting format, so this is believed, not measured).
