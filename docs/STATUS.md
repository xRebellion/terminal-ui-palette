# terminal-ui-palette — Status
Last updated: 2026-11-16

## What works
- Three crates build and link as one workspace — `cargo build --workspace` → `Finished`
- A RON or JSON document parses, validates, resolves to pane rects, and deserializes its key map — `cargo test -p tuipalette-core --test spec` → `16 passed`

## Known broken and believed
- Nothing broken, nothing awaiting confirmation that bears on item 04.

## State of what the project touches
| what | state now | checked with |
|---|---|---|
| repo root | git, clean tree, items 01–03 committed | `git status --porcelain`; `git log --oneline` |
| `crates/tuipalette-core` | spec, layout, error, action; 16 tests | `cargo test -p tuipalette-core --test spec` |
| `crates/tuipalette-tui` | library stub that re-exports `ratatui` | `cargo build -p tuipalette-tui` |
| `crates/tuipalette` | `--check` only; no picker yet | `./target/debug/tuipalette --check --spec crates/tuipalette-core/tests/fixtures/launcher.ron` |
| host toolchain | rustc/cargo 1.95.0 | `cargo --version` |

## Next steps
- [ ] 04 — `Component`/`Reaction` in core, focus ring, action router, key-map merge over `UiSpec`
- [ ] 12 — cold-start probe: exec → first frame with spawn excluded (unblocked since 02; de-risks 09)
- [ ] 05 — built-in components (input, menu, text, status) with `insta` snapshots (after 04)

## How to reproduce the verification
From the repo root on the host: `cargo build --workspace && cargo test --workspace` (rustc 1.95.0; crates resolve from crates.io on the first build). Then run the commands in "What works" in this order.
