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

## What is not verified, and why

The item's original checkable — *side by side with `rofi` on the same keybind* — could not be run without putting rofi on the user's screen, because of how rofi draws on this stack:

- `hyprctl layers -j` shows `rofi` as a **Wayland layer surface** (namespace `rofi`), not a toplevel — with or without `-normal-window`.
- Layer surfaces never appear in `hyprctl clients -j`, so no window rule can park this one on a special workspace, and `grim -T` has no window buffer to capture.
- It maps on the active workspace and takes the keyboard; there is no flag that makes it invisible without also making it useless.

So the comparison is a one-minute manual step, done when convenient:

```
PICKER=rofi          scripts/launch-apps.sh
PICKER=tuipalette    scripts/launch-apps.sh
```

Same key, same entries, one line apart. The tunables that could still differ are the exact pixel geometry (our card is 60x14 cells; rofi's is whatever `config.rasi` says), the sorting (`fzf` in rofi, nucleo here) and the row height. Those are the things to look at on that run, and they are recorded as open in `docs/STATUS.md` rather than claimed here.
