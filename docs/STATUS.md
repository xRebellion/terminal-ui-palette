# terminal-ui-palette — Status
Last updated: 2026-11-16

## What works
- Eight items landed, the workspace builds and its suites pass — `cargo test --workspace` → `78 passed, 0 failed`
- A RON or JSON document parses, validates, resolves to pane rects and deserializes its key map — `cargo test -p tuipalette-core --test spec` → `16 passed`
- Focus, bubbling, emit and key-map override/unset work headless — `cargo test -p tuipalette-core --test dispatch` → `14 passed`
- The built-ins draw from their pane kind, pinned by snapshots — `cargo test -p tuipalette-core --test components` → `14 passed`
- The picker speaks the dmenu contract: items on stdin, keys from the terminal, one value out — `scripts/pty-check.py ./target/release/tuipalette layouts/dmenu.ron '\x1b[B\r'` → `exit=0 value=b'herdr\n'`; `'\x1b'` → `exit=130 value=b''`
- Streaming, ranking and highlighting work end to end — 60 000 `find /usr` items, typed `libx`, enter → `/usr/share/licenses/libxml2` in 0.97 s
- **Floating host, on a real compositor** — `scripts/verify-floating.sh` → `class tuipalette-verify, floating, 541x281 px (=60x14 cells), special:tuipalette-verify`, `focus before/after` unchanged, `value: beta`, `windows left: 0`, `result: ok`
- **Inline host, in a real kitty** — the picker's region is wiped and the cursor parked at its first cell, so the next output starts clean: screen after exit showed blank rows then `beta`, `AFTER-LINE rc=0`, `MARK>hey`, `GOT[hey]`; scrollback contained no frame characters (`grep -c 'select\|enter choose'` → `0`)
- The timing switch reports its phases — `TUIPALETTE_TIMING=1` → `spec-parse 0.3ms, terminal-init 1.2ms, first-frame 2.0ms` (release)

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
| `layouts/` | `rofi-like.ron`, `dmenu.ron` — data no crate names | `tuipalette --check --spec layouts/rofi-like.ron` → `ok — 3 panes, 2 bindings` |
| `scripts/` | pty harness, floating adapter, app/dmenu providers, focus-safe verification | `scripts/verify-floating.sh` → `result: ok` |
| `~/.config/hypr/conf/windowrules.lua` | two `tuipalette` rules (float+center; verify on a hidden special workspace) — **+21 lines, uncommitted in `dotfiles_private`** | `git diff --stat` in `~/dotfiles_private`; `hyprctl configerrors` empty |
| host toolchain | rustc/cargo 1.95.0 | `cargo --version` |

## Next steps
- [ ] 13 — close the rofi side-by-side: same entry point with `PICKER=rofi` and `PICKER=tuipalette`, launched on a special workspace so nothing appears under the user
- [ ] 12 — whole-path latency: inner phases are 0.3/1.2/2.0 ms; still missing the number with terminal spawn, and the written verdict
- [ ] 10 — migrate `herdr-sessions.sh` (edits a live script; needs the user's word)
- [ ] 14 — rofi parity inventory (declares `after: 10, 13`, so it cannot start before both)
- [ ] 15 — icon slots: width from the provider, placeholder fallback, `--no-icons`
- [ ] 11 — spec reference and component guide
- [ ] 09 — latency verdict (rests on 12's numbers)

## How to reproduce the verification
From the repo root on the host: `cargo build --workspace && cargo test --workspace` (rustc 1.95.0; crates resolve from crates.io on the first build). Then run the commands in "What works" in this order. The two host checks need a live session: `scripts/verify-floating.sh` for the compositor, and a kitty window running `printf 'alpha\nbeta\n' | tuipalette --inline --height 6` for the inline one.
