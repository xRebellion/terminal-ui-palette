# 13 — The rofi-like layout

Detail for item 13 of `docs/PLAN.md`. The rule this item exists to prove: a rofi-shaped picker is **data plus provider scripts**, and nothing in the crates knows rofi exists.

## What shipped

| file | what it is |
|---|---|
| `layouts/rofi-like.ron` | 60x14 card, monokai tokens taken from `themes/monokai.rasi` (accent `#a6e62a`, background `#141411`, text `#f8f8f2`, dim `#75715e`), three panes (query 3 rows, results filling, status 1 row), two bindings |
| `layouts/dmenu.ron` | the dmenu shape: one query row, results, no status |
| `scripts/apps.sh` | the provider: 106 entries from `*.desktop`, `display\tcommand`, `Terminal=true` wrapped in kitty, field codes stripped, sorted, deduplicated |
| `scripts/launch-apps.sh` | the app-launcher entry point: one `PICKER=` line, then either `rofi -show drun -dmenu -i` or the floating adapter over `layouts/rofi-like.ron` |
| `scripts/dmenu.sh` | the dmenu entry point: lines in, one line out, the same one `PICKER=` line |

No crate contains the string `rofi` except this documentation: the layout is read by the generic spec parser, and the provider is a shell script.

## Verified

| claim | command | result |
|---|---|---|
| the layout is a valid document | `tuipalette --check --spec layouts/rofi-like.ron` | `ok — 3 panes, 2 bindings` |
| its geometry is the rofi card's | `tuipalette --spec layouts/rofi-like.ron --print-geometry` | `60x14` |
| the window class the compositor rule matches | `tuipalette --spec layouts/rofi-like.ron --print-class` | `tuipalette` |
| the provider lists real applications | `scripts/apps.sh | wc -l` | `106` |
| the dmenu contract still holds | `scripts/pty-check.py ./target/release/tuipalette layouts/dmenu.ron '\x1b[B\r'` | `exit=0 value=b'herdr\n'` |
| the floating host draws this layout | `scripts/verify-floating.sh layouts/rofi-like.ron` | 541x281 px, floating, hidden workspace, value back |

Both entry points read the **same** provider: `launch-apps.sh` pipes `scripts/apps.sh` into whichever picker the one `PICKER=` line names, so the entries are identical by construction rather than by comparison.

## The side-by-side

It was run on the user's screen, both halves on the same keybind with one line changed:

```
PICKER=rofi       scripts/launch-apps.sh
PICKER=tuipalette scripts/launch-apps.sh
```

The verdict was "everything looks good" with one difference: **the hint bar said nothing about navigation**. Rofi's does, ours listed only `enter`, `esc` and `type to filter`.

That is the right kind of finding for this item: the fix was one line of data, no recompile, no code change —

```ron
(id: "status", kind: Status, items: ["enter run", "↑↓ move", "tab focus", "esc cancel", "type filters"]),
```

— and it landed in all three shipped layouts (`dmenu.ron`, `wifi.ron` included), each still `--check` clean. The hints are 56 cells in a 58-cell inner width, so nothing clips at 60×14, and navigation now sits second, where the eye lands, instead of last where a narrow document would cut it first.

## First attempt, kept because it failed

The first launch printed `tuipalette: exit 127` and no window: a compositor-spawned script inherits a minimal PATH, and the adapter looked for a binary named `tuipalette` that only exists in `target/release`. The adapter now checks for it and says so —

```
tuipalette-floating: `tuipalette` is not on PATH — set TUIPALETTE_BIN to a built binary (target/release/tuipalette)
```

— instead of opening a terminal that dies inside itself.

## What is still not compared

Only the eyes' judgements, and they were made: size, position, density, sort order, prompt and hints. The timing comparison (ours ~120 ms to appear, rofi ~43 ms) belongs to `docs/specs/09-latency.md` and is recorded there.

## Providers: speed and usage memory (2026-11-16, after item 14)

Two complaints from using the migrated binds: the app list took a second to appear, and both lists were plain alphabetical with no memory of what gets used.

**Speed.** `scripts/apps.sh` forked four subprocesses per `.desktop` file (283 files → ~1.3 s for 106 rows). It is now one `awk` pass over the files array: **20 ms**, byte-identical output (`diff` empty). The picker opens immediately because the first rows arrive immediately.

**Usage memory.** `scripts/usage.sh` keeps `~/.cache/tuipalette/usage.tsv` as `key<TAB>count<TAB>last	time`, and offers two subcommands: `order <key-field>` (reorders stdin rows: most recently used first, then most used, then arrival order) and `record <key>` (bump after an accept). Both providers use it:

```
scripts/apps.sh   | usage.sh order 2 | tuipalette-floating.sh layouts/rofi-like.ron cat -
run-command.sh    | commands | usage.sh order 1 | tuipalette-floating.sh layouts/dmenu.ron … cat -
```

Field 2 for apps is the exec line (what the picker returns, so it is the natural key); field 1 for commands is the command itself.

This works because the picker sorts by match score and keeps **arrival order for ties**: with an empty query every row scores 0, so the list appears exactly as the provider ordered it. As soon as the user types, match score takes over — which is the right priority, and the reason an in-library frecency tie-break is still worth having (item 18).

Verified live: accept `zsh` in the run picker → the next open shows `zsh` above `[`; accept `alacritty` in the app picker → `usage.tsv` gains `alacritty	1	<now>` and the app launches.
