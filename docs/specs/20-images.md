# 20 — Images: the kitty graphics protocol, as a host feature

Last updated: 2026-11-18

The clipboard picker has to preview a copied screenshot, and the fzf script this
project replaces does it with `kitten icat` (`~/.config/hypr/scripts/cliphist-fzf.fish`:
`icat --silent --transfer-mode=file --stdin no --place "$FZF_PREVIEW_COLUMNS"x"$FZF_PREVIEW_LINES"@0x0`).
This library draws text into a cell buffer and knows nothing about images. So the
question is not "should the library learn about images" but the narrower one the
plan asked: **can a provider hand the picker something that displays as a
picture, with the library unchanged?**

Measured answer: a provider can already put the *cells* on screen and cannot
choose which *image* they show — the image id lives in the cell's foreground
colour, and the foreground colour is the pane's theme token, not item data. The
decision is **no graphics in v1**, with the one design that could work written
down here so a later item starts from evidence instead of from this argument.

## What the protocol puts on the cell grid

Kitty's graphics protocol is an APC escape (`ESC _ G … ESC \`) naming an image
id, a payload (base64 PNG, or a path with `t=f`), and a rectangle in cells
(`c` columns, `r` rows). A *placement* is where an image is drawn. In the
Unicode-placeholder mode (`--unicode-placeholder`, `U=1`) the client creates a
**virtual placement** instead — invisible, only a prototype — and then displays
the image by printing ordinary text:

> "Since this character is just normal text, Unicode aware application will move
> it around as needed when they redraw their screens, thereby automatically
> moving the displayed image as well, even though they know nothing about the
> graphics protocol." — kitty 0.49.2, `/usr/share/doc/kitty/html/graphics-protocol.html`,
> § *Unicode placeholders*.

A placeholder cell is `U+10EEEE` followed by combining diacritics: the row, the
column, and the image id's most significant byte. The id itself is the cell's
**foreground colour**, and the placement id is the underline colour:

> "encoding the image ID in its foreground color … By using only the foreground
> color for image ID you are limited to either 8-bit IDs in 256 color mode or
> 24-bit IDs in true color mode … you can specify the most significant byte via
> a third diacritic." — same doc, § *Unicode placeholders*.

As text, one placeholder cell is four characters and six bytes. This is the run
`kitten icat --unicode-placeholder --place=6x3@1x1 --scale-up=yes --image-id=42`
emitted for the first cells of the first two rows (captured to a file,
`xxd -g1`; the full command line is under *What was checked*):

    00000080: f4 8e bb ae cc 85 cc 85 cc 85      U+10EEEE + U+0305 U+0305 U+0305   row 0 col 0 msb 0
    0000008a: f4 8e bb ae cc 85 cc 8d cc 85      U+10EEEE + U+0305 U+030D U+0305   row 0 col 1 msb 0
    00000094: f4 8e bb ae cc 85 cc 8e cc 85      U+10EEEE + U+0305 U+030E U+0305   row 0 col 2 msb 0
    000000c2: f4 8e bb ae cc 8d cc 85 cc 85      U+10EEEE + U+030D U+0305 U+0305   row 1 col 0 msb 0

so a 6-column row is six clusters of six bytes, and the diacritics are the table
the doc links (`rowcolumn-diacritics.txt`, whose values are legible in the
capture itself: 0 → U+0305, 1 → U+030D, 2 → U+030E, 3 → U+0310, 4 → U+0312, 5 →
U+033D). The same capture shows the discipline around the text: `ESC 7` save
cursor, `ESC[38:2:0:0:42m` (the id, as a colour), the placeholder rows,
`ESC[39m`, `ESC 8` — icat is polite enough to be embedded in someone else's
draw. The virtual placement is created by one
silent APC — `a=T,q=2,f=24,o=z,C=1,U=1,s=60,v=60,c=6,r=3,i=42` — transmit +
virtual placement, no cursor movement (`C=1`), no terminal response (`q=2`).

Two properties follow from the same doc, and they matter for a TUI:

- A **placement** is not a cell. Nothing in the text stream deletes it — the
  only removals are the `a=d` commands and re-transmitting the same id
  (§ *Deleting images*) — and with the default z-index it is
  composited **above** the text (the `z` key: "Negative z-index values mean that
  the images will be drawn under the text", § *Controlling displayed image
  layout*). So a repaint neither removes nor moves it — which is why the
  original fzf script opens every preview with `printf "\033_Ga=d\033\\"`, a
  delete of everything visible.
