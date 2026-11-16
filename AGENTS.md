# terminal-ui-palette — agent notes

The documents in `docs/` are this project's memory, and every turn that changes
state keeps them true rather than re-deriving them later:

- `docs/STATUS.md` — the tracker. Only what the next step rests on; trim it as
  items finish. Ordered `- [ ]` checkboxes, each naming the plan item it advances.
- `docs/DONE.md` — the ledger, append-only, newest first. One line per finished
  item, with the command that proved it.
- `docs/PLAN.md` — the goal record. A work item per piece of work, a
  `· done YYYY-MM-DD` suffix only once its checkable command passed. No checkboxes.
- `docs/DECISIONS.md` — append-only `D<n>` entries for decisions that bind more
  than one item; a decision local to one item belongs in its `docs/specs/NN-*.md`.

Never mark work done without running its checkable command, and refresh
`Last updated` in every file you touch. Build and test with the host toolchain
(`cargo test --workspace`) — the agent sandbox image has no usable Rust compiler.
