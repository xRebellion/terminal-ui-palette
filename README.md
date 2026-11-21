# terminal-ui-palette

A terminal picker built from a **data document**, plus the library that runs it.

One binary, `tuipalette`, reads a RON or JSON document describing a small set of
panes — a query line, a list, a preview, a status row — and puts them on screen.
Items arrive on stdin; one value leaves on stdout. Anything rofi can do with a
mode, this does with a layout file and a provider script:

- **rofi-like pickers are a customer of the schema, not a property of it.** The
  shipped `layouts/*.ron` are data. No crate names them, no crate knows what an
  app launcher is (D7 in [`docs/DECISIONS.md`](docs/DECISIONS.md)).
- **Terminal-agnostic.** No kitty, sixel or wezterm protocol code: ratatui draws
  cells, the terminal is just a terminal.
- **No daemon.** One process per invocation, a stdin/stdout contract, ~2 ms from
  exec to the first frame.
- **No config language.** The document is validated data — no conditionals, no
  expressions, no embedded runtime (D4). Anything dynamic is a provider writing
  lines to stdin.

Status: **pre-release** (`0.0.0`). Sixteen work items landed (1–15 and 19),
`cargo test --workspace` → **88 passed**. Roadmap at the bottom.

## Quick start

```
cargo build --release

# a dmenu in one line: no tabs, so each line is both display and value
printf 'a\nb\nc\n' | ./target/release/tuipalette --spec layouts/dmenu.ron

# validate a document without opening a window
./target/release/tuipalette --spec layouts/rofi-like.ron --check
# → layouts/rofi-like.ron: ok — 3 panes, 2 bindings
```

Exit codes: `0` accepted (value on stdout), `130` cancelled, `1` the document is
wrong (with the path, line and column), `2` unusable host or flags.

## A document

```ron
(
    window: (title: "launcher", frame: true),
    theme: (accent: "#a6e62a", selected_bg: "#141411"),
    host: (width: 60, height: 14, class: "tuipalette"),
    layout: (
        dir: Vertical,
        sizes: ["3", "fill", "1"],
        children: [(pane: "query"), (pane: "results"), (pane: "status")],
    ),
    panes: [
        (id: "query", kind: Input, title: "run"),
        (id: "results", kind: Menu, title: "applications"),
        (id: "status", kind: Status, items: ["enter run", "esc cancel"]),
    ],
    keys: { "esc": Some(Quit), "enter": Some(Accept) },
)
```

| concept | what it buys |
|---|---|
| `layout` | one tree of splits and leaves; sizes are `N`, `N%`, `fill`, `min:N`, `max:N` |
| `panes` | `Input`, `Menu`, `Status`, `Text`, `Spacer`, `Detail`, `Counter` — plus your own, as a `Component` impl and a recompile (D5) |
| `keys` | global map + per-pane overrides; `None` clears an inherited bind (D8) |
| `theme` | six colours (`accent`, `selected_bg`, `text`, `dim`, `scrollbar`, `scrollbar_bg`) plus an icon placeholder glyph, `#rrggbb` only |
| `host` | a hint for the launcher: cell size and window class. The library never places a window (D3) |
| `window.frame` | one border around everything, instead of one per pane |

Every field, default and merge rule is in
[`docs/spec-reference.md`](docs/spec-reference.md) — written to be sufficient on
its own: `layouts/wifi.ron` and its provider were built from that page alone.

## Items on stdin

One item per line, up to five TAB-separated fields; trailing fields optional:

```
display<TAB>value<TAB>icon<TAB>search<TAB>detail
```