- A **placeholder** image is cell-bound: "Real images displayed on top of Unicode
  placeholders are not considered placements from the protocol perspective …
  they should be moved, deleted, or modified by manipulating the underlying
  Unicode placeholder as normal text." Clipping the text clips the picture;
  moving the text moves it; erasing a column of cells erases that column of
  picture.

## The half a provider already gets right

The text path needs no library change. This is the real binary, driven through
`scripts/pty-check.py` with a two-item `--in json` stream whose `detail` is a 4×2
grid of placeholder cells; the capture is the terminal's own byte stream:

    1b 5b 33 3b 32 36 48   f4 8e bb ae cc 85 cc 85 cc 85   f4 8e bb ae cc 85 cc 8d cc 85 …
    ESC [ 3 ; 26 H         U+10EEEE row0 col0 msb0           row0 col1 …               (×4)

Eight clusters for eight cells (a 4×2 grid), each emitted exactly once in a
1 580-byte capture spanning several frames — ratatui's `Buffer::diff`
(`ratatui-core-0.1.2/src/buffer/buffer.rs:471`) re-sends only changed cells, and
these never change. The path is: `source.rs::parse_json_line` (a field is text; the
line protocol's `parse_line` additionally rewrites control characters to spaces,
D15) → `DetailPane::set_text` → `ui::put` →
`Buffer::set_stringn`, which stores the **whole grapheme cluster** as one cell's
symbol (`buffer.rs:350–360`: split by `UnicodeSegmentation::graphemes`, filtered
by `!symbol.contains(char::is_control)`, width from `unicode_width`) → the
backend's `Print(cell.symbol())` (`ratatui-crossterm-0.1.2/src/lib.rs:274`),
verbatim. Measured on the host: the cluster's `unicode_width` is **1**, so it
occupies one cell, and `ui::fit` / `ui::put` count it as one column.

So "the library treats placeholders as ordinary characters" is not a hope — it
is the current behaviour, byte for byte.

## The one thing a provider cannot supply

The id. The bytes above land after this SGR, which the picker emitted for the
*previous* cell and which still applies:

    1b 5b 33 38 3b 32 3b 32 34 38 3b 32 34 38 3b 32 34 32 3b 34 39 6d
    ESC [ 3 8 ; 2 ; 2 4 8 ; 2 4 8 ; 2 4 2 ; 4 9 m           →  fg = #F8F8F2 = 16316658

That number is `layouts/clipboard.ron`'s `theme.text` token, coming from
`DetailPane::render`'s `Style::default().fg(paint.theme.text_color())` — one
colour for the whole pane, from the document. Kitty reads it as the image id, so
**every placeholder cell in that pane addresses image 16316658**, whatever the
provider wanted. The provider has eight bits of freedom (the msb diacritic:
id = `(k << 24) | 16316658`, 256 images), not the id space. `Menu` makes it
worse in the list: the selected row is `accent_color()` (#F92672, 16328306),
the others `text_color()`, so a thumbnail in a row would change identity the
moment the selection reached it.

There is no text-only way around this: the id is a *colour*, and the colour is a
`Cell` field the renderer sets. Escapes in item text do not survive either —
they are killed twice. `source.rs::sanitise` maps every control character to a
space (D15), and `set_stringn` drops any grapheme cluster containing one. Fed
`a\x1b[31mb` straight to a buffer, the six remaining cells are:

    [a][[][3][1][m][b]

the ESC gone, the parameters left behind as visible text. That is exactly what
the picker showed when icat's own output was piped in as items, below.

## The three things a provider would have to be able to say

| what it must say | what is true today |
|---|---|
| which image this cell shows | the cell's foreground colour — the pane's theme token; the provider's text cannot change it |
| here are the pixels | item fields are text, sanitised twice; the picker owns the tty and writes nothing to it but its own frame |
| draw again when the selection changes | the provider writes rows and exits, and nothing flows back: `scripts/clipboard.sh` pipes `cliphist list` into the picker, so the process that would draw is gone before a key is pressed |

The third is the structural one, and it is D18's own reason for existing: a
preview that follows the selection is item data plus a pane, because the picker
is the only process that knows the selection. A provider cannot redraw. Writing
to `/dev/tty` or stderr instead of stdout would put drawing behind the library's
back — with no rect (the layout is the library's, D3), no lifetime, and no
teardown — and would still be a single frame from a process that exits before
the first keystroke.

## The kitten icat route, as bytes

`kitten icat` is a program, not a library call, and what it writes depends on
one flag. Without `--unicode-placeholder` (this is what `cliphist-fzf.fish` runs):

    00000000: 0d 1b 5b 32 3b 32 48 1b 5f 47 61 3d 54 2c 71 3d 32 …
              \r ESC[2;2H  ESC_G a=T,q=2,f=24,o=z,C=1,s=60,v=60,i=42;<base64>ESC\

one APC that paints 60×60 px over whatever cells are under it, at a position
the *producer* chose. Nothing ties it to a cell, so nothing moves it when the
pane moves or the terminal resizes, and deleting is the caller's job (`a=d`) —
which is why the fzf script deletes before every preview. With `--unicode-placeholder` the same image becomes the
save-cursor / SGR / placeholder-grid / restore-cursor sequence quoted in the
first section — the one that is cell-bound and clip-safe.

Neither form reaches the terminal through this picker, because a provider's
stdout is the picker's stdin, not the display. Piping the placeholder-flag
capture into a real run (`layouts/dmenu.ron`, 60×16 pty) produced this:

    ›   _Ga=T,q=2,f=24,o=z,C=1,U=1,s=60,v=60,c=6,r=3,i=42;eJzs…
    …    [1C􎻮̍̅̅􎻮̍̍̅􎻮̍̎̅􎻮̍̐̅􎻮̍̒̅􎻮̍̽̅    [1C􎻮̎̅̅… [39m 8

— the APC header as a menu row, the base64 as its tail, and each following icat
line as its own item because a newline ends an item. The CR and ESC became
spaces; the placeholder clusters came through intact, in a list whose colour
names image 16316658, where no such image exists.

So `--unicode-placeholder` makes icat *polite*, not *sufficient*. It solves
clipping and movement — the library already carries those cells correctly — and
it does not solve who names the image, or who draws after the provider has
exited.

## What the picker would look like

The driving case is one picture, in the pane that already follows the selection:
`layouts/clipboard.ron`'s `preview`, the `fill` column of a 106×26 window —
**61×24 cells, 59×22 inside its frame** (resolved through the repo's own
constraint triple; spec 19's capture of the original agrees, columns 43–103).
At this machine's cell size (kitty window 1876×983 px, 208×49 cells → ≈9×20 px)
the box is 531×440 px, and an image is fitted into it whole:

| image | columns used of 59 | rows used of 22 |
|---|---|---|
| 16:9 screenshot | 59 | 15 |
| 4:3 photo | 59 | 20 |
| square | 49 | 22 |

The list does not change: one item is still one row, `Menu::window` is still one
number, the scrollbar and the counter are unaffected. **Row maths only changes
if thumbnails go into the list**, where an item N rows tall breaks the
`row == item` assumption that `Menu::window` and the scrollbar thumb are built
on; nothing in this item proposes that, and it is a separate item if it is ever
wanted.

For the record, the shape of a picture is available today without any of this: a
provider that renders the PNG to text and strips the colours (`img2txt` emits SGR
that the picker would otherwise show as literal `[0;34;5;44m`) fills the detail
pane with an ASCII circle. That is a fallback, not a preview of a screenshot.

## The decision

**No image protocol in the library, in v1.** Not because placeholders fail to
survive a redraw — they survive it exactly as designed — but because:

1. the id is a cell colour and the cell colour is the document's, so a provider
   can only ever address one image per pane token;
2. the provider is gone before the selection exists, so the preview that follows
   the selection cannot be its work at all (D18);
3. the only remaining channels are the tty behind the library's back, without a
   rect, a lifetime or a teardown.

If images come back, they come back as the plan's host-level option and not as a
provider trick, and the contract needs exactly three statements:

- **a reference, not escapes**: the item carries a path (a sixth field, or
  `detail` reinterpreted only inside a graphics pane); the host opens it. D15's
  sanitation stays true, and no escape ever enters the cell grid;
- **a rectangle, from the layout**: `kind: Graphics` in the pane vocabulary, so
  the host takes `c` / `r` from the `Rect` the layout resolved and never
  guesses;
- **a fallback per pane**: what to draw when the terminal is not kitty is
  document data, the way D11's icon placeholder is — a `Detail` beside it, or a
  text line, never a blank hole.

The host side is where the work is, and it is all lifecycle: allocate an id per
payload, `a=T,q=2,U=1` once, write the placeholder cells from the tui crate with
the id as the foreground colour, `a=d,d=i,i=<id>` on every payload change and on
every exit path including a panic, and re-place on `Resize`. That is a kitty
branch in `tuipalette-tui` — allowed by D1, refused by the plan's non-goals for
v1 and named in Risks as the sign of scope creep — plus a capability query (the
`a=q` probe the doc describes, § *Querying support and available transmission
mediums*) and a sixth protocol field. Roughly an item of its own, for one pane in
one document.

**Consequence for D19, which this file cannot change:** the shelf's image
condition cannot be met inside the picker in v1. The two honest ways out are
(a) unshelve after 21 and let image rows show what cliphist already sends them —
`[[ binary data 1.4 MiB png ]]` — or (b) keep the bind on fzf for image-heavy
use. This spec recommends (a): two pickers for one clipboard is worse than a
missing thumbnail, and the text-rendered shape is available to a provider that
wants it. Whatever is chosen should be recorded as a new decision rather than an
edit to D19; `DECISIONS.md` was outside this item's write scope, so the id is
free.

## What was checked

Kitty 0.49.2's own docs on this machine, not memory:
`/usr/share/doc/kitty/html/graphics-protocol.html` (§ *Transferring pixel data*,
*The transmission medium*, *Display images on screen*, *Controlling displayed
image layout*, *Unicode placeholders*, *Deleting images*, *Suppressing
responses*, *Requesting image ids from the terminal*) and `kitten icat --help`.

    # what icat emits, placeholder mode (a 6x3 place, --scale-up, --image-id=42)
    kitten icat --unicode-placeholder --stdin=no --transfer-mode=stream \
        --use-window-size=80,24,800,480 --place=6x3@1x1 --scale-up=yes \
        --image-id=42 red.png > place-6x3.bin; xxd -g1 place-6x3.bin
    -> 327 bytes: a=T,q=2,f=24,o=z,C=1,U=1,s=60,v=60,c=6,r=3,i=42;<b64>
       ESC[38:2:0:0:42m ESC7 ESC[2;0H ESC[1C <18 clusters> ESC[39m ESC8

    # the same without --unicode-placeholder
    -> \r ESC[2;2H ESC_G a=T,q=2,…,i=42;<b64> ESC\     (one placement, no text)

    # the placeholder cluster as the library handles it (scratch crate, /tmp)
    unicode_width of "U+10EEEE U+0305 U+0305 U+0305"        -> 1
    set_stringn stores f4 8e bb ae cc 85 cc 85 cc 85 in one cell, fg from the style
    set_stringn of "a\x1b[31mb"                             -> cells [a] [[][3][1][m][b]

    # the real binary, detail = a 4x2 placeholder grid
    PTY07_ITEMS_FILE=items.jsonl scripts/pty-check.py ./target/release/tuipalette \
        layouts/clipboard.ron '\x1b' '' --in json
    -> exit=130, screen 1580 bytes, 8 placeholder clusters at ESC[3;26H,
       preceded by ESC[38;2;248;248;242;49m  (theme.text = #F8F8F2 = 16316658)

    # icat's output piped in as items
    PTY07_ITEMS_FILE=place-6x3.bin scripts/pty-check.py ./target/release/tuipalette \
        layouts/dmenu.ron '\x1b' ''
    -> the APC header as a row: _Ga=T,q=2,f=24,o=z,C=1,U=1,s=60,v=60,c=6,r=3,i=42;eJzs…

    # the preview pane's rectangle, through the repo's layout constraints
    [Percentage(40), Fill(1), Length(1)] over 104x24   -> 42 | 61x24 | 1
    -> preview 61x24, 59x22 inside its frame

Every capture above was made in `/tmp` and removed; no image was displayed and no
pane of this session was touched. The item's checkable is met by its second
half: the reason the image preview stays out, with the commands above.

## What it costs

Nothing in code. The cost is the picture: image entries in the clipboard picker
show a header line instead of the screenshot, so a user choosing between three
copied screenshots reads three identical-looking rows. That is the regression
D19 refused, and it is why the decision above is recorded with its way back
rather than as a closed question.
