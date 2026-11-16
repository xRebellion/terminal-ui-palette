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

Placement is the compositor's, never the picker's (D3). Two rules, both in `~/.config/hypr/conf/windowrules.lua`:

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

Size comes from kitty, not from the compositor: the adapter turns `--print-geometry` (`60x14`) into `--override initial_window_width=60c --override initial_window_height=14c`, with `remember_window_size=no` so a previous run cannot win.

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
