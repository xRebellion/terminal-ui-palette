# 14 — Rofi parity inventory

Detail for item 14 of `docs/PLAN.md`. Every rofi call site a user can still reach, what flags it used, and what happened to each.

The item's first wording scoped this to `~/.config/hypr/scripts`, where `grep -w rofi` now finds **nothing** — herdr was the only call site and item 10 migrated it. The sites below are the ones that exist.

| # | call site | rofi flags | status | replacement / reason |
|---|---|---|---|---|
| 1 | `SUPER+R` → `MENU run` | `-show run` | **migrated** | `scripts/run-command.sh` |
| 2 | `SUPER+Space` → `MENU drun` | `-show drun` | **migrated** | `scripts/launch-apps.sh` (+ `layouts/rofi-like.ron`) |
| 3 | `SUPER+Shift+,` → `MENU window` | `-show window` | **migrated** | `scripts/switch-window.sh` |
| 4 | `XF86Calculator` → `MENU calc` | `-show calc` | **benched** | rofi evaluates the query as arithmetic and shows the result live. That is a `Component` (D5), not a document: an expression pane that paints its own evaluation. Nothing else needs it. |
| 5 | wifi list, `rofi-wifi-menu.sh:281` | `-dmenu -i -p … -auto-select` | **migrated** | `layouts/wifi.ron` + `scripts/wifi.sh`; `-auto-select` is the default here — Enter with no move accepts the top row |
| 6 | wifi password, `:172` and `:177` | `-dmenu -password` | **benched** | no masked input yet: an `Input` pane draws its query as typed. A `mask` pane field is the honest fix, and it is a library change, not a document. |
| 7 | wifi placement, `:281` | `-location 0 -opacity 80 -font …` | **impossible by design** | placement belongs to the compositor (D3), opacity and font to the terminal; a picker that steers those is a picker that owns the window. |
| 8 | wifi errors, `:168` | `-e "message"` | **benched** | a message with no list and no input. Today `scripts/wifi.sh` fails through `notify-send` like the herdr script; item 16's `Message` effect is the shape it should take. |

## Evidence for the migrated rows

**1. `run`** — PATH executables as candidates, free text accepted because `--accept-query` is on, exactly rofi's custom input:

```
$ scripts/run-command.sh --list | head -3
4channels
7z
7za
$ scripts/run-command.sh --list | wc -l        # candidates offered
$ printf 'htop -d 5' | tuipalette --spec layouts/dmenu.ron --prompt run --accept-query
htop -d 5                                      # typed text is the answer
```

**2. `drun`** — the item-13 comparison: same keybind, 106 entries from `scripts/apps.sh`, ``tuipalette --check --spec layouts/rofi-like.ron` → `ok — 3 panes, 2 bindings`.

**3. `window`** — the compositor enumerates, the picker chooses, a dispatch focuses:

```
$ scripts/switch-window.sh --list | head -3
π - rebellion	0x557812ace770
Friends List	0x5578130e0850
~ - fish	0x55781345f270
$ # pick the second row in the pty harness
exit=0 value=b'0xbbb\n'                      # the value is the address, not the title
$ WINDOW_PRINT_ONLY=1 … scripts/switch-window.sh   # with a fake answer carrying a title
0x557812ace770
```

The row is `title\taddress`, so the picker hands back the address alone while the eye reads the title — a title with spaces cannot break the choice. The focus itself is one dispatch: `hl.dsp.focus({ window = { address = … } })`.

**5. wifi** — `tuipalette --check --spec layouts/wifi.ron` → `ok — 3 panes, 2 bindings`, and the fake-list run from item 11: typing `cafe` + Enter → `cafe`; `-auto-select` needs nothing because the first row starts selected.

## What is not done

The four binds still point at rofi: `MENU = "rofi -show"` in `~/.config/hypr/hyprland.lua` and four `hl.bind` lines in `conf/binds.lua`. Migrating them is one line each and a reload — deliberately left to the user, since it changes what four of their keys do.
