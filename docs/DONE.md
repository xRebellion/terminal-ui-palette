# terminal-ui-palette — Done
Last updated: 2026-11-16

## Completed
- 2026-11-16 — Item 04, component model and router: `Component`/`Paint`/`Reaction` over a plain buffer, focus ring in draw order, bubbling, `Emit`, per-pane key maps with an explicit unset, press-only keys — `cargo test -p tuipalette-core --test dispatch` → `14 passed`; `--check` on `keys: { "ctrl-": … }` → exit 1 naming the path · spec: `specs/04-component-dispatch.md`
- 2026-11-16 — Item 03, spec + layout + theme tokens: RON and JSON adapters, size grammar, field-path validation, `--check` — `cargo test --workspace` → `16 passed` (core spec) + `5 passed` (CLI) + `2 passed` (unit); `--check` on a bad size → exit 1 naming `layout.sizes[0]` · spec: `specs/03-spec-and-layout.md`
- 2026-11-16 — Item 02, workspace skeleton: three crates, root git repo, nested `spike/.git` removed — `cargo build --workspace && cargo test --workspace` → `Finished` + `0 passed` doc-tests, binary prints `Rect { width: 80, height: 24 }` · spec: `specs/02-workspace.md`
- 2026-11-16 — Item 01 tty check: the interactive path renders, accepts, cancels and restores — `bash spike/tty-check.sh` → `esc` exit 1, `enter` exit 0 with `firefox` on stdout and `\e[?25h` after it · spec: `specs/01-spike.md`
- 2026-11-16 — Item 01, feasibility spike: RON spec → resolved rects → Component dispatch, four tests — `cd spike && cargo test` → `test result: ok. 4 passed; 0 failed` · spec: `specs/01-spike.md`
