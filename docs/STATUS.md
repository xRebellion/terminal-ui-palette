# terminal-ui-palette — Status
Last updated: 2026-11-16

## What works
- Three crates build and link as one workspace — `cargo build --workspace` → `Finished`
- A RON or JSON document parses, validates, resolves to pane rects, and deserializes its key map — `cargo test -p tuipalette-core --test spec` → `16 passed`
- Focus, bubbling, emit and the key-map override/unset work headless — `cargo test -p tuipalette-core --test dispatch` → `14 passed`

## Known broken and believed
- Nothing broken, nothing awaiting confirmation that bears on item 05.

## State of what the project touches
| what | state now | checked with |
|---|---|---|
| repo root | git, clean tree, items 01–03 committed | `git status --porcelain`; `git log --oneline` |
| `crates/tuipalette-core` | spec, layout, key map, component model, router; 30 tests | `cargo test -p tuipalette-core` |
| `crates/tuipalette-tui` | library stub that re-exports `ratatui` | `cargo build -p tuipalette-tui` |
| `crates/tuipalette` | `--check` only; no picker yet | `./target/debug/tuipalette --check --spec crates/tuipalette-core/tests/fixtures/launcher.ron` |
| host toolchain | rustc/cargo 1.95.0 | `cargo --version` |

## Next steps
- [ ] 05 — built-in components (input, menu, text, status) drawn through `Paint`, with `insta` snapshots
- [ ] 12 — cold-start probe: exec → first frame, spawn excluded (needs a frame to measure, so after 05)
- [ ] 06 — streaming source, matcher, match highlighting (after 05)

## How to reproduce the verification
From the repo root on the host: `cargo build --workspace && cargo test --workspace` (rustc 1.95.0; crates resolve from crates.io on the first build). Then run the commands in "What works" in this order.
