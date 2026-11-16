# 08 — Host adapters: floating window and inline

Detail for item 08 of `docs/PLAN.md`.

## Two hosts, chosen automatically

| stdout | chosen host | what the picker owns |
|---|---|---|
| a terminal | **inline**: an inline viewport at the bottom, no alternate screen | the invoking terminal's lower lines |
| a pipe (a script, a dmenu call) | **fullscreen on `/dev/tty`**, value to the pipe | `/dev/tty` |
| a terminal launched for the occasion | **floating**: `scripts/tuipalette-floating.sh` gives kitty a class, a title and the document's cell geometry | its own window |

## What a floating window changes

Two things break when the picker is not in the caller's terminal:

1. **It has no stdin.** The provider therefore runs *inside* the terminal — `scripts/tuipalette-floating.sh <layout> <provider...>` builds `sh -c 'provider | picker --spec layout'`. That is why `scripts/launch-apps.sh` passes `scripts/apps.sh` as an argument instead of piping into the adapter.
2. **It has no stdout.** The picker's stdout is the pty it draws on. So the adapter names a file, passes `--out-file <path>`, waits for the window to close and prints the file to its own stdout. From the caller's side the contract is unchanged: one value on stdout.

Both are host concerns, so both live in the adapter; the crates learned one flag (`--out-file`) and nothing else.

## The compositor rules

Placement is the compositor's, never the picker's (D3). **Nothing is installed right now** — the picker will tile like any other window until these lines are added to `~/.config/hypr/conf/windowrules.lua`:

```lua
-- the picker as a user sees it: a centred floating card
hl.window_rule({
  name = "tuipalette-floating",
  match = { class = "^(tuipalette)$" },
  float = true,
  center = true,
})

-- what the verification launches: never visible, never focused
hl.window_rule({
  name = "tuipalette-verify",
  match = { class = "^(tuipalette-verify)$" },
  workspace = "special:tuipalette-verify silent",
  float = true,
  no_initial_focus = true,
})
```

The second rule is not a convenience: it is the only thing that keeps the verification off the user's screen. `scripts/verify-floating.sh` checks for it and refuses to launch when it is missing, rather than putting a window on the active workspace.

Size comes from kitty, not from the compositor: the adapter turns `--print-geometry` (`60x14`) into `--override initial_window_width=60c --override initial_window_height=14c`, with `remember_window_size=no` so a previous run cannot win.

### Three ways to make it float, measured

| mechanism | measured | applies when |
|---|---|---|
| **static window rule** (config) | floating at map, size exactly 60x14 cells (541x281 px), centred at [690,426]; no flash | anything launches it — keybind, script, terminal |
| **per-exec rule** `hl.dsp.exec_cmd(cmd, { float = true, center = true })` | floating at map, no flash | only when the *compositor* starts the process, so a script that runs kitty itself cannot use it |
| **dispatch after map** `hl.dsp.window.float()` | tiled at [22,75] for 37 ms, then floating at [485,68], size grew to 950x997 | anything, but it targets the *active* window |

The rule wins on three counts: the adapter stays compositor-agnostic (D3 — the host places the window, the picker does not ask), it is the only option that works when `herdr-sessions.sh` launches kitty itself, and it has no race — the dispatch form leaves a tiled frame on screen for about 37 ms and then floats whatever has focus, which is the wrong window if focus moved.

Installed today: the floating rule only. The verify rule is not installed, which is why `scripts/verify-floating.sh` refuses to run.

## Verification

`scripts/verify-floating.sh` launches through the same adapter a keybind would use, but under the verify rule, so nothing appears and nothing takes focus. It then drives the window through kitty's remote control.

```
$ scripts/verify-floating.sh
client: {"class":"tuipalette-verify","floating":true,"size":[541,281],
         "workspace":"special:tuipalette-verify","mapped":true}
document geometry: 60x14; window pixels: 541	281
focus while open: kitty/4; workspace visible: 0
value: beta
focus before: kitty/4 after: kitty/4; windows left: 0
result: ok
```

541x281 px is 60x14 cells at this font, the window is floating, it is on the hidden workspace, the special workspace was never shown, the user's focus stayed on their own window before, during and after, the chosen value came back out of the adapter, and no window was left behind.

## Inline

