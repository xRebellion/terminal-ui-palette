# terminal-ui-palette — Status
Last updated: 2026-11-21

## What works
- The workspace builds and every suite passes — `cargo test --workspace` → `104 passed, 0 failed` (core 89, tui 4, bin 11)
- All four shipped documents validate, so a schema change has a cheap gate — `for f in layouts/*.ron; do tuipalette --check --spec $f; done` → `clipboard.ron: ok — 6 panes, 2 bindings`, `dmenu.ron: ok — 3 panes, 1 binding`, `rofi-like.ron: ok — 3 panes, 2 bindings`, `wifi.ron: ok — 3 panes, 2 bindings`
- The dmenu contract is driven without a terminal emulator: items in, keys, one value out — `scripts/pty-check.py ./target/release/tuipalette layouts/dmenu.ron '\x1b[B\r'` → `exit=0 value=b'b\n'`
- A long list says where in it you are, in whichever of two styles the document asks for: `scrollbar: true` on a `Menu` draws a proportional thumb in the pane's last column and `scrollbar_style` picks the drawing — `Background` (the default: `scrollbar_bg`/`scrollbar` as cell backgrounds, the way tmux and helix draw one) or `Glyph` (`░`/`█`, the only style a text snapshot shows); off by default so no shipped document moved — `cargo test -p tuipalette-core --test components` → `28 passed`, 12 snapshots including `menu_scrollbar`/`menu_scrollbar_end`, and a live 200-row run puts the thumb on the same row in both styles (`....T_______..` → `....___T____..` after 100 `Down`s)
- It also says how many rows it found: `kind: Counter` (`format`, `align`) is written by the picker after every refilter — `2/4` on a typed query; the clipboard layout's header row carries it and keeps every digit of a full history — `the_clipboard_count_keeps_every_digit_of_a_full_history` → `250/250`, `133/250`, `1000/1000` at 106x26, and the same three live in tmux (it drew `250/25` before the 2026-11-21 review fix); a counter with no room cuts from the left (`…0/250`)
- Three of four rofi binds are the picker's, committed in the dotfiles (`02c34c0`): `grep -n TUIPALETTE ~/dotfiles_private/hypr/conf/binds.lua` → `R`, `Space`, `SHIFT + COMMA`; `XF86Calculator` is still `rofi -show calc`
- The floating rule is installed and committed — `grep -c tuipalette-floating ~/dotfiles_private/hypr/conf/windowrules.lua` → `1`
- The clipboard picker works (rows, the preview that follows the selection, the paste) but is **shelved**: `SUPER+Shift+V` still runs `cliphist-fzf.fish` until mouse (21) lands and the image consequence of 20's spec is recorded — the scrollbar (22) and the count (23) are in (D19)
- Providers answer: `scripts/run-command.sh --list` → `4671`, `scripts/apps.sh` → `106`, `scripts/switch-window.sh --list` → `4` (live clients), `scripts/clipboard.sh --list` → `28`

## Known broken and believed
- **Known gap:** enter with no match does nothing unless the caller passes `--accept-query` — `pty-check.py … 'zzz\r'` → no exit within the harness's 10 s, against `exit=0 value=b'zzz\n'` with the flag. A document cannot decide it yet; that is 16's `rules`. Blocks next step: no
- **Known, user-facing — ranking (18):** `kn` in the app picker puts Konsole first and Arknights nowhere near it, although Arknights holds `kn` as a literal substring and has 5 recorded launches (Konsole has none) — `PTY07_ITEMS_FILE=<apps.sh | usage.sh order 2> pty-check.py … layouts/rofi-like.ron 'kn\r' ''` → `konsole`, then `cachyos-kernel-manager`, `kdeconnect-indicator`, `kdeconnect-app` on the next three rows. Two causes: nucleo's word-start bonus on `K`-`o`-`n` beats the adjacent `kn` inside a word, and usage only orders the empty query (`usage.sh` pre-sorts; `Picker::refilter` sorts by score with arrival order as the only tie-break). Blocks next step: no; it is the user's top concern
- **Thin cover:** `--out-file` is exercised end to end only by `scripts/verify-floating.sh`; there is no smaller test naming the write path. Blocks next step: no
- **Believed:** `scripts/apps.sh` mis-lists applications on a multi-locale system (first `Name=`, no `TryExec`) — would confirm by running it under `LANG=de_DE.UTF-8` against `rofi -show drun`

## State of what the project touches
| what | state now | checked with |
|---|---|---|
| repo | `main`, items 20 (spec), 22 and 23 committed; **no remote configured**, so nothing is pushed | `git log --oneline -3`; `git remote -v` → no lines |
| `crates/tuipalette-core` | spec, layout, keys, router, six built-ins + `Detail` and `Counter`, the `Menu` scrollbar in both styles, matcher, picker — 89 tests | `cargo test -p tuipalette-core` |
| `crates/tuipalette-tui` | crossterm backend, key decoding, draw loop, inline teardown — 4 tests | `cargo test -p tuipalette-tui` |
| `crates/tuipalette` | the binary: flags, fd handling, exit codes, `--check`, `examples/minimal.rs` — 11 tests | `cargo test -p tuipalette`; `cargo build --examples` |
| `layouts/` | four documents — `rofi-like`, `dmenu`, `wifi`, `clipboard` (6 panes, the only one with a counter) — all `--check` clean, no crate naming them | the `--check` loop above |
| `scripts/` | providers and the floating adapter; the counts above | `--list` on each provider |
| cliphist history | 28 rows, capped at 250 (`~/.config/cliphist/config` → `max-items 250`), both `wl-paste` watchers running | `cliphist list \| wc -l`; `pgrep -a wl-paste` |
| `~/.config/hypr` (symlink into `~/dotfiles_private/hypr`) | binds and window rule committed at `02c34c0`; the checkout is dirty only in `pi/` files unrelated to this project | `git -C ~/dotfiles_private status --porcelain -- hypr` → no lines |
| host toolchain | rustc/cargo 1.95.0 | `cargo --version` |

## Next steps
- [ ] 20 — the spec is written and decides **no graphics in v1** (`docs/specs/20-images.md`, committed); what is left is the user's choice for D19 — unshelve with image rows as text, or keep fzf for images — recorded as D20, then 20 is done
- [ ] 21 — mouse capture, hover, click and wheel, opt-in via `window: (mouse: true)` — SGR sequences driven through `scripts/pty-check.py`
- [ ] 16 — rules: conditional actions and document transitions; it blocks 17, which blocks 18, and it is where accept-on-empty becomes data
- [ ] 17 — revisit herdr and the wifi flow against `docs/specs/17-revisit.md`
- [ ] 18 — ranking: literal substring before subsequence, then shorter display, then frecency as a documented tie-break — the `kn` → Konsole-over-Arknights case is (a) exactly; declared `after: 17`, which the user may choose to relax (its stated reason is provider scope, and this is core ranking)
- [ ] rewire `SUPER+Shift+V` to `scripts/clipboard.sh` and unshelve it (closes D19) — after D20 and 21
- [ ] no plan item covers `XF86Calculator` (`rofi -show calc`, benched in 14 for want of an expression component), and the goal says no rofi call remains on a keybind — add an item or narrow the goal

## How to reproduce the verification
From the repo root on the host: `cargo build --release --workspace && cargo test --workspace` (rustc 1.95.0; crates come from crates.io on the first build). Then run the commands in "What works" in this order. The pty checks need no display; the floating one needs a live session, where `scripts/verify-floating.sh` refuses to run unless its verify rule is installed. `scripts/clipboard.sh` needs a non-empty cliphist history to show rows.
