# terminal-ui-palette — Status
Last updated: 2026-11-16

## What works
- Eleven items landed, the workspace builds and its suites pass — `cargo test --workspace` → `81 passed, 0 failed`
- A RON or JSON document parses, validates, resolves to pane rects and deserializes its key map — `cargo test -p tuipalette-core --test spec` → `16 passed`
- Focus, bubbling, emit and key-map override/unset work headless — `cargo test -p tuipalette-core --test dispatch` → `14 passed`
- The built-ins draw from their pane kind, pinned by snapshots, with icon slots whose four fill states keep the text column fixed — `cargo test -p tuipalette-core --test components` → `16 passed`
- The picker speaks the dmenu contract: items on stdin, keys from the terminal, one value out — `scripts/pty-check.py ./target/release/tuipalette layouts/dmenu.ron '\x1b[B\r'` → `exit=0 value=b'herdr\n'`; `'\x1b'` → `exit=130 value=b''`
- Streaming, ranking and highlighting work end to end — 60 000 `find /usr` items, typed `libx`, enter → `/usr/share/licenses/libxml2` in 0.97 s
- **Floating host, on a real compositor** — `scripts/verify-floating.sh` → `class tuipalette-verify, floating, 541x281 px (=60x14 cells), special:tuipalette-verify`, `focus before/after` unchanged, `value: beta`, `windows left: 0`, `result: ok`
- **Inline host, in a real kitty** — the picker's region is wiped and the cursor parked at its first cell, so the next output starts clean: screen after exit showed blank rows then `beta`, `AFTER-LINE rc=0`, `MARK>hey`, `GOT[hey]`; scrollback contained no frame characters (`grep -c 'select\|enter choose'` → `0`)
- A new picker can be written from `docs/spec-reference.md` alone — `layouts/wifi.ron` + `scripts/wifi.sh` were, and `--check` → `ok — 3 panes, 2 bindings`, typed `cafe` + enter → `cafe`
- The timing switch reports its phases, and the whole cold path fits the budget — `TUIPALETTE_TIMING=1` → `spec-parse 0.1ms, terminal-init 0.5–0.8ms, first-frame 0.7–1.0ms` (**under 2 ms**, terminal spawn 217–495 ms measured separately)

## Known broken and believed
- **Known gap:** enter with no matches does nothing — `Menu` answers `Handled` over an empty list, so a mistyped query gets silence instead of a cancel or the typed text; would fix by making accept-on-empty return the query when `--print-query` is set.
- **Believed:** ¶--out-file¶ is correct only because `scripts/verify-floating.sh` exercises it; it has no unit test, so a refactor could break it silently.
- **Believed:** `scripts/apps.sh` mis-lists applications on a multi-locale system (first `Name=`, no `TryExec`); would confirm with `LANG=de_DE.UTF-8` against `rofi -show drun`.

## State of what the project touches
| what | state now | checked with |
|---|---|---|
| repo | clean tree at `d57e8a4` before the item-08 fix, 7 items committed, ~75 tracked files | `git status --porcelain`; `git log --oneline` |
| `crates/tuipalette-core` | spec, layout, keys, router, five built-ins, matcher, picker — 63 tests | `cargo test -p tuipalette-core` |
| `crates/tuipalette-tui` | crossterm backend, key decoding, draw loop, inline teardown | `cargo test -p tuipalette-tui` |
| `crates/tuipalette` | the picker binary: flags, fd handling, exit codes, timing switch | `scripts/pty-check.py ./target/release/tuipalette …` |
| `layouts/` | `rofi-like.ron`, `dmenu.ron`, `wifi.ron` — data no crate names | `tuipalette --check --spec layouts/wifi.ron` → `ok — 3 panes, 2 bindings` |
| `scripts/` | pty harness, floating adapter, app/dmenu providers, focus-safe verification | `scripts/verify-floating.sh` → `result: ok` |
| `~/.config/hypr/conf/windowrules.lua` | two `tuipalette` rules (float+center; verify on a hidden special workspace) — **+21 lines, uncommitted in `dotfiles_private`** | `git diff --stat` in `~/dotfiles_private`; `hyprctl configerrors` empty |
| host toolchain | rustc/cargo 1.95.0 | `cargo --version` |

## Next steps
- [ ] 13 — one manual minute: `PICKER=rofi` then `PICKER=tuipalette` on the same keybind, compared by eye (rofi is a layer surface here, so it cannot be parked off screen or driven without focus — see `docs/specs/13-rofi-like-layout.md`)
- [ ] 10 — migrate `herdr-sessions.sh` (edits a live script; needs the user's word)
- [ ] 14 — rofi parity inventory (declares `after: 10, 13`, so it cannot start before both)
- [ ] 09 — latency verdict (rests on 12's numbers)

## How to reproduce the verification
From the repo root on the host: `cargo build --workspace && cargo test --workspace` (rustc 1.95.0; crates resolve from crates.io on the first build). Then run the commands in "What works" in this order. The two host checks need a live session: `scripts/verify-floating.sh` for the compositor, and a kitty window running `printf 'alpha\nbeta\n' | tuipalette --inline --height 6` for the inline one.
