# 12 — Cold-start probe

Detail for item 12 of `docs/PLAN.md`. The question: is a 100 ms first frame reachable, and where do the milliseconds actually go?

## Method

One env-gated switch inside the binary, no separate harness: `TUIPALETTE_TIMING=1` prints each phase to stderr, measured from process start with `Instant::now()` taken in `main`.

Measured on the host, release build, three runs, launched exactly the way a keybind launches it (`hl.dsp.exec_cmd(...)` → kitty → the adapter). Terminal spawn is **excluded** from the phases and reported separately.

## Phase table

| phase | run 1 | run 2 | run 3 | what it covers |
|---|---|---|---|---|
| spec-parse | 0.1 ms | 0.1 ms | 0.1 ms | read the RON, validate, resolve the layout |
| terminal-init | 0.5 ms | 0.5 ms | 0.8 ms | raw mode, inline viewport setup (which sends the cursor query and waits for the reply) |
| first-frame | 0.7 ms | 0.7 ms | 1.0 ms | build the buffer, diff, write and flush to the pty |
| **ours, exec → first flushed frame** | **1.3 ms** | **1.3 ms** | **1.9 ms** | sum of the above |

An earlier release run on the same machine measured 0.3 / 1.2 / 2.0 ms — same shape, cold page cache. The number to quote is **under 2 ms**.

## The spawn, measured for context (item 09's half)

| run | dispatch → the binary's first instruction |
|---|---|
| 1 (cold) | 495 ms |
| 2 | 217 ms |
| 3 | 227 ms |

Compositor dispatch, kitty's process start, GPU context, pty allocation and the shell — all of it the host's cost, and all of it ~150× our own.

## Verdict

**Yes — the 100 ms budget survives, with roughly 60× headroom.** Everything the library controls (parse, terminal init, first frame) costs under 2 ms; a fifth of the budget would still leave room for a 20× regression.

Which also settles the shape of item 09: on this measurement there is nothing for a resident daemon to save. The 100 ms budget is spent on the terminal, not on us, and a daemon cannot pre-spawn a window.
