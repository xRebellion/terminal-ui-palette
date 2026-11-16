# terminal-ui-palette — Status
Last updated: 2026-11-16

## What works
- RON spec → layout rects → component render, headless — `cd spike && cargo test` → `panes_land_in_their_rects ... ok`
- Action routing, per-pane override and explicit unset — `cd spike && cargo test` → `per_pane_override_wins_and_explicit_none_unsets ... ok`
- Spec errors carry line and column — `cd spike && cargo test` → `malformed_spec_reports_a_position ... ok`
- Whole suite — `cd spike && cargo test` → `4 passed; 0 failed`
- Interactive path under a pty: renders the frame, accepts, cancels, restores the screen — `bash spike/tty-check.sh` → `exit=1` on esc, `exit=0` with `firefox` on stdout on enter
- Workspace of three crates builds and links — `cargo build --workspace` → `Finished`; `cargo test --workspace` → all suites ok
- Spec parses and validates in RON and JSON, geometry resolves — `cargo test -p tuipalette-core --test spec` → `16 passed`
- `--check` reports a broken spec with a field path or a line/column — `./target/debug/tuipalette --check --spec <bad>` → exit 1

## Known broken and believed
- **Believed:** the draw loop wastes a frame per 200 ms poll timeout (each idle tick redraws); would confirm by counting frames in `spike/tty-check.sh` output, and it must be fixed before item 09 measures key-to-paint.
- **Believed:** exec-to-first-paint will sit well inside 100 ms — would confirm by the timing switch planned in item 12.

## State of what the project touches
| what | state now | checked with |
|---|---|---|
| `spike/` | excluded from the workspace, still builds on its own, 4 tests green | `cd spike && cargo test`; `bash spike/tty-check.sh` |
| `crates/` | three crates, core	ui → bin, no behaviour yet | `cargo build --workspace` |
| repo root | git repo, one commit `9f5698a`, clean tree; nested `spike/.git` deleted | `git -C . log --oneline`; `git status --short` |
| host toolchain | rustc/cargo 1.95.0 | `cargo --version` |

## Next steps
- [ ] 04: component trait, focus ring, action router, key maps over `UiSpec`
- [ ] 05: built-in components (input, menu, text, status) + `insta` snapshots

## How to reproduce the verification
On the host: `cd ~/Code/hobby/terminal-ui-lib && cargo build --workspace && cargo test --workspace`, then `cd spike && cargo test && bash tty-check.sh`. Crates resolve from crates.io on first build (ratatui 0.30.2, ron 0.12.3, serde 1). Then run the commands in "What works" in this order.
