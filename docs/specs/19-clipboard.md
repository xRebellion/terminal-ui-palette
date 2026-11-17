# 19 — The clipboard picker

Last updated: 2026-11-16

Replaces `~/.config/hypr/scripts/cliphist-fzf.fish`: cliphist's history in a
floating window, the selected entry rendered beside it, a pick copied to the
clipboard and pasted into the window that had focus before.

## What the original did, and where each part went

| fzf / fish | here |
|---|---|
| `cliphist list` piped into fzf | `scripts/clipboard.sh` builds the rows; the picker streams them |
| `--delimiter '\t'`, `{1}` (id) | `value` is the id; the row is the entry's text |
| `--preview` running the script per selection | `detail` on every row, shown by `kind: Detail` — a field, not a command (D18) |
| `--preview-window=right:60%` | `sizes: ["40%", "fill", "1"]` in a `Horizontal` split |
| `--layout=reverse`, prompt on top | `query` first in the left column |
| `--header` | a `Text` pane with one line |
| `--border` | `window: (frame: true)`, with every pane `frame: false` except the preview |
| `bat --color=always` | plain text: no ANSI is interpreted (see below) |
| `icat` for images | out of scope, and it is plan item 20 |
| `cliphist decode \| wl-copy` | the same two commands, from the shell |
| `--paste` mode + `ydotool` after focus returns | the same trick, unchanged: the compositor spawns the paste so it is not a child of the picker's terminal |

## The measurements the layout was matched to

Both windows captured with `hyprshot-window`, then the border boxes found by
scanning for ink: cell ≈ 9×20 px.

| | fzf original (960×540) | ours (955×521) |
|---|---|---|
| outer box | rows 0–26, columns 0–105 | rows 0–25, columns 0–105 |
| prompt / header | rows 1 / 2 | rows 1 / 2 |
| list starts | row 3, column 1 | row 3, column 1 |
| preview box | rows 1–25, columns 43–103 | rows 1–24, columns 43–102 |
| blank column before the right border | 1 | 1 (the `Spacer` in the layout) |

The first attempt put the preview inside the left column's vertical split, so it
started below the header (row 3) instead of beside the prompt (row 1), and it
butted against the window border with no gap. That is what "the preview border
is too narrow" was: one missing column and a preview that was two rows too
short. Both are structural, which is why the fix is in the layout file.

## What a provider pays

`scripts/clipboard.sh` decodes every text entry up front (49 entries, 72 ms) so
a keystroke never waits on `cliphist`. Images are not decoded: their row already
says `[[ binary data … ]]`, and the detail says the image itself needs a
graphics protocol. A history of thousands of entries would want the detail
capped or fetched lazily — the point of a field is that this is the provider's
decision, not the library's.

## Checkable

    cargo test --workspace                       # 88 passed, incl. both clipboard snapshots
    tuipalette --spec layouts/clipboard.ron --check
    scripts/clipboard.sh --list | wc -l          # 49
    scripts/clipboard.sh                         # open, filter, Enter → wl-copy holds the entry

## Shelved (2026-11-16, same day)

The picker is **not** wired to a keybind, and `SUPER+Shift+V` still runs
`cliphist-fzf.fish` (D19). The code stays and stays tested, because it is what
proved the schema — a preview that follows a selection, a frameless pane and a
single window frame are all things the other pickers can use.

What a user would lose today by switching: images (the original renders them
with `kitty +kitten icat`, plan item 20), the mouse (21), a scrollbar (22) and
the `49/49` count (23). Nothing else in the table above is a regression — the
list, the preview, the paste-after-focus and the shape all match.

