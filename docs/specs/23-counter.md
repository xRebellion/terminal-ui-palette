# 23 — The item count (`shown/total`)

Last updated: 2026-11-21

fzf prints `49/49` — the rows that survived the query over everything the provider
sent — and it was the one piece of chrome the clipboard layout still missed
(`docs/specs/19-clipboard.md`: "no counter"). Nothing on screen knew either
number, because the picker holds the total and the panes hold the rows.

## The shape it took

`kind: Counter`, whose two numbers the picker writes after every refilter:

    panes: [(id: "count", kind: Counter, align: Right, frame: false)]

| field | default | meaning |
|---|---|---|
| `format` | `{shown}/{total}` | how the numbers are written; those are the only two placeholders |
| `align` | `Left` | `Left` / `Center` / `Right` inside the pane's rect |

- The pane is a label: it is not focusable, it never reacts to an action, and it
  wants no click. `CounterPane::set_counts` is the whole interface, and the picker
  calls it from `refilter` — the same mechanism as `Detail` (D18), where the pane
  displays a value the picker owns.
- `shown` is the menu's row count after filtering. `total` is
  `max(items received, shown)`: a document whose menu declares its rows statically
  has no streamed items at all, and its counter still has to say how many rows
  there are.
- `format` is validated at load time: a `{rows}` or a stray brace is a spec error
  naming `panes[N].format`, because the alternative is a counter that prints the
  word `{rows}` instead of a number. `format` and `align` on a pane that is not a
  `Counter` are refused for the same reason — a field that does nothing is a typo.

## The one document that had to change

`layouts/clipboard.ron`, because the counter is what the layout was missing:
the header row became a `Horizontal` split, `["fill", "6"]`, with the existing
`header` `Text` pane on the left and the new `count` pane on the right. That is
fzf's info line — label against the left edge, number against the right edge of
the same row. The document now has **6 panes** (`--check` says so) and its two
snapshots gained one row of chrome:

    │Clipboard History     2/2│first entry, shown on the right   │ │

The other three documents are untouched and still `--check` clean; nothing else
in the repository had to change to keep the versioned schema honest, because both
new fields default to "no counter" and an absent `format`.

## What was checked

    cargo test -p tuipalette-core --test components      # 24 passed, 12 snapshots
    cargo test --workspace                               # 97 passed, 0 failed
    ./target/release/tuipalette --check --spec layouts/clipboard.ron
    -> layouts/clipboard.ron: ok — 6 panes, 2 bindings

- `components__counter.snap` — a typed query `ta` over four rows: `2/4` against the
  right edge, six rows of screen.
- `the_counter_follows_typing_and_streaming_rows` — `0/0` before anything arrives,
  `4/4` after four rows, `2/4` after typing `ta` (beta and delta contain it), and
  `3/6` when two more rows stream in and one of them matches — both numbers move.
- `a_counter_takes_its_wording_and_its_edge_from_the_document` — `{shown} of {total}`
  reads `3 of 49` and lands at column 0 / 7 / 14 in 21 cells for left, centre, right.
- `the_clipboard_header_shows_the_count_beside_its_label` — the shipped layout's
  header row holds `Clipboard History` and `2/2` at its right edge.
- A live run of a 200-row list: `200/200` right-aligned under the list, `119/200`
  after typing `1` (captured from a `tmux` pane and from `scripts/pty-check.py`).

## Review fix, 2026-11-21

A review found the clipboard header cut a full history short: the count pane was
`"6"` cells, so cliphist's own cap (`max-items 250`) drew as `250/25`, and 1000
rows as `1000/1` — a wrong total, not a short one. Three changes:

- `layouts/clipboard.ron`: the header split is `["min:17", "fill"]` — the label
  keeps its 17 cells and the count gets the rest (24 at the layout's 106 columns);
- `CounterPane::render` cuts from the **left** with `…` when it truly has no room,
  so the total survives (`…0/250` in six cells);
- spec errors: `align` alone on a non-Counter pane is reported as
  `panes[i].align`, and `unfillable` scans the format as written instead of
  deleting the valid placeholders first, so `{sho{total}wn}` quotes `{sho` rather
  than a `{shown}` the document never wrote.

    cargo test --workspace                               # 104 passed, 0 failed
    # the_clipboard_count_keeps_every_digit_of_a_full_history: 250/250, 133/250, 1000/1000 at 106x26
    # a_counter_too_narrow_for_its_text_keeps_the_total: 250/250 in 7, …0/250 in 6, … in 1
    # live, tmux 106x26, --inline: 250/250, 133/250 after typing 1, 1000/1000

## What it costs

One `usize` comparison and two string replacements per refilter, and no work at
all in a document without a counter.
