# 15 — Icon slots and their fallback

Detail for item 15 of `docs/PLAN.md`. The rule: an icon is **one slot of declared width**, filled by whatever glyph the provider sends. No lookup table lives in the library (D12); the provider decides what an application's glyph is.

## The slot

- Declared per pane: `icon_width: 3` reserves three cells and one cell of gap before the marker.
- Absent: `icon_width` unset means no slot at all, which is what `--no-icons` selects.
- Width is measured **per grapheme cluster**, not per `char` and not per string: `unicode-segmentation` splits the glyph, `unicode-width` measures each cluster. An emoji built from a ZWJ sequence is one cluster two cells wide; measuring its parts would both over-count and invite a cut in the middle.

## The fallbacks

| glyph the provider sent | what the slot shows |
|---|---|
| fits the slot | the glyph, padded to the slot width |
| nothing (or empty) | the theme's `icon_placeholder`, default `·` |
| wider than the slot | the placeholder — a half glyph is worse than none |
| no slot declared | nothing; the text column simply starts four cells earlier |

The token lives in the theme, so it is document data like every other colour:

```ron
theme: (accent: "#a6e62a", selected_bg: "#141411", text: "#f8f8f2", dim: "#75715e", icon_placeholder: "·")
```

## Verified

One test renders all four required cases through the same menu and asserts the column where the display text starts, measured on buffer cells:

```text
-- native --            -- wide --              -- absent --          -- off --
│◆   › firefox         ││🎉   › firefox         ││·   › firefox         ││› firefox             │
│·     kitty           ││·     kitty           ││·     kitty           ││  kitty               │
```

- native-width glyph, double-width emoji and absent glyph: text starts on the **same cell** in all three.
- `--no-icons` (no slot): text starts exactly four cells earlier, and stays aligned within itself.
- A fifth test puts a double-width glyph in a one-cell slot and asserts the placeholder takes the cell instead of half an emoji.

Evidence: `cargo test -p tuipalette-core --test components` → `16 passed`; the matrix is the `components__icon_matrix` snapshot.

## What is not checked

Whether the terminal's font actually has the glyph. Nothing can know that from inside the process — the slot and the fallback keep the layout correct either way, which is the whole point of D12: the provider picks the glyph, the renderer refuses to let it break the columns.
