# terminal-ui-palette — Status
Last updated: 2026-11-16

## What works
- Three crates build and link as one workspace — `cargo build --workspace` → `Finished`
- A RON or JSON document parses, validates, resolves to pane rects, and deserializes its key map — `cargo test -p tuipalette-core --test spec` → `16 passed`
- Focus, bubbling, emit and the key-map override/unset work headless — `cargo test -p tuipalette-core --test dispatch` → `14 passed`
- All five built-ins draw from their pane kind, pinned by snapshots — `cargo test -p tuipalette-core --test components` → `14 passed`

## Known broken and believed
- Nothing broken, nothing awaiting confirmation that bears on item 06.

## State of what the project touches
| what | state now | checked with |
|---|---|---|
| repo root | git, clean tree, items 01–03 committed | `git status --porcelain`; `git log --oneline` |
| `crates/tuipalette-core` | spec, layout, key map, router, five built-ins; 44 tests | `cargo test -p tuipalette-core` |
| `crates/tuipalette-tui` | library stub that re-exports `ratatui` | `cargo build -p tuipalette-tui` |
| `crates/tuipalette` | `--check` only; no picker yet | `./target/debug/tuipalette --check --spec crates/tuipalette-core/tests/fixtures/launcher.ron` |
| host toolchain | rustc/cargo 1.95.0 | `cargo --version` |

## Next steps
- [ ] 06 — streaming source, matcher, match highlighting
- [ ] 07 — CLI contract: stdin lines, stdout value, exit codes 0/130/2
- [ ] 08 — host adapters, which is what gives item 12 a frame to measure

## How to reproduce the verification
From the repo root on the host: `cargo build --workspace && cargo test --workspace` (rustc 1.95.0; crates resolve from crates.io on the first build). Then run the commands in "What works" in this order.
