# 02 — Workspace skeleton and repo hygiene

Detail for item 02 of `docs/PLAN.md`.

## Shape

```
Cargo.toml                 # workspace: members = ["crates/*"], exclude = ["spike"]
crates/tuipalette-core/    # spec, layout, components — no terminal
crates/tuipalette-tui/     # ratatui backend — the only terminal-aware crate
crates/tuipalette/         # the binary
spike/                     # item 01 evidence, outside the workspace
```

Dependency edges, and nothing else yet:

| crate | depends on | why exactly |
|---|---|---|
| `tuipalette-core` | `ratatui-core`, `serde`, `ron` | geometry (`Rect`), the spec's serde derive, the RON adapter. **Not** `ratatui`, not `crossterm` |
| `tuipalette-tui` | `ratatui`, `tuipalette-core` | the backend and the event loop |
| `tuipalette` | `tuipalette-core`, `tuipalette-tui` | argument parsing and process behaviour, from 07 |

The split is not decoration: it is D1 made enforceable. `tuipalette-core` cannot reach a terminal, so a component that wants one fails to compile rather than failing review.

## Decisions taken inside this item

- **`rust-version = "1.88"`** in `[workspace.package]` — the floor set by let-chains and edition 2024, both already used in the spike. Raising it later is a one-line change with a visible blast radius.
- **Version `0.0.0`** until the CLI contract lands in 07. A published version number is a promise; the skeleton makes none.
- **No `clap` yet.** Argument parsing arrives with the flags it has to parse (07), not before.
- **`spike/` excluded from the workspace, kept in the repo.** It still answers "how did this look on day one", and `cargo build --workspace` must not compile it.
- **`Cargo.lock` committed.** This is a binary workspace, so the lockfile is part of the build.

## Evidence

```
git init && rm -rf spike/.git
git commit -m "chore: workspace skeleton for tuipalette (item 02)"   # 9f5698a, 24 files

cargo build --workspace
  Compiling tuipalette-core v0.0.0 … tuipalette-tui v0.0.0 … tuipalette v0.0.0
  Finished `dev` profile … in 11.74s

cargo test --workspace
  Doc-tests tuipalette_core … ok. 0 passed
  Doc-tests tuipalette_tui  … ok. 0 passed

./target/debug/tuipalette
  tuipalette 0.0.0 (core 0.0.0 + tui 0.0.0) — scaffold only, see docs/PLAN.md
  root area would be Rect { x: 0, y: 0, width: 80, height: 24 }
```

The binary's two lines exist to prove the three crates *link*, which a bare `fn main() {}` would not (unused dependencies are invisible to cargo).

## Repo state after this item

- Root is a git repository with one commit; `spike/.git` is gone, so there is no nested repo any more.
- `.gitignore` is `target/` + `*.log`; no build output is tracked.
- Working tree clean.
