# terminal-ui-palette — Status
Last updated: 2026-11-16

## What works
- Three crates build and link as one workspace — `cargo build --workspace` → `Finished`
- A RON or JSON document parses, validates, resolves to pane rects, and deserializes its key map — `cargo test -p tuipalette-core --test spec` → `16 passed`
- Focus, bubbling, emit and the key-map override/unset work headless — `cargo test -p tuipalette-core --test dispatch` → `14 passed`
- All five built-ins draw from their pane kind, pinned by snapshots — `cargo test -p tuipalette-core --test components` → `14 passed`
- The real picker runs: items on stdin, keys from the terminal, one value out — `scripts/pty-check.py` on `tests/fixtures/dmenu.ron` → `exit=0 value=b'herdr\n'`, esc → `exit=130`
- Streaming, ranking and highlighting work end to end — 60 000 `find /usr` items, typed `libx`, enter → `/usr/share/licenses/libxml2` in 0.97 s

## Known broken and believed
- **Believed:** `--inline` in kitty leaves the cursor and scrollback clean on exit — verified only in a pty, where ratatui's cleanup ran but kitty's own scroll region was not involved; would confirm by running `tuipalette --inline --height 8` in kitty and scrolling afterwards.
- **Believed:** inline leaves kitty's scrollback and cursor exactly as it found them — verified in a pty only; would confirm by running `--inline --height 8` in kitty and scrolling afterwards.
- **Done, not suspected:** the floating window's class, size and position are now checked against a live compositor by `scripts/verify-floating.sh`, which also proves the launch takes no focus.

## State of what the project touches
| what | state now | checked with |
|---|---|---|
| repo root | git, clean tree, items 01–03 committed | `git status --porcelain`; `git log --oneline` |
| `crates/tuipalette-core` | spec, layout, keys, router, five built-ins, matcher, picker; 63 tests | `cargo test -p tuipalette-core` |
| `crates/tuipalette-tui` | crossterm backend, key decoding, draw loop | `cargo test -p tuipalette-tui` |
| `scripts/` | pty harness, floating adapter, app/dmenu providers, focus-safe verification | `scripts/verify-floating.sh` → `result: ok` |
| `layouts/` | `rofi-like.ron`, `dmenu.ron` (data, no crate names them) | `tuipalette --check --spec layouts/rofi-like.ron` |
| `crates/tuipalette-tui` | library stub that re-exports `ratatui` | `cargo build -p tuipalette-tui` |
| `crates/tuipalette` | the picker: flags, fd handling, exit codes, timing switch | `scripts/pty-check.py ./target/debug/tuipalette …` |
| host toolchain | rustc/cargo 1.95.0 | `cargo --version` |

## Next steps
- [ ] 08 — one check left: `--inline --height 8` inside a real kitty, scrollback and cursor column verified after exit (the floating half passes in `scripts/verify-floating.sh`)
- [ ] 12 — cold-start probe: inner phases already measured (spec-parse 0.6 ms, terminal-init 1.3 ms, first frame 1.7 ms); needs the whole-path number with terminal spawn and the verdict written down
- [ ] 13 — ship `layouts/rofi-like.ron` so there is a document to launch

## How to reproduce the verification
From the repo root on the host: `cargo build --workspace && cargo test --workspace` (rustc 1.95.0; crates resolve from crates.io on the first build). Then run the commands in "What works" in this order.
