# terminal-ui-palette — Done
Last updated: 2026-11-16

## Completed
- 2026-11-16 — Item 01, feasibility spike: RON spec → resolved rects → Component dispatch, four tests — `cd spike && cargo test` → `test result: ok. 4 passed; 0 failed` · spec: `specs/01-spike.md`
- 2026-11-16 — Item 01 tty check: the interactive path renders, accepts, cancels and restores — `bash spike/tty-check.sh` → `esc` exit 1, `enter` exit 0 with `firefox` on stdout and `\e[?25h` after it · spec: `specs/01-spike.md`
- 2026-11-16 — Item 02, workspace skeleton: three crates, root git repo, nested `spike/.git` removed — `cargo build --workspace && cargo test --workspace` → `Finished` + `0 passed` doc-tests, binary prints `Rect { width: 80, height: 24 }` · spec: `specs/02-workspace.md`
