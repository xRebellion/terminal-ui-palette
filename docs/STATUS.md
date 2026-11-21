# terminal-ui-palette — Status
Last updated: 2026-11-21

## What works
- The workspace builds and every suite passes — `cargo test --workspace` → `141 passed, 0 failed` (core 115, tui 14, bin 12)
- All four shipped documents validate, so a schema change has a cheap gate — `for f in layouts/*.ron; do tuipalette --check --spec $f; done` → `clipboard.ron: ok — 6 panes, 2 bindings`, `dmenu.ron: ok — 3 panes, 1 binding`, `rofi-like.ron: ok — 3 panes, 2 bindings`, `wifi.ron: ok — 3 panes, 2 bindings`
- The dmenu contract is driven without a terminal emulator: items in, keys, one value out — `scripts/pty-check.py ./target/release/tuipalette layouts/dmenu.ron '\x1b[B\r' 'a\nb\nc'` → `exit=0 value=b'b\n'` (the harness takes binary, spec, keys **and items**; with `PTY07_ITEMS_FILE` set the items argument is still required and ignored)
- A long list says where in it you are, in whichever of two styles the document asks for: `scrollbar: true` on a `Menu` draws a proportional thumb in the pane's last column and `scrollbar_style` picks the drawing — `Background` (the default: `scrollbar_bg`/`scrollbar` as cell backgrounds, the way tmux and helix draw one) or `Glyph` (`░`/`█`, the only style a text snapshot shows); off by default so no shipped document moved — `cargo test -p tuipalette-core --test components` → `28 passed`, 12 snapshots including `menu_scrollbar`/`menu_scrollbar_end`, and a live 200-row run puts the thumb on the same row in both styles (`....T_______..` → `....___T____..` after 100 `Down`s)
- It also says how many rows it found: `kind: Counter` (`format`, `align`) is written by the picker after every refilter — `2/4` on a typed query; the clipboard layout carries it at the right end of the query row (moved there from a header row in `500bdfa`) and keeps every digit of a full history — `the_clipboard_count_keeps_every_digit_of_a_full_history` → `250/250`, `133/250`, `1000/1000` at 106x26, and the same three live in tmux (it drew `250/25` before the 2026-11-21 review fix); a counter with no room cuts from the left (`…0/250`)
- Three of four rofi binds are the picker's, committed in the dotfiles (`02c34c0`): `grep -n TUIPALETTE ~/dotfiles_private/hypr/conf/binds.lua` → `R`, `Space`, `SHIFT + COMMA`; `XF86Calculator` is still `rofi -show calc`
- The floating rule is installed and committed — `grep -c tuipalette-floating ~/dotfiles_private/hypr/conf/windowrules.lua` → `1`
- The clipboard picker is on `SUPER+Shift+V` (D21; dotfiles `hypr/conf/binds.lua`): `hyprctl -j configerrors` → none, a new `exec_cmd` handler on SUPER+SHIFT+V; `scripts/clipboard.sh --list` → 25 live entries; `PTY07_ITEMS_FILE=<that list> pty-check.py … layouts/clipboard.ron '\x1b[B\r' '' --in json` → `13429`. The copy + paste into the previously focused window was confirmed by the user on a real `SUPER+Shift+V` press (2026-11-21; not driven by the agent). PNG rows show the screenshot itself in kitty (25, D22): live capture of the real picker shows the selected image, swaps it on Down, drops it on a text row and after Escape (`exit=130`, clean window); `--images none` shows `[[ binary data … ]]`
- Providers answer: `scripts/run-command.sh --list` → `4671`, `scripts/apps.sh` → `106`, `scripts/switch-window.sh --list` → `9` (live clients, varies), `scripts/clipboard.sh --list` → `25` (varies)
- Typed queries rank a literal hit above a scattered one, and usage still breaks ties after typing (18, D20) — `examples/rank.rs` on the real app list → `kn`: Arknights: Endfield, Arknights, Konsole (was Konsole first); `PTY07_ITEMS_FILE=apps.tsv pty-check.py ./target/release/tuipalette layouts/rofi-like.ron 'kn\r' ''` → `…lutris:rungameid/12` (was `konsole`)
- The mouse works where a document opts in (`window: (mouse: true)`; `rofi-like` and `clipboard` do): hover tints without selecting; a click accepts in one go (`click: Accept`, what both shipped layouts use, as rofi does) or selects with a second click accepting (`click: Select`, the default); the wheel moves three rows, a click on the query focuses it — `PTY07_ITEMS_FILE=rows.txt pty-check.py ./target/release/tuipalette layouts/rofi-like.ron '\x1b[<0;10;7M' ''` over 200 rows (`seq -f 'row %03g' 1 200`) → one click `row 003`, 10 notches + Enter `row 031`; capture released on Escape (`ESC[?1000l`); `dmenu.ron` never enables it

