# 06 — Streaming source, matcher, match highlighting

Detail for item 06 of `docs/PLAN.md`.

## The line protocol (D12, D15)

```
firefox                          display = value = the whole line
Firefox\tfirefox.desktop\t🌐      display, value, icon
a\tb\tc\td                         icon is everything after the second tab
```

The display is sanitised — a control character becomes a space, because a menu row is one row — while the value is kept byte for byte, since it is what the caller gets back. An empty line is not an item. A line with an empty display but a value shows the value, so a row is never blank.

## Streaming

`source::stream(reader)` reads on its own thread and sends items down a channel; the loop drains it every 30 ms. The test that matters is `an_item_arrives_before_the_source_ends`: a reader that hands over one line and then waits, an item received while the reader is still open. Nothing waits for EOF.

## The matcher

```rust
pub trait Matcher: Send {
    fn set_query(&mut self, query: &str);
    fn score(&mut self, display: &str) -> Option<Match>;
}

pub struct Match { pub score: u32, pub positions: Vec<usize> }
```

Two calls, not one, because a scorer rebuilds its pattern per keystroke rather than per row. `positions` are character indices into the display, ascending, and they are what the menu turns into coloured runs.

- **`Nucleo` (default)** adapts `nucleo-matcher` for fzf-compatible ranking: `Pattern::parse(query, Smart, Smart)`, `pattern.indices` for score and spans, then sort and dedup, because nucleo returns them in match order.
- **`SimpleMatcher`** (behind `--no-default-features`) is a case-insensitive substring, ranked by how early it starts.
- **Smart case in both**: a query with an upper-case letter is matched exactly, a lower-case one matches anything.
- **The one difference, stated on purpose**: nucleo fuzzes, the fallback does not. `ff` finds `firefox` under nucleo and not under simple. Both are in the tests, so nobody has to guess.

## The picker

`Picker` owns the items, the query and the rows. On every keystroke it takes the input pane's value, re-scores, and pushes the surviving rows into the menu with `Menu::set_rows`. Ordering is score descending with arrival order breaking ties. A document with no query pane still types: the picker keeps the query itself, so `kind: Menu` alone is a dmenu.

The menu draws the matched characters in the accent colour and bold, in runs rather than per character, advancing x by `ui::width` (grapheme width, via `unicode-width`) so a wide glyph does not push the row.

## Two behaviours this item added or found

- **A default key map.** A document with no `keys` still answers to up/down, ctrl-p/ctrl-n, ctrl-k/ctrl-j, enter, tab, shift-tab and esc. The document is laid over those, and an explicit `None` clears a default — found when a bare menu document ignored every arrow.
- **Editing keys fall through.** `Session::on_key` hands an unbound character and an unbound backspace to the focused component, which is why typing reaches the query without a bind per letter. Delete needs a cursor to act on and is not an action yet.

## Evidence

```
cargo test -p tuipalette-core -- matcher     → 7 passed
cargo test -p tuipalette-core --test source  → 10 passed
cargo build -p tuipalette-core --no-default-features → Finished

find /usr -maxdepth 4 | head -60000 > items
pty harness: 60000 items streamed, typed `libx', enter
  → exit 0, value=/usr/share/licenses/libxml2, whole run 0.97s
```

The last one is the plan's manual check: results were on screen and selectable while the source was still delivering.

## Not in this item

A cursor inside the query (left/right, delete, home/end), so editing is append-and-backspace only. Enter with no matches does nothing rather than accepting the typed text; `--print-query` is how a caller gets that text today. The icon column is filled (protocol) but not yet width-centred, which is item 15.

## Hidden search text (2026-11-16, item 14 follow-up)

rofi's drun matches five fields — Name, GenericName, Exec, Categories, Keywords — while displaying only the Name (`source/modes/drun.c`, `matching_entry_fields` on the `next` branch, which is what this machine runs as 2.0.0). Typing `seahorse` there finds "Passwords and Keys" because `Exec=seahorse %u` and `Keywords=…;seahorse;`.

Our protocol grew a fourth field for exactly that: `display<TAB>value<TAB>icon<TAB>search`. `source::parse_line` keeps it, `Picker::refilter` falls back to it (and to the value) when the display does not match, scores such a hit at a constant 1 — below every display match — and gives it no highlight, since positions index the display. `scripts/apps.sh` fills it with GenericName, Categories, Keywords and the Exec line.

Verified in the real window: `seahorse` leads with "Passwords and Keys"; `keyring` leads with it too, followed by apps whose Keywords contain the word (Ark, KWalletManager), which is rofi's behaviour.
