# terminal-ui-palette — Status
Last updated: 2026-11-16

## What works
- The workspace builds and every suite passes — `cargo test --workspace` → `88 passed, 0 failed` (core 73, tui 4, bin 11)
- All four shipped documents validate, so a schema change has a cheap gate — `for f in layouts/*.ron; do tuipalette --check --spec $f; done` → `clipboard.ron: ok — 5 panes, 2 bindings`, `dmenu.ron: ok — 3 panes, 1 binding`, `rofi-like.ron: ok — 3 panes, 2 bindings`, `wifi.ron: ok — 3 panes, 2 bindings`
- The dmenu contract is driven without a terminal emulator: items in, keys, one value out — `scripts/pty-check.py ./target/release/tuipalette layouts/dmenu.ron '\x1b[B\r'` → `exit=0 value=b'b\n'`
- The two hooks the next items extend: `Menu` derives its own scroll window (22), and the picker writes a pane it owns on every selection change (23, the `Detail` mechanism) — `cargo test -p tuipalette-core --test components` → `17 passed`, 10 snapshots including both clipboard states
- Three of four rofi binds are the picker's, committed in the dotfiles (`02c34c0`): `grep -n TUIPALETTE ~/dotfiles_private/hypr/conf/binds.lua` → `R`, `Space`, `SHIFT + COMMA`; `XF86Calculator` is still `rofi -show calc`
- The floating rule is installed and committed — `grep -c tuipalette-floating ~/dotfiles_private/hypr/conf/windowrules.lua` → `1`
- The clipboard picker works (rows, the preview that follows the selection, the paste) but is **shelved**: `SUPER+Shift+V` still runs `cliphist-fzf.fish` until 20–23 close its four gaps (D19)
- Providers answer: `scripts/run-command.sh --list` → `4669`, `scripts/apps.sh` → `106`, `scripts/switch-window.sh --list` → `4` (live clients), `scripts/clipboard.sh --list` → `0` because the history is empty right now

## Known broken and believed
- **Known gap:** enter with no match does nothing unless the caller passes `--accept-query` — `pty-check.py … 'zzz\r'` → no exit within the harness's 10 s, against `exit=0 value=b'zzz\n'` with the flag. A document cannot decide it yet; that is 16's `rules`. Blocks next step: no
- **Thin cover:** `--out-file` is exercised end to end only by `scripts/verify-floating.sh`; there is no smaller test naming the write path. Blocks next step: no
- **Believed:** `scripts/apps.sh` mis-lists applications on a multi-locale system (first `Name=`, no `TryExec`) — would confirm by running it under `LANG=de_DE.UTF-8` against `rofi -show drun`

## State of what the project touches
| what | state now | checked with |
|---|---|---|
| repo | all of this committed — README, the example, plan item 24 and this tracker | `git status --porcelain` → no lines |
| `crates/tuipalette-core` | spec, layout, keys, router, five built-ins + `Detail`, matcher, picker — 73 tests | `cargo test -p tuipalette-core` |
| `crates/tuipalette-tui` | crossterm backend, key decoding, draw loop, inline teardown — 4 tests | `cargo test -p tuipalette-tui` |
| `crates/tuipalette` | the binary: flags, fd handling, exit codes, `--check`, `examples/minimal.rs` — 11 tests | `cargo test -p tuipalette`; `cargo build --examples` |
| `layouts/` | four documents — `rofi-like`, `dmenu`, `wifi`, `clipboard` — all `--check` clean, no crate naming them | the `--check` loop above |
| `scripts/` | providers and the floating adapter; the counts above | `--list` on each provider |
| cliphist history | **empty** (0 rows) while both `wl-paste` watchers run, so the clipboard picker lists nothing until something is copied | `cliphist list \| wc -l` → `0`; `pgrep -a wl-paste` → text + image watchers |
| `~/.config/hypr` (symlink into `~/dotfiles_private/hypr`) | binds and window rule committed at `02c34c0`; the checkout is dirty only in `pi/` files unrelated to this project | `git -C ~/dotfiles_private status --porcelain -- hypr` → no lines |
| host toolchain | rustc/cargo 1.95.0 | `cargo --version` |

## Next steps
- [ ] 22 — scrollbar on a `Menu`: `scrollbar: true`, a proportional thumb in the rightmost column, `Theme.scrollbar`/`scrollbar_bg` — snapshot of 5 rows in a 3-row window, no bar when everything fits
- [ ] 23 — `kind: Counter`, `3/49` written by the picker after every refilter, `format` and `align` — snapshot with a typed query, plus the clipboard header showing it
- [ ] 20 — hold the images decision (host-level escape passthrough or keep graphics out) and write `docs/specs/20-images.md` — the first of the four D19 gaps
- [ ] 21 — mouse capture, hover, click and wheel, opt-in via `window: (mouse: true)` — SGR sequences driven through `scripts/pty-check.py`
- [ ] 16 — rules: conditional actions and document transitions; it blocks 17, which blocks 18, and it is where accept-on-empty becomes data
- [ ] 17 — revisit herdr and the wifi flow against `docs/specs/17-revisit.md`
- [ ] 18 — ranking: literal substring before subsequence, then shorter display, then frecency as a documented tie-break
- [ ] rewire `SUPER+Shift+V` to `scripts/clipboard.sh` and unshelve it (closes D19) — after 20, 21, 22 and 23

## How to reproduce the verification
From the repo root on the host: `cargo build --release --workspace && cargo test --workspace` (rustc 1.95.0; crates come from crates.io on the first build). Then run the commands in "What works" in this order. The pty checks need no display; the floating one needs a live session, where `scripts/verify-floating.sh` refuses to run unless its verify rule is installed. `scripts/clipboard.sh` needs a non-empty cliphist history to show rows.