## Known broken and believed
- **Believed:** a panic mid-preview deletes the picture (`Pictures` is a `Drop` guard, the same mechanism as the mouse capture; not provoked) — would confirm with a test build that panics after the first frame. Non-kitty terminals under `--images auto` show text (believed from `KITTY_WINDOW_ID`; not run). Blocks next step: no
- **Known gap:** enter with no match does nothing unless the caller passes `--accept-query` — `pty-check.py ./target/release/tuipalette layouts/dmenu.ron 'zzz\r' 'a\nb'` → `exit=-1 value=b''` after the harness's 10 s, against `exit=0 value=b'zzz\n'` with the flag. A document cannot decide it yet; that is 16's `rules`. Blocks next step: no
- **Thin cover:** `--out-file` is exercised end to end only by `scripts/verify-floating.sh`; there is no smaller test naming the write path. Blocks next step: no
- **Believed:** `scripts/apps.sh` mis-lists applications on a multi-locale system (first `Name=`, no `TryExec`) — would confirm by running it under `LANG=de_DE.UTF-8` against `rofi -show drun`

## State of what the project touches
| what | state now | checked with |
|---|---|---|
| repo | `main` at `origin/main` on `git@github.com:xRebellion/terminal-ui-palette.git` (private), clean and pushed | `git status -sb` → `## main...origin/main`; `git ls-remote origin main` |
| `crates/tuipalette-core` | spec, layout, keys, router, six built-ins + `Detail` and `Counter`, the `Menu` scrollbar in both styles, `Rule`, `window.padding`, `Detail` wrap, `Image` pane, mouse, ranking, matcher, picker — 115 tests | `cargo test -p tuipalette-core` |
| `crates/tuipalette-tui` | crossterm backend, key decoding, draw loop, inline teardown, mouse capture, the `Graphics` trait and its kitty backend — 14 tests | `cargo test -p tuipalette-tui` |
| `crates/tuipalette` | the binary: flags, fd handling, exit codes, `--check`, `examples/minimal.rs` — 11 tests | `cargo test -p tuipalette`; `cargo build --examples` |
| `layouts/` | four documents — `rofi-like`, `dmenu`, `wifi`, `clipboard` (6 panes, the only one with a counter, rules, padding and an `Image` preview) — all `--check` clean, no crate naming them | the `--check` loop above |
| `scripts/` | providers and the floating adapter; the counts above | `--list` on each provider |
| cliphist history | 25 rows (moves with use), capped at 250 (`~/.config/cliphist/config` → `max-items 250`), both `wl-paste` watchers running | `cliphist list \| wc -l`; `pgrep -a wl-paste` |
| `~/.config/hypr` (symlink into `~/dotfiles_private/hypr`) | the four picker binds (R, Space, SHIFT+COMMA, SHIFT+V) and the window rule committed (`15af83c`); the dotfiles repo is **85 commits ahead of its remote, unpushed** pending the user; `XF86Calculator` is still rofi | `git -C ~/dotfiles_private status --porcelain -- hypr` → no lines |
| dependencies | 103 unique crates, unchanged by item 25 | `cargo tree -e normal --prefix none \| sort -u \| wc -l` |
| build limits | `.cargo/config.toml`: 6 jobs, rustc under `nice` | a clean build peaks at 6 rustc (`pgrep -c -x rustc`) |
| image cache | `$XDG_RUNTIME_DIR/tuipalette-clipboard/<id>.png`, mode 700, trimmed to live ids each run | `ls $XDG_RUNTIME_DIR/tuipalette-clipboard` |
| host toolchain | rustc/cargo 1.95.0 | `cargo --version` |

## Next steps
- [ ] 16 — rules: conditional actions and document transitions; it blocks 17, and it is where accept-on-empty becomes data
- [ ] 17 — revisit herdr and the wifi flow against `docs/specs/17-revisit.md`
- [ ] no plan item covers `XF86Calculator` (`rofi -show calc`, benched in 14 for want of an expression component), and the goal says no rofi call remains on a keybind — add an item or narrow the goal

## How to reproduce the verification
From the repo root on the host: `cargo build --release --workspace && cargo test --workspace` (rustc 1.95.0; crates come from crates.io on the first build). Then run the commands in "What works" in this order. `scripts/pty-check.py` takes four positional arguments — binary, spec, keys, items — plus any picker flags after them; with `PTY07_ITEMS_FILE` set, pass `''` as the items. The pty checks need no display; the floating one needs a live session, where `scripts/verify-floating.sh` refuses to run unless its verify rule is installed. `scripts/clipboard.sh` needs a non-empty cliphist history to show rows.
