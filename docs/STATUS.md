# terminal-ui-palette — Status
Last updated: 2026-11-16

## What works
- Nineteen items landed, the workspace builds and its suites pass — `cargo test --workspace` → `88 passed, 0 failed`
- A RON or JSON document parses, validates, resolves to pane rects and deserializes its key map — `cargo test -p tuipalette-core --test spec` → `16 passed`
- Focus, bubbling, emit and key-map override/unset work headless — `cargo test -p tuipalette-core --test dispatch` → `14 passed`
- The built-ins draw from their pane kind, pinned by snapshots, with icon slots whose four fill states keep the text column fixed — `cargo test -p tuipalette-core --test components` → `16 passed`
- The picker speaks the dmenu contract: items on stdin, keys from the terminal, one value out — `scripts/pty-check.py ./target/release/tuipalette layouts/dmenu.ron '\x1b[B\r'` → `exit=0 value=b'herdr\n'`; `'\x1b'` → `exit=130 value=b''`
- Streaming, ranking and highlighting work end to end — 60 000 `find /usr` items, typed `libx`, enter → `/usr/share/licenses/libxml2` in 0.97 s
- **Floating host, on a real compositor** — `scripts/verify-floating.sh` → `class tuipalette-verify, floating, 541x281 px (=60x14 cells), special:tuipalette-verify`, `focus before/after` unchanged, `value: beta`, `windows left: 0`, `result: ok`
- **Inline host, in a real kitty** — the picker's region is wiped and the cursor parked at its first cell, so the next output starts clean: screen after exit showed blank rows then `beta`, `AFTER-LINE rc=0`, `MARK>hey`, `GOT[hey]`; scrollback contained no frame characters (`grep -c 'select\|enter choose'` → `0`)
- A new picker can be written from `docs/spec-reference.md` alone — `layouts/wifi.ron` + `scripts/wifi.sh` were, and `--check` → `ok — 3 panes, 2 bindings`, typed `cafe` + enter → `cafe`
- `herdr-sessions.sh` runs on the picker — the live script opens with its prompt, hint line and caller order, and escape returns `rc=0`; attach, new+typed name, delete+confirm, cancel and decline also driven through a fake picker and stubs; the three rofi-shaped flags checked in a pty
- The three rofi *modes* have equivalents, verified: `run` (`scripts/run-command.sh`, 4669 candidates, free text accepted), `drun` (`scripts/launch-apps.sh`) and `window` (`scripts/switch-window.sh`, address returned); `calc` and `-password` are benched with reasons in `docs/specs/14-parity.md`
- All four rofi keybinds are rewired: `SUPER+R` → `scripts/run-command.sh`, `SUPER+Space` → `scripts/launch-apps.sh`, `SUPER+Shift+,` → `scripts/switch-window.sh`; `XF86Calculator` stays on rofi (benched)
- drun matches Name, GenericName, Exec, Categories and Keywords while showing the Name (rofi parity, D17); `seahorse` finds "Passwords and Keys"
- The clipboard picker migrated from fzf and then **shelved**: rows from cliphist, the selected entry previewed through `Item.detail` + `kind: Detail`, a paste after focus returns, the layout measured against the original — but `SUPER+Shift+V` still runs the fish script until 20–23 land (D19, `docs/specs/19-clipboard.md`)
- The app list opens in 20 ms (was 1276 ms) and both launchers remember what was used (`scripts/usage.sh`)
- The floating host survives a real keybind: focused window, typed query, Escape closes it with no dead window (four bugs fixed — `docs/specs/08-hosts.md`)
- The picker matches rofi on the same keybind — same card, same entries, compared on screen; whole path 117–132 ms against rofi's 42–45 ms, the delta being the terminal's spawn
- The timing switch reports its phases, and the whole cold path fits the budget — `TUIPALETTE_TIMING=1` → `spec-parse 0.1ms, terminal-init 0.5–0.8ms, first-frame 0.7–1.0ms` (**under 2 ms**, terminal spawn 217–495 ms measured separately)

## Known broken and believed
- **Known gap:** enter with no matches does nothing — `Menu` answers `Handled` over an empty list, so a mistyped query gets silence instead of a cancel or the typed text; would fix by making accept-on-empty return the query when `--print-query` is set.
- **Thin cover:** `--out-file` is exercised end to end by `scripts/verify-floating.sh` and by nothing smaller — a refactor of the write path would break the floating host, which that script catches, but there is no unit test that names the behaviour.
- **Believed:** `scripts/apps.sh` mis-lists applications on a multi-locale system (first `Name=`, no `TryExec`); would confirm with `LANG=de_DE.UTF-8` against `rofi -show drun`.

## State of what the project touches
| what | state now | checked with |
|---|---|---|
| repo | clean tree at `d57e8a4` before the item-08 fix, 7 items committed, ~75 tracked files | `git status --porcelain`; `git log --oneline` |
| `crates/tuipalette-core` | spec, layout, keys, router, five built-ins, matcher, picker — 63 tests | `cargo test -p tuipalette-core` |
| `crates/tuipalette-tui` | crossterm backend, key decoding, draw loop, inline teardown | `cargo test -p tuipalette-tui` |
| `crates/tuipalette` | the picker binary: flags, fd handling, exit codes, timing switch | `scripts/pty-check.py ./target/release/tuipalette …` |
| `layouts/` | `rofi-like.ron`, `dmenu.ron`, `wifi.ron` — data no crate names; hints carry navigation | `for f in layouts/*.ron; do tuipalette --check --spec $f; done` → three `ok` lines |
| `scripts/` | pty harness, floating adapter, app/dmenu/window/run providers, focus-safe verification | `scripts/run-command.sh --list | wc -l` → `4669`; `scripts/switch-window.sh --list | wc -l` → `9` |
| `~/.config/hypr/conf/windowrules.lua` | the **floating rule is installed** (verified: `floating: true`, 541x241 px for the 60x12 dmenu document); the verify rule is not, so `scripts/verify-floating.sh` refuses to run | `grep -c tuipalette-floating ~/.config/hypr/conf/windowrules.lua`; `hyprctl clients -j` |
| host toolchain | rustc/cargo 1.95.0 | `cargo --version` |

## Next steps

## How to reproduce the verification
From the repo root on the host: `cargo build --workspace && cargo test --workspace` (rustc 1.95.0; crates resolve from crates.io on the first build). Then run the commands in "What works" in this order. The two host checks need a live session: `scripts/verify-floating.sh` for the compositor, and a kitty window running `printf 'alpha\nbeta\n' | tuipalette --inline --height 6` for the inline one.
