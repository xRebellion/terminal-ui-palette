# 10 — Migrating one real picker

Detail for item 10 of `docs/PLAN.md`. `~/.config/hypr/scripts/herdr-sessions.sh` ran every menu through one wrapper; that wrapper now goes to tuipalette instead of rofi, and the rest of the 247 lines are untouched.

## What rofi was giving it, and what replaced each

| rofi | now | why |
|---|---|---|
| `-dmenu -i` | the document `layouts/dmenu.ron` | a query line over a list; matching is case-insensitive in both |
| `-no-sort` | `--no-sort` | new: the caller's order carries meaning (sessions, `No` before `Yes`) |
| `-p prompt` | `--prompt` | the same string in the frame title |
| `-mesg hint` | `--message` | new: replaces the status row for that one menu |
| custom input (rofi's default) | `--accept-query` | new: Enter with no match answers with the typed text — the name prompt depends on it |
| `-no-custom` | the default | fixed rows must not invent answers |
| exit 1 on escape | exit 130 | the callers all use `|| exit 0`, so any non-zero cancels |

Three flags did not exist before this item; they are the part of rofi's surface that real call sites actually used.

## Two structural differences, both forced

1. **A floating terminal has no stdin.** The wrapper reads the rows into a temp file and passes `cat <file>` as the provider, because the pipe the callers write to is not the picker's stdin once a window is involved.
2. **Prompt and message flags had to reach the picker through the adapter.** `scripts/tuipalette-floating.sh` now takes picker flags before the provider (`--prompt`, `--message`, `--no-sort`, `--accept-query`, `--print-query`, `--icons`/`--no-icons`), and anything else starts the provider command.

## Verified — the script's own behaviours

A fake picker (`HERDR_PICKER`) and stubs for `herdr` and `kitty` drive every path, so the flow is checked without a window and without touching a real session:

```
1. pick an existing session      -> kitty <herdr> session attach sess1
2. pick '+ new session', typed   -> kitty <herdr> session attach brandnew   (this is --accept-query)
3. delete, confirmed             -> herdr session delete sess1
4. cancel at the first menu      -> exit 0, no attach, no herdr call beyond the list
5. delete, declined              -> exit 0, nothing deleted
6. delete a running session     -> herdr session stop sess2, then session delete sess2
```

All six as shown; `bash -n` passes and the backup is `/tmp/herdr-sessions.sh.bak`.

## Verified — the three new picker behaviours

Through the real binary in a pty (`scripts/pty-check.py`):

```
$ printf 'xxabc\nabc\n' | tuipalette --spec layouts/dmenu.ron        # type abc, enter
abc
$ printf 'xxabc\nabc\n' | tuipalette --spec layouts/dmenu.ron --no-sort
xxabc
$ printf 'b\na\nc\n' | tuipalette ... --accept-query --message 'no such entry'   # type zzz, enter
zzz
$ tuipalette ... --prompt 'herdr session' --message 'Enter attaches'
(the frame is titled "herdr session" and the status row reads "Enter attaches")
```

The first pair is the discriminator: with ranking, `abc` wins; with `--no-sort`, the row the caller put first does.

## Not verified

The real window path: the Hyprland rules are not installed right now (docs/specs/08-hosts.md), so launching the adapter would tile a window on the active workspace rather than float it. The adapter's argument handling was instead checked with `TUIPALETTE_DRY_RUN=1`, which prints the command it would run:

```
kitty --class tuipalette ... sh -c cat /tmp/items.txt | tuipalette --spec layouts/dmenu.ron \
  --out-file /tmp/tuipalette-result.XXXX --prompt herdr\ session --message Enter\ attaches \
  --no-sort --accept-query
```

Binding it and pressing SUPER+SHIFT+Return is the last step, and it needs the floating rule back first.
