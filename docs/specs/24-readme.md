# 24 — A README that is true

Last updated: 2026-11-16

The README is the first thing a stranger reads and the easiest document to let rot:
it makes claims about behaviour that only a run can confirm. So the rule for this
item was that nothing goes in that had not just been executed, and the library
snippet is a compiled file rather than a fragment.

## What is in it

- The one-paragraph why, and the four commitments that shape the code (D7, D1, D2, D4).
- Quick start: build, a dmenu in one line, `--check`, and the four exit codes.
- A full RON document, plus a table of what each concept buys.
- The five-field item protocol, and why `search` and `detail` exist (D17, D18).
- The six layout/provider pairs in `layouts/` and `scripts/`, with their measured costs.
- Both hosts, including the Hyprland window rule verbatim.
- The three crates and the library call — pointing at `crates/tuipalette/examples/minimal.rs`.
- The measured phase table and the whole-path comparison against rofi.
- A docs map, the roadmap, and what is shelved with the reason (D19).

## What was verified, and how

- The sample document was **extracted from the README** and validated:
  `tuipalette --spec /tmp/readme-sample.ron --check` → `ok — 3 panes, 2 bindings`.
- The quick-start pipeline through a real pty:
  `PTY07_ITEMS_FILE=… scripts/pty-check.py ./target/release/tuipalette layouts/dmenu.ron '\x1b[B\r'`
  → `exit=0 value=b'b\n'`.
- The `--check` line quoted in the quick start, run as quoted.
- The example: `cargo build --examples`, then in a tmux pane
  `printf 'alpha\nbeta\ngamma\n' | ./target/debug/examples/minimal` with
  `Down Down Enter` → the pane showed `gamma`.
- `cargo test --workspace` → `88 passed, 0 failed`.

## What the checking caught

1. `scripts/run-command.sh` and `scripts/switch-window.sh` use **`dmenu.ron`**, not
   `rofi-like.ron`; the first draft had the wrong pairing in the table.
2. `Theme` holds **four colours** plus an `icon_placeholder` glyph — not "six colour
   tokens".
3. `tuipalette_tui::run` draws on **stdout**, so the picker needs the terminal: the
   binary re-points stdin and stdout at `/dev/tty` and keeps duplicates for the items
   and the value (`crates/tuipalette/src/host.rs`). A library example run in a pipeline
   printed a whole frame into the pipe. The example now refuses a non-terminal stdout
   and says why, and the README states the caveat — this is the one API truth the
   README exists to stop people rediscovering.

Also corrected here: `docs/STATUS.md` claimed nineteen landed items; it is sixteen
(1–15 and 19), with 16–18 and 20–23 open.
