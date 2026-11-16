# 09 — Latency verdict and the daemon decision

Detail for item 09 of `docs/PLAN.md`. Two comparisons, because comparing our inner number to rofi's whole path would flatter us, and then one verdict.

## Table A — ours, against the 100 ms budget

Same machine, release build, launched the way a keybind launches it (`hl.dsp.exec_cmd` → kitty → the adapter). Terminal spawn excluded, as D14 says it must be.

| what | number | how |
|---|---|---|
| spec parse → layout resolved | 0.1 ms | `TUIPALETTE_TIMING=1` |
| terminal init (raw mode, inline viewport) | 0.5–0.8 ms | same |
| first frame built, written and flushed | 0.7–1.0 ms | same |
| **exec → first flushed frame** | **1.3–1.9 ms** | sum of the above |
| **key → repaint** (1000 items) | **0.61 ms median, 1.38 ms p95** | `cargo test -p tuipalette-core --test latency -- --nocapture` (debug build, so release is faster) |

The budget is **exec → first flushed frame**, and it is met with roughly **60× headroom**. A keystroke — re-rank 1000 items, rebuild the buffer, diff it — costs well under a millisecond in a debug build.

## Table B — the whole path, against rofi

Same instrument for both: dispatch the exec, poll the compositor until the thing is on screen. Each poll is one `hyprctl` call, so both numbers carry the same ~10 ms bias. rofi is a Wayland layer surface, ours is a toplevel window, so the probe differs while the clock does not.

| run | rofi | tuipalette |
|---|---|---|
| 1 | 43 ms | 117 ms |
| 2 | 42 ms | 127 ms |
| 3 | 45 ms | 132 ms |

Ours is **about 2.8× slower to appear**, and the entire delta is the terminal: rofi draws its own surface from a single process, we ask kitty for a window and kitty's start-up (GPU context, pty, shell) costs ~80–90 ms on its own.

## Verdict

1. **The 100 ms budget holds.** Everything the library controls is under 2 ms, keystrokes included.
2. **D2 stands, with no exception to record.** One process per invocation costs nothing measurable on our side; a resident daemon could only have shortened the terminal spawn, and a daemon cannot pre-spawn a window it does not own.
3. **The whole-path gap is accepted and recorded**, not hidden: rofi appears in ~43 ms, we appear in ~120 ms. That is the price of "the host places the window" (D3) plus a real terminal, and it is paid once per invocation.
4. **The one escape route, named but not taken:** a pre-warmed kitty driven over its remote-control socket ("kitten @ launch") would remove most of the spawn, at the cost of a resident process — exactly the thing D2 rules out. If the gap ever matters more than the model, that is the experiment to run.

## First attempt, kept because it was wrong

The first measurement run reported oor 119–198 ms and I quoted it as a picker that draws. It was not: the compositor-spawned script had a minimal PATH, `tuipalette` was not on it, the adapter exited 127, and kitty mapped its window before the shell inside died. The probe was measuring kitty's spawn and the table said "tuipalette". The numbers above are the retake with `TUIPALETTE_BIN` set and the adapter's log checked. The adapter now refuses to start quietly when the binary is missing.

## Which terminal hosts it

The same measurement, run with two terminals asked for the same 60×14 picker (alacritty and kitty are both installed here):

| run | kitty | alacritty |
|---|---|---|
| 1 | 126 ms | 97 ms |
| 2 | 134 ms | 93 ms |
| 3 | 130 ms | 86 ms |
| **median** | **~130 ms** | **~93 ms** |
| picker's own first frame | 0.4–0.5 ms | 0.5–0.6 ms |

Same keybind path, same document, same 106 entries, same instrument. **Alacritty appears about 37 ms sooner**, and the picker's own work is identical in both — the difference is entirely terminal start-up, which is what Table B said the delta was.

Two caveats, both pushing the same way:

- Alacritty **ignored the geometry**. Its own config asks for 100×30, `-o window.dimensions.columns=60` did not change that, and the window mapped at 800×600 px against kitty's 541×281 — so alacritty drew a ~2.5× larger area and still won.
- The adapter's sizing flags are kitty's (`--override initial_window_width=…c`). Making the host choice a one-variable change would need a small per-terminal flag builder, which does not exist yet.

The larger lever is not the terminal at all: with `--inline` there is **no window to spawn**, so the whole path collapses to the 1.3–1.9 ms in Table A whenever the picker is called from a terminal that is already open.
