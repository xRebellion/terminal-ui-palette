# 18 — Ranking: a literal hit before a scattered one

Last updated: 2026-11-21

The user's report: typing `kn` in the app picker put **Konsole** first, and
**Arknights** — which holds `kn` as two adjacent letters, and has 5 recorded
launches in `~/.cache/tuipalette/usage.tsv` against Konsole's none — was not
among the top four. Item 18 had already named the same defect for `kni`.

## Why it happened

Two causes, both read in the code and the first measured:

1. **The scorer's bonuses.** nucleo reproduces fzf's algorithm, which rewards a
   query character that lands on a word start or a case change. `K`onsole's
   `K` is the start of the display, so a *scattered* `K…n` scores above the
   *adjacent* `kn` inside Ar`kn`ights. That is fzf's behaviour, not a misuse
   of nucleo — and it is right for a file finder, wrong for a launcher.
2. **Usage stopped at the first keystroke.** `scripts/usage.sh` sorts the list
   most-recent first before the picker sees it; `Picker::refilter` then sorted
   by score alone, so arrival order — the usage order — only broke exact score
   ties, which almost never happen.

## The design (D20)

`crates/tuipalette-core/src/rank.rs`. A row is put in a **tier**, and the tier
decides first:

| tier | the display … | example for `ark` |
|---|---|---|
| `Exact` | is the query (smart case) | `Ark` |
| `Prefix` | starts with it | `Arknights` |
| `WordStart` | has a word that starts with it (after a non-alphanumeric, or a lower→upper change) | `Bookmark Ark` |
| `Substring` | contains it anywhere | `Spark` |
| `Fuzzy` | holds it as a subsequence — the scorer's hit | `A…r…k` |
| `Hidden` | does not match; the value or search field does (D17) | `Passwords and Keys` for `sea` |

Inside a tier, **arrival order** decides — except in `Fuzzy`, where the
scorer's number decides first, because "how scattered" is the only question a
literal hit does not raise. Arrival order is the provider's own ranking, which
for both launchers is usage, so usage now orders typed results too without the
library learning what usage is (D12: provider logic stays in providers).

A literal row is highlighted where it matched literally, so the colour agrees
with the reason it ranks where it does.

### What the plan proposed and what changed

- (a) literal before subsequence — **done**, refined into four literal tiers.
- (b) "then shorter display" — **dropped**. Measured on the real list, it put
  `Arknights` above `Arknights: Endfield` regardless of which was launched
  last, which is the opposite of what (c) asks for. Provider order already
  carries a better tie-break than length.
- (c) usage as a tie-break for filtered results — **done** through arrival order,
  with `scripts/usage.sh` unchanged (`bash -n` clean, same output).

### `after: 17` relaxed

The plan sequenced 18 after 17 because "the revisit decides what stays in the
providers". The fix is entirely in `tuipalette-core` and changes no provider,
so the user relaxed the link on 2026-11-21. 17 may still move usage into the
library; this design does not stand in its way.

## What was checked

`crates/tuipalette-core/examples/rank.rs` prints the top rows the real picker
ranks, from the line protocol on stdin, without a terminal:

    scripts/apps.sh | scripts/usage.sh order 2 > apps.tsv
    cargo run -q --release -p tuipalette-core --example rank -- <query> 5 < apps.tsv

| query | before (top five) | after (top five) |
|---|---|---|
| `kn` | Konsole, CachyOS Kernel Manager, KDE Connect Indicator, KDE Connect, KDE Connect SMS | Arknights: Endfield, Arknights, Konsole, CachyOS Kernel Manager, KDE Connect Indicator |
| `kni` | KDE Connect Indicator, Arknights: Endfield, Arknights, ElecWhat, Discord (Wayland) | Arknights: Endfield, Arknights, KDE Connect Indicator, ElecWhat, Discord (Wayland) |
| `fire` | LibreOffice Impress, LibreOffice Writer, LibreWolf Modifier Wrapper, LibreOffice, Passwords and Keys | unchanged — no app display contains `fire`, so every row is `Fuzzy`/`Hidden` as before |
| `code` | LibreOffice, Discord (Wayland), DBeaver Community, Passwords and Keys, Ark | unchanged, for the same reason |
| `ark` | — | Ark, Arknights: Endfield, Arknights, Passwords and Keys, ElecWhat |

The two Arknights rows lead in usage order: `…/rungameid/12` (Endfield) was
launched last (`1795195610` vs `1795193952`).

On the command runner's 4671 PATH entries, `fire` gained `aafire` (a literal
`Substring`) in fifth place where `fish_key_reader` (a scattered `f…i…r…e`)
was; `code` is unchanged (every top row is a `Prefix`).

- `cargo test --workspace` → 112 passed, 0 failed: `rank.rs` unit tests (each
  tier told apart, best occurrence wins and is highlighted, smart case, a query
  longer than the display) and in `tests/source.rs`
  `a_literal_hit_outranks_a_scattered_one`,
  `literal_hits_rank_exact_then_prefix_then_word_then_inside`,
  `equally_good_hits_keep_the_providers_order`,
  `a_literal_row_highlights_the_run_it_matched`; every existing ordering test
  unchanged.
- The real binary in a pty: `PTY07_ITEMS_FILE=apps.tsv scripts/pty-check.py
  ./target/release/tuipalette layouts/rofi-like.ron 'kn\r' ''` →
  `value=b'env LUTRIS_SKIP_INIT=1 lutris lutris:rungameid/12\n'` (Arknights:
  Endfield), where it was `konsole` before.

## What it costs

One extra pass of a character comparison per row per keystroke, before the
scorer is asked at all — and a literal hit skips the scorer. The example ranks
all 4671 commands for `e` in 5–6 ms end to end, including process start and
reading stdin.
