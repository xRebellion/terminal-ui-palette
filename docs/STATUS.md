# terminal-ui-palette — Status
Last updated: 2026-11-16

## What works
- RON spec → layout rects → component render, headless — `cd spike && cargo test` → `panes_land_in_their_rects ... ok`
- Action routing, per-pane override and explicit unset — `cd spike && cargo test` → `per_pane_override_wins_and_explicit_none_unsets ... ok`
- Spec errors carry line and column — `cd spike && cargo test` → `malformed_spec_reports_a_position ... ok`
- Whole suite — `cd spike && cargo test` → `4 passed; 0 failed`
- Interactive path under a pty: renders the frame, accepts, cancels, restores the screen — `bash spike/tty-check.sh` → `exit=1` on esc, `exit=0` with `firefox` on stdout on enter

## Known broken and believed
- **Believed:** the draw loop wastes a frame per 200 ms poll timeout (each idle tick redraws); would confirm by counting frames in `spike/tty-check.sh` output, and it must be fixed before item 09 measures key-to-paint.
- **Believed:** exec-to-first-paint will sit well inside 100 ms — would confirm by the timing switch planned in item 12.

## State of what the project touches
| what | state now | checked with |
|---|---|---|
| `spike/` | builds, 4 tests green, one binary + `spec.ron` + `tty-check.sh` | `cd spike && cargo test`; `bash spike/tty-check.sh` |
| repo root | not a git repo; `spike/.git` nested from `cargo init` | `git -C spike rev-parse --show-toplevel` |
| host toolchain | rustc/cargo 1.95.0 | `cargo --version` |

## Next steps
- [ ] 02: `git init` at the root, delete `spike/.git`, create `tuipalette-core`/`tuipalette-tui`/`tuipalette`
- [ ] 03: RON schema, layout resolution and theme tokens in `tuipalette-core`

## How to reproduce the verification
On the host: `cd ~/Code/hobby/terminal-ui-lib/spike && cargo test`. Crates resolve from crates.io on first build (ratatui 0.30.2, ron 0.12.3, serde 1). Then run the commands in "What works" in this order.
