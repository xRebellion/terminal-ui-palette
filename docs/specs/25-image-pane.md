# 25 — An Image pane, with the picture protocol behind a trait

Last updated: 2026-11-21

Item 20 decided "no graphics protocol in v1" because a *provider* cannot name
an image through the cell grid (`20-images.md`). This item does what that file
named as the way back: the image is drawn by the **host**, from a **path** in
the item, into a **rect from the layout**, with a **text fallback** per pane.
It adds no crate (103 unique dependencies before and after,
`cargo tree -e normal --prefix none | sort -u | wc -l`).

## The design

Three layers, each knowing only its own part:

| layer | knows | does not know |
|---|---|---|
| item protocol (`--in json`) | `image`: a file path | pixels, escapes |
| `tuipalette-core` | `kind: Image` pane; `Component::image(area) -> Option<ImageRequest>` (path + rect); `Session::images` / `Picker::images` collect them per frame | any terminal or protocol |
| `tuipalette-tui` | the `Graphics` trait and its backends; `detect(Protocol)` picks one | which pane asked, or why |

**The pane.** `ImagePane` wraps a `DetailPane`: it always draws the row's
`detail` text, then answers `image()` with the selected row's path and the
rect inside its frame. A row with no `image` answers `None`, so a text entry
never shows the previous picture. It follows a menu exactly as `Detail` does
(`from`, `wrap`); the picker fills both through the same `sync_detail`.

**The swap point.** `tuipalette-tui/src/graphics/mod.rs`:

    pub trait Graphics {
        fn place(&mut self, images: &[ImageRequest], buf: &mut Buffer, out: &mut dyn Write) -> io::Result<()>;
        fn clear(&mut self, out: &mut dyn Write) -> io::Result<()>;
    }

`place` runs inside `terminal.draw`, after the panes painted and before
ratatui flushes, with the frame's buffer and the tty. That lets a
cell-based backend overwrite the fallback cells and a beside-the-grid backend
(sixel, iTerm2) write its escapes; both reach the terminal in the same frame.
`place` is told the whole set for the frame, so "what is no longer asked for"
is the backend's to delete. `clear` runs from a `Drop` guard (`Pictures` in
`run.rs`), so a normal exit, an error and an unwinding panic all delete what
is up — the same shape as `MouseCapture`.

**Adding a protocol** is one file under `graphics/` implementing the trait,
one `Protocol` variant, and one arm in `detect`. No pane, document or item
changes.

**Choosing one.** `--images auto|kitty|none`, default `auto`. `auto` reads the
environment (`KITTY_WINDOW_ID` set, `TMUX` unset) instead of querying the
terminal, because a query's reply would race the key reader on the same tty.
A wrong guess shows a pane of placeholder glyphs; `--images none` is the
escape. Inline runs (`--inline`) never draw pictures: they share the screen
with the shell and a picture would outlive the viewport.

## The kitty backend

`graphics/kitty.rs`, about 170 lines including the 297-entry diacritic table
(copied from kitty's `rowcolumn-diacritics.txt`) and a 15-line base64 for the
path.

- **Upload**: `ESC_G a=T,t=f,f=100,U=1,q=2,C=1,i=<id>,c=<cols>,r=<rows>;<base64 path> ESC\`.
  kitty reads the PNG itself (`t=f`, `f=100`), so nothing is decoded here;
  `U=1` makes a virtual placement that kitty fits into `c`x`r` cells with the
  aspect ratio kept; `q=2` means no reply ever lands in the key stream;
  `C=1` leaves the cursor alone.
- **Display**: every cell of the rect becomes `U+10EEEE` + row diacritic +
  column diacritic, with the foreground colour set to the 24-bit id. Each
  cell carries its own row and column, so ratatui's diff can resend any one
  of them.
- **Lifetime**: same path at the same size reuses the id and sends nothing; a
  new path or size uploads a new id; every id not asked for this frame is
  deleted with `a=d,d=I` (placements and data).
- **Limits**: PNG only (`f=100`; cliphist stores screenshots as PNG, other
  binary entries stay text); 297 cells on a side; the terminal must be able
  to read the file, so a local session only.

## The clipboard picker

`scripts/clipboard.sh` decodes each `png` entry once to
`$XDG_RUNTIME_DIR/tuipalette-clipboard/<id>.png` (mode 700 directory) and
passes it as `image`; files whose id has left the history are removed on the
next run. `layouts/clipboard.ron` switches `preview` from `Detail` to
`Image`. Listing 25 entries with 3 images: 194 ms cold, 140 ms warm.

## What was checked

- `cargo test --workspace` → `141 passed, 0 failed` (core 115, tui 14, bin
  12). New: `tuipalette-tui/tests/graphics.rs` (base64 against RFC 4648
  vectors, the diacritic table, the transmit bytes, placeholder cells and
  colour, upload-once / delete-on-swap / delete-on-empty, re-place on resize,
  `clear` twice, `detect(None)`); core tests for the pane's request and
  fallback, no request on a text row, no request without an Image pane, a
  JSON `image` field, `wrap` on an Image; the `--images` flag.
- Live, real kitty 0.49.2, the real picker through `scripts/clipboard.sh`:
  the selected screenshot drawn in the preview; moving to a text row shows
  the text and no picture; moving across the three images swaps them; a
  window resize re-places at the new size; Escape exits `130` and the kitty
  window that ran it shows no picture afterwards; `--images none` shows the
  `[[ binary data … ]]` text instead. Captures were taken with
  `hyprshot-window` and viewed.

Not checked: a panic mid-preview (the `Drop` guard is the same mechanism as
the mouse capture, unproven here); a non-kitty terminal with `auto`
(believed: `KITTY_WINDOW_ID` is unset, so text); SSH.
