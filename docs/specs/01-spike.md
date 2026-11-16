# 01 — Feasibility spike

Detail for item 01 of `docs/PLAN.md`. Throwaway evidence, kept until 02 lands.

## What exists

`spike/` — one binary crate, `ratatui 0.30.2` + `ron 0.12.3` + `serde 1`, ~640 lines:

| file | role |
|---|---|
| `spec.ron` | the screen as data: window/theme, a `Vertical` split tree, three pane declarations, a global key map, per-pane overrides |
| `src/spec.rs` | RON → `UiSpec` (`deny_unknown_fields`), split tree → `Vec<(id, Rect)>`, `fill/N/N%/min:/max:` grammar, hex colours |
| `src/action.rs` | `Action` enum, key-string parser (`ctrl-j`, `tab`, `esc`), per-pane map over global map with `None` = explicit unset |
| `src/component.rs` | `Component { id, focusable, render, update -> Reaction, value }`; `InputLine`, `Menu`, `TextPane` |
| `src/app.rs` | focus ring, dispatch with bubbling, draw pass, four tests |
| `src/main.rs` | `ratatui::init()` → event loop → `restore()`, prints the chosen value or exits 1 |

## Evidence

```
cd spike && cargo test
test app::tests::malformed_spec_reports_a_position ... ok
test app::tests::per_pane_override_wins_and_explicit_none_unsets ... ok
test app::tests::down_down_enter_yields_the_third_item ... ok
test app::tests::panes_land_in_their_rects ... ok
test result: ok. 4 passed; 0 failed
```

Host toolchain: rustc 1.95.0, cargo 1.95.0. Run on the host — the sandbox guest for this session was Alpine/musl with no C toolchain, and rustc there could not link proc-macro dylibs.

## What it settled

- **RON expresses the schema.** Each layout node is a struct variant (`Vertical(sizes: [...], children: [...], spacing: N)`), a leaf is a newtype variant (`Pane("query")`), and the key maps are `BTreeMap<String, Option<Action>>` — so D8's explicit unset is literally `"ctrl-j": None`, no escape hatch needed.
- **The split tree resolves to the panes' rects.** The test asserts each pane's top-left cell is a border corner and that the menu's first item sits inside the menu's rect.
- **Components stay out of the terminal.** They render into `&mut Buffer`, never `Frame`; the whole draw pass runs headless under `TestBackend` (D1 validated).
- **Bubbling works as designed.** `down, down, enter` is dispatched while the *input* pane holds focus; the menu handles it after the input ignores it, and emits `Invoke("run:herdr")`, which the app turns into the result.
- **Per-pane override beats global, and `None` stops the lookup** rather than falling through to the global map.
- **A broken spec names its position.** Two failures covered: a syntax error and an unknown field, both reported as `line N, column M`.

## What surprised us (small, real)

- `ron 0.12`'s error type is `SpannedError { code, span }`; the position is `e.span.start.line / .col`, not `e.position`.
- `ratatui 0.30`'s `Terminal::draw` returns `Result<_, B::Error>`, so a generic `run<B: Backend>` needs `io::Error: From<B::Error>` or a concrete type. The idiomatic fix is `ratatui::DefaultTerminal` — one alias, no bound.
- Three compile errors in total, all of them the two above plus an import; nothing about the design.

## TTY check (interactive path, verified)

Run through `spike/tty-check.sh`, which pipes keys into a pty (`script -q -e -c "stty rows 22 cols 72; target/debug/ratatui_spike spec.ron" /tmp/out.ts`):

| case | keys | exit | observed |
|---|---|---|---|
| cancel | `esc` | 1 | three pane titles on screen, alt-screen entered and left |
| accept | `enter` | 0 | full frame rendered, `firefox` on stdout, `\e[?25h` after it |
| navigation | `↓ ↓ ↓ ↑` then `esc` | 1 | no crash, no stray process |

The captured frame is machine-extracted, not hand-drawn (`/tmp/tty-accept.value` parsed by a small CSI interpreter, 72×22). An earlier revision of this excerpt was trimmed by hand and its right edges did not line up — the fix is to paste the parsed capture, never to retype a box.

```
┌ launcher ────────────────────────────────────────────────────────────┐
│┌ search ────────────────────────────────────────────────────────────┐│
││▏                                                                   ││
│└────────────────────────────────────────────────────────────────────┘│
│┌ results ───────────────────────────────────────────────────────────┐│
││› firefox                                                           ││
││  kitty                                                             ││
││  herdr                                                             ││
```

… sixteen more rows of the results pane, then:

```
│└────────────────────────────────────────────────────────────────────┘│
│┌ status ────────────────────────────────────────────────────────────┐│
└──────────────────────────────────────────────────────────────────────┘
```

Three things the harness taught us:

- **A pty with no size draws nothing.** `script` inherits the caller's size; with no terminal behind it the app gets 0x0 and every frame is empty. `stty rows 22 cols 72` inside the pty is the fix — worth remembering for item 12's measurements, which must state the geometry they measured at.
- **The draw loop runs on the poll timeout.** Each 200 ms `event::poll` timeout still redraws (visible as repeated style-reset groups in the capture). Harmless now; wrong for item 09, which measures key-to-paint.
- **A `Length(1)` pane is all border and no content.** The spec's `sizes: ["3", "fill", "1"]` gives the status pane one row, so its inner area is zero-height and the text never appears. The schema needs to say so out loud (a pane is `Length(n+2)` if you want `n` lines) — item 11's reference doc owes that sentence.

## Not covered here (later items)
- Streaming stdin, the matcher, icons, inline host, JSON in/out, and the shipped layout.
- `spike/` still carries its own `.git` from `cargo init`; 02 removes it.

## Sandbox note

This session's guest was Alpine 3.24 (musl), 567 MB disk, no `cc`/`gcc`. A musl rustup toolchain installed and compiled, but rustc could not link proc-macro dylibs (`-lgcc_s`, then a static-musl `.so` segfaulting the compiler, then a 52 MB-free root that cannot hold gcc). The verification above therefore ran on the host via `hbash`.