`--inline --height N` draws with `Viewport::Inline(N)`: no `\e[?1049h` in the output, the terminal's own cursor is hidden while drawing, and the picker occupies the bottom N lines.

### The bug the kitty run found

The first real run left the frame on screen with the shell's next line printed on top of it:

```
BEFORE-LINE
╭─ select ───────────────────────────────────────────────────────────╮
│b▏beta                                                              │
AFTER-LINE rc=0──────────────────────────────────────────────────────╯
MARK>hey─────────────────────────────────────────────────────────────╮
GOT[hey]─────────────────────────────────────────────────────────────╯
enter choose  esc cancel
```

Ratatui's inline viewport leaves its rows drawn and hands the cursor back at their **first** cell, so everything the shell printed afterwards overwrote the top of the frame while the bottom of it survived. Ratatui's own `Terminal::clear` is not enough on its own: it restores the cursor to where it found it, which is the end of the last drawn row.

The fix is the inline teardown in `crates/tuipalette-tui/src/run.rs`:

```rust
// An inline picker owns the bottom N lines, and ratatui leaves them
// drawn with the cursor back at their first cell — so the shell's next
// line lands on top of a half-erased frame. Wipe the region and park
// the cursor where the next line of output belongs.
let height = options.inline.unwrap_or_default();
let top = crossterm::terminal::size()
    .map(|(_, rows)| rows.saturating_sub(height))
    .unwrap_or_default();
terminal.clear().ok();
terminal.backend_mut().execute(cursor::MoveTo(0, top)).ok();
```

The viewport is the bottom of the terminal, so its first row is `rows - height`; no query of ratatui's own area is needed (it keeps that field private).

### The same run after the fix

```
BEFORE-LINE
(blank ×5)
beta
AFTER-LINE rc=0
MARK>hey
GOT[hey]
```

The region is blank, the cursor sits at its first cell (typing `hey` after `MARK>` landed at column 8 on the right line), the accepted value still reaches stdout — that is `beta` on its own line — and `kitten @ get-text --extent=all | grep -c 'select\|enter choose'` returns `0`, so no frame character survives in the scrollback either. An inline picker leaves no trace, which is what a modal should do.

The rows are blanked rather than collapsed: a terminal cannot remove lines above the cursor without a scroll trick, and fzf-style inline tools leave the same blank band.

## Not in this item

- The side-by-side rofi swap: `scripts/dmenu.sh` and `scripts/launch-apps.sh` each have one `PICKER=` line, but nobody has yet run both pickers from the same keybind and compared (that is item 13's checkable).

## What driving the real window taught us (2026-11-16, during item 14)

Verifying the migrated keybinds meant opening the picker from a real keybind and typing into it. Four things were wrong that the pty harness could not see:

1. **`cat -` swallowed the terminal.** The floating adapter pipes the provider into the window, and `-` was the natural spelling for "my stdin". Inside the terminal `-` names the *terminal*, so the inner `cat` read the keyboard: keystrokes were fed to the picker as items, Escape never arrived, and when the terminal answered ratatui's startup cursor query (DSR, `ESC[6n`) that same `cat` ate the reply — so the picker sometimes stalled ~2 s and died with `The cursor position could not be read within a normal duration`. The adapter now materialises `cat -` from its own stdin into a temp file *before* the window opens, and passes `cat <file>` inside.
2. **Inline was the default whenever stdout was a tty.** That made `--inline` a no-op, and a dedicated floating window drew inline at the bottom of itself. Inline is now opt-in: `--inline` asks for it, nothing else does.
3. **A dead window stayed on screen after every exit.** kitty keeps a window open when its child exits (`close_on_child_death no`); the adapter now passes `--override close_on_child_death=yes`.
4. **The picker did not take focus**, so keystrokes and Escape landed on the window behind it. The compositor rule now sets `stay_focused = true`.

Verified from the real keybinds, typing through the window's own input path:

```
SUPER+R        run    → window, active=tuipalette, query "hto",  ESC → 0 windows
SUPER+Space    drun   → window, active=tuipalette, query "fire", ESC → 0 windows
SUPER+Shift+,  window → window, active=tuipalette, query "fish", ESC → 0 windows
```

And inline-with-a-pipe, the case fix 1 was about, in a real kitty window: `printf … | tuipalette --inline --height 6` → typed `b` appeared in the query, Escape exited `rc=130`, the window closed, and the line was not left drawn.