- `display` is what the row shows; `value` is what `Accept` returns, byte for byte.
- `search` is matched and never drawn — which is how `seahorse` finds an entry
  whose name is "Passwords and Keys" (rofi's drun matches the same five fields).
- `detail` is shown by a `Detail` pane while the row is selected — a preview that
  follows the selection, kept as data rather than a command the picker runs (D18).
- `--in json` / `--out json` carry values byte-exact, newlines included.

## Shipped pickers

Data in `layouts/`, logic in `scripts/` — copy them as templates.

| layout | script | does |
|---|---|---|
| `rofi-like.ron` | `scripts/launch-apps.sh` | app launcher over `.desktop` files, 106 rows in 20 ms, most-used first |
| `dmenu.ron` | `scripts/dmenu.sh` | a plain menu for shell use |
| `dmenu.ron` | `scripts/run-command.sh` | rofi's `run`: 4669 PATH executables, free text accepted |
| `dmenu.ron` | `scripts/switch-window.sh` | rofi's `window`: mapped toplevels, focuses the pick |
| `wifi.ron` | `scripts/wifi.sh` | network list, two-stage flow |
| `clipboard.ron` | `scripts/clipboard.sh` | cliphist history with a live preview, then pastes — implemented, **shelved**: the keybind still runs the fzf version until 20–23 land (D19) |

`scripts/usage.sh` remembers what was accepted (`~/.cache/tuipalette/usage.tsv`)
and orders the next open; `scripts/tuipalette-floating.sh` is the floating-window
adapter; `scripts/pty-check.py` drives a real run through a pty — that is how the
acceptance checks in `docs/specs/*` were made.

## Hosts

- **Inline** (`--inline [--height N]`) draws at the bottom of the calling
  terminal and wipes its region on exit. Opt-in: without the flag the picker
  takes the alternate screen.
- **Floating** is a terminal window placed by the compositor, with the size from
  the document's `host` and a class a window rule can match. On Hyprland:

```lua
hl.window_rule({
  name = "tuipalette-floating",
  match = { class = "^(tuipalette)$" },
  float = true, center = true, stay_focused = true,
})
```

Both hosts are adapters over the same stdin/stdout contract, which is why
nothing in the library knows which one it is in.

## Library

| crate | holds |
|---|---|
| `tuipalette-core` | spec, layout, key map, session, matcher, picker, the built-in components |
| `tuipalette-tui` | crossterm + ratatui: the run loop, input translation, the terminal |
| `tuipalette` | the binary: CLI, hosts, wires the pieces together |

```rust
let spec = UiSpec::load(Path::new("layout.ron"), None)?;
let mut picker = Picker::new(spec)?;
let options = Options { inline: None, timing: None };
match tuipalette_tui::run(&mut picker, source::stdin(), &options)? {
    Exit::Accepted(value) => println!("{value}"),
    Exit::Cancelled => std::process::exit(130),
}
```

A compiling version is `crates/tuipalette/examples/minimal.rs`:

```
printf 'alpha\nbeta\n' | cargo run --example minimal
```

One caveat the example spells out: the picker draws on the terminal, so
stdout has to *be* one. The binary makes that true even in a pipe by
re-pointing stdin and stdout at `/dev/tty` and keeping duplicates for the
items and the value (`crates/tuipalette/src/host.rs`) — a library user who
wants the same needs the same trick.

Components render into a plain `Buffer`, never a `Frame`, so every pane is
testable headless and the snapshots are text diffs.

## Performance

Measured on this machine, release build, launched the way a keybind launches it
(spawn excluded, as D14 defines the budget):

| phase | time |
|---|---|
| document parse | 0.1 ms |
| terminal init | 0.5–0.8 ms |
| exec → first flushed frame | **1.3–1.9 ms** |
| key → repaint | 0.61 ms median, 1.38 ms p95 |

Whole path including the terminal's own spawn: 117–132 ms (kitty), 86–97 ms
(alacritty), against rofi's 42–45 ms — the delta is the spawn, not the picker.

## Docs

| file | what |
|---|---|
| [`docs/spec-reference.md`](docs/spec-reference.md) | every field, default, action, flag and the item protocol — start here to write a picker |
| [`docs/PLAN.md`](docs/PLAN.md) | the work items, what each was for and how it is checked |
| [`docs/DECISIONS.md`](docs/DECISIONS.md) | D1–D19, append-only, with the rejected alternative |
| [`docs/STATUS.md`](docs/STATUS.md) | what works, what is known broken, what is next |
| [`docs/DONE.md`](docs/DONE.md) | the ledger: one line per finished item, with the command that proved it |
| `docs/specs/NN-*.md` | one page per item: measurements, dead ends, what it cost |

## Roadmap

Open: conditional actions and document transitions (16), the herdr and wifi
revisit (17). Ranking (18) is done: a literal hit outranks a scattered one, and
the provider's usage order breaks ties after typing too (D20).

Also open: mouse support — hover, click and wheel (21). Decided against for v1,
with the evidence in `docs/specs/20-images.md`: images over the kitty graphics
protocol (20) — an image is not a cell, so it belongs to a host, not the core.
The clipboard picker is implemented but not wired to a keybind until mouse lands
and the image question is settled for it (D19).

## License

MIT OR Apache-2.0.
