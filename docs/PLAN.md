# terminal-ui-lib — Plan
Last updated: 2026-11-16

## Contents
- Goal
- Non-goals
- Scope
- Work
  - 01 — Feasibility spike: components, actions, layout from a spec · done 2026-11-16
  - 02 — Workspace skeleton and repo hygiene · done 2026-11-16
  - 03 — Spec schema in RON, layout resolution, theme tokens · done 2026-11-16
  - 04 — Component trait, focus ring, action router, key maps · done 2026-11-16
  - 05 — Built-in components and snapshot tests · done 2026-11-16
  - 06 — Streaming source, matcher, match highlighting · done 2026-11-16
  - 07 — CLI contract: dmenu-compatible stdin/stdout, exit codes · done 2026-11-16
  - 08 — Host adapters: floating window and inline · done 2026-11-16
  - 12 — Cold-start probe: is 100 ms reachable at all? · done 2026-11-16
  - 09 — Latency verdict and the daemon decision · done 2026-11-16
  - 10 — Migrate one real picker end to end · done 2026-11-16
  - 13 — Ship a rofi-like layout · done 2026-11-16
  - 14 — Rofi parity inventory and empirical check · done 2026-11-16
  - 15 — Icon slots and their fallback · done 2026-11-16
  - 11 — Spec reference and component guide · done 2026-11-16
  - 16 — Conditional actions and document transitions
- Decisions
- Risks
- Open questions
- Acceptance criteria

## Goal
A Rust library plus one reference binary that builds rofi-class pickers inside any terminal emulator: the screen is described by a RON document (panes, splits, sizes, theme tokens, icon slots, per-pane key overrides), components react to a flat `Action` rather than to raw key events, and the process speaks a dmenu-compatible stdin/stdout contract so a shell script calls it exactly where it calls `rofi -dmenu` today. The library contains no rofi concepts: the rofi-compatible picker is a shipped layout (`layouts/rofi-like.ron` plus provider scripts) and one of several customers of the schema. Two hosts, no compositor assumption: a floating terminal window placed by the compositor when launched from a keybind, an inline viewport when invoked from a terminal. Done when the app-launch bind and `herdr-sessions.sh` run on the shipped layout with a cold start at or under 100 ms, and no rofi invocation remains on a Hyprland keybind.

## Non-goals
- Not a rofi fork, and not a drop-in for `rofi`'s CLI. Compatibility is deliberately limited to the dmenu-style contract and the two modi the user actually runs.
- No muxer popups: no `tmux display-popup`, no `zellij` floating panes. Benched, not rejected — there is still no use case for one.
- No image-protocol icon backends in v1 (kitty graphics, sixel, iTerm inline). Icons render as glyphs with a placeholder fallback (15); `ratatui-image`-style rendering is a later, terminal-specific opt-in and never enters `picker-core`.
- Not a general TUI application framework. It hosts pickers: a query line, a result list, static text and layout around them. No editor, no tree, no tabs, no mouse-driven drag.
- Not a Wayland layer-shell or GTK client. Never links a display server, never owns a surface.
- No config-as-code: no Lua/Rhai/Starlark, no conditionals, no expressions, no templating in the spec. Anything dynamic is a provider writing lines to stdin.
- No plugin runtime and no stable external ABI in v1. Extension = adding a `Component` impl and rebuilding; that is a separate project.
- Not an X11 tool, and not a bundle of prebuilt launchers. Provider logic (desktop files, wifi, clipboard) stays in scripts outside the crate.

## Scope
Goal surface, three crates in one workspace:

- `crates/tuipalette-core` — spec types + validation, layout resolution to `Rect`s (`ratatui-core`'s geometry type is the one ratatui type allowed in the public API — re-inventing it buys nothing), theme tokens, the `Component` trait, `Reaction`, focus ring + dispatch, key maps, the `Matcher` trait and its scorers, the stdin/stdout protocol model. No `Buffer`, no `Frame`, no widgets, no terminal I/O.
- `crates/tuipalette-tui` — ratatui/crossterm backend: event loop, `Buffer` rendering, `Viewport::Fullscreen` and `Viewport::Inline`, icon width handling, draw helpers. The only crate that knows a terminal exists.
- `crates/tuipalette` — the binary: argument parsing, spec loading, host selection, protocol, exit codes.
- `layouts/` — shipped RON documents, `rofi-like.ron` among them. Data, never compiled in; deleting the directory leaves a working library.
- `scripts/` — provider scripts and the host adapter that launches the picker in its floating window.
- `docs/` — this plan, one spec per item in `docs/specs/NN-<slug>.md`, plus the user-facing schema reference from 11.
- `spike/` — throwaway evidence from 01. It currently carries its own `.git` and `.gitignore` (from `cargo init`), and the project root is not a repository at all; 02 owns deleting that nested repo and initialising the root.

Seams that must stay clean: **spec → layout** (data in, `Rect`s out, no widgets), **component → app** (`Reaction`, no global mutation), **app → terminal** (ratatui and all width/icon mechanics behind `tuipalette-tui`), **process → caller** (lines on stdout, code on exit, nothing else on stdout ever), **library → shipped layout** (the crates compile and test green with `layouts/` absent). Placement is deliberately outside every seam — it belongs to the compositor, see D3.

Verified before writing this plan: root `/home/rebellion/Code/hobby/terminal-ui-lib` holds only `spike/` (default `cargo init` output, no source written); `cargo`/`rustc 1.95.0` present; `ratatui 0.30.2`, `ron 0.12.3` and `serde_json` resolve from crates.io; `ratatui::crossterm`, `Viewport::{Fullscreen,Inline,Fixed}`, `StatefulWidget`, `Layout::{vertical,split,areas}`, `Frame::render_widget`, `init/restore/run` present in the fetched source; crate names `tuipalette-core`, `tuipalette-tui`, `tuipalette` are free on crates.io (`palette` and `palette-core` are taken by the colour crate). Not checked: real cold-start latency (12).

## Work
### 01 — Feasibility spike: components, actions, layout from a spec · done 2026-11-16
- A ~350-line spike under `spike/` that loads a RON spec, renders three panes on resolved rects, dispatches `down`/`down`/`enter` through a `Component` trait, and prints the chosen value — checkable by `cd spike && cargo test` (headless `TestBackend` render asserts pane titles land inside their rects; the key sequence yields `run:herdr`; a malformed spec surfaces RON's line/column error, not a `None`).
- detail: `docs/specs/01-spike.md`

### 02 — Workspace skeleton and repo hygiene · done 2026-11-16
- `git init` at the root, the nested `spike/.git` removed, `.gitignore` covering `target/`, the three crates under their settled names with the dependency edges the scope fixes — checkable by `cargo build --workspace && cargo test --workspace` from a clean clone-level state.
- after: 01
- detail: `docs/specs/02-workspace.md`

### 03 — Spec schema in RON, layout resolution, theme tokens · done 2026-11-16
- `UiSpec` in `tuipalette-core`: window, theme tokens (hex colours), pane declarations, key maps, an icon slot width, a split tree with vertical/horizontal nodes and the `fill|N|N%|min:N|max:N` size grammar, deserialized from RON with every wire name pinned by `#[serde(rename)]` and `deny_unknown_fields`, plus a JSON adapter behind `--format json`; validation errors carrying line/column and the field path — checkable by `cargo test -p tuipalette-core -- spec` (a golden table of spec→`Vec<(id, Rect)>` cases across both formats, plus a malformed spec that must name its field) and `tuipalette --check --spec bad.ron` exiting non-zero with that message.
- after: 01
- detail: `docs/specs/03-spec-and-layout.md`

### 04 — Component trait, focus ring, action router, key maps · done 2026-11-16
- `Component { id, focusable, render, update -> Reaction, value }` plus the router: global actions first (`quit`, `focus-next/prev`), then the focused pane, then bubbling to the rest, and `Reaction::Emit(Action::Invoke(..))` reaching app-level command handling; key maps resolve per-pane over a global map, an explicit `none` clears an inherited bind, and release/repeat key events are ignored — checkable by `cargo test -p tuipalette-core --test dispatch` covering focus skip over non-focusable panes, bubbling, emit, override and unset.
- after: 02, 03
- detail: `docs/specs/04-component-dispatch.md`

### 05 — Built-in components and snapshot tests · done 2026-11-16
- `InputLine` (cursor, backspace), `Menu` (selection clamp, scroll window, item template with leading icon slot), `TextPane`, status line, spacer — rendered through `insta` snapshots against `TestBackend`, so a visual regression fails a test instead of needing eyes — checkable by `cargo test -p tuipalette-core --test components` → `14 passed`, with snapshots for all five kinds and the launcher end to end.
- after: 04
- detail: `docs/specs/05-components.md`

### 06 — Streaming source, matcher, match highlighting · done 2026-11-16
- A source abstraction that accepts lines on stdin as they arrive (no waiting for EOF) and a `Matcher` trait at `tuipalette-core/src/matcher/mod.rs` returning order + matched character spans, which the menu turns into coloured spans; each line carries up to three TAB-separated fields (display, value, icon — D12); the default scorer is `matcher/nucleo.rs`, adapting `nucleo-matcher` for fzf-compatible ranking because the current rofi config asks for `sorting-method: "fzf"`, with `matcher/simple.rs` as the dependency-free fallback reachable through `--no-default-features`; neither crate is re-exported — checkable by `cargo test -p tuipalette-core -- matcher` (span positions and ordering, the same trait for both scorers), by `cargo build -p tuipalette-core --no-default-features` proving the lean path, and a manual run against `find /usr` showing results before the command finishes.
- after: 04
- detail: `docs/specs/06-source-and-matcher.md`

### 07 — CLI contract: dmenu-compatible stdin/stdout, exit codes · done 2026-11-16
- `tuipalette` reads lines on stdin, writes only the chosen line to stdout, and exits `0` accepted / `130` cancelled / `2` unusable host; flags for `--spec`, `--format`, `--prompt`, `--theme`, `--inline`, `--icons`/`--no-icons`, `--print-query`, plus `--check` — checkable by `printf 'firefox\nherdr\n' | tuipalette --spec spec.ron | cat` printing exactly one line, and `esc` verified to leave stdout empty with code 130.
- after: 05, 06
- detail: `docs/specs/07-cli-contract.md`

### 08 — Host adapters: floating window and inline · done 2026-11-16
- Two adapters and nothing else, both driven by the same `host` section of the spec (width, height, position, with inline defaulting to the whole terminal). Floating: `scripts/tuipalette-floating.sh` launches the picker in a terminal that names its own window class, applies the spec's geometry, and relies on the documented Hyprland rule for float, size and position on a keybind. Inline: `--inline` renders in the terminal it was invoked from, taking the whole terminal when no height knob is set and the last N lines otherwise, leaving scrollback intact and restoring the cursor and scroll region on exit. Selection is automatic — stdout a tty with no floating adapter means inline — and `2` when neither host is usable — checkable by `hyprctl clients -j` showing the floating window's class, size and position matching the spec, and by `tuipalette --inline` inside kitty with scrollback and cursor verified after exit.
- after: 07
- detail: `docs/specs/08-hosts.md` (floating verified; the inline-in-kitty half is outstanding)

### 12 — Cold-start probe: is 100 ms reachable at all? · done 2026-11-16
- A throwaway measurement, before the picker does anything useful: exec of the binary to first flushed frame, with the phases reported separately (spec parse, terminal init, first draw) and terminal spawn excluded — spawn is the host's cost, not ours — checkable by a phase table in the item's spec and a yes/no on whether the 100 ms budget survives, all of it driven by one env-gated timing switch inside the binary so it is not a separate harness.
- after: 02
- detail: `docs/specs/12-cold-start.md`

### 09 — Latency verdict and the daemon decision · done 2026-11-16
- Two comparisons on the finished binary: ours (exec to first paint, then key to repaint) against the 100 ms budget, and the whole path (terminal spawn included) against `rofi -dmenu` on the same keybind — comparing our inner number to rofi's whole path would flatter us, so both tables exist — plus a written verdict on D2's one-process-per-invocation model, the resident daemon recorded as an explicit non-answer unless the numbers force it, and a missed budget defaulting to "accept and record" — checkable by both tables and the verdict in the item's spec, any exception to D2 recorded as a new decision.
- after: 08, 12
- detail: `docs/specs/09-latency.md`

### 10 — Migrate one real picker end to end · done 2026-11-16
- `~/.config/hypr/scripts/herdr-sessions.sh` (multi-step `rofi -dmenu` menus, typed input as the answer) running on `tuipalette` with no behaviour change — checkable by binding it: attach, back, empty-answer entry, cancel.
- after: 08
- detail: `docs/specs/10-migration.md`

### 13 — Ship a rofi-like layout · done 2026-11-16
- `layouts/rofi-like.ron` reproducing the look and behaviour of the user's current rofi setup (`~/.config/rofi/config.rasi`: `modi run,drun`, fzf sorting; `themes/monokai.rasi` for the colours), plus the provider scripts it needs — app listing and `.desktop` parsing among them, since provider logic is not the library's job — and the two entry points (app launch, dmenu), with no rofi concept entering the crates — checkable by launching it side by side with `rofi` on the same keybind and swapping them with a one-line change in the adapter.
- after: 07
- detail: `docs/specs/13-rofi-like-layout.md`

### 14 — Rofi parity inventory and empirical check · done 2026-11-16
- Every rofi call site **a user can still reach** enumerated — `~/.config/hypr/scripts` (which, after item 10, has none left), the four binds that go through `MENU = "rofi -show"` in `hyprland.lua` (`run`, `drun`, `window`, `calc`), and `~/.local/bin/rofi-wifi-menu.sh` with `-dmenu`/`-password`/`-auto-select`/`-location`/`-opacity`/`-font`/`-e` — each recorded as migrated, benched or impossible, with the flags it used.
- The original wording scoped this to `~/.config/hypr/scripts`, where the only call site was the herdr picker and item 10 had already migrated it: the checkable would have passed while claiming nothing. Widened to the sites that actually exist.
- checkable by the inventory table in the item's spec, with a command and its output for every migrated row, and a one-line reason for every benched or impossible one.
- after: 10, 13
- detail: `docs/specs/14-parity.md`

### 15 — Icon slots and their fallback · done 2026-11-16
- An icon is one slot of declared width in the item template, filled by the glyph the provider sends — lookup belongs to the provider (D12), so 15 is a renderer and nothing else; width is computed per grapheme cluster (`unicode-width`), and anything that does not fit or cannot be assumed renderable is replaced by a placeholder from the theme, so a missing Nerd Font degrades the picture rather than the layout — checkable by a snapshot matrix of native-width glyph, double-width emoji, absent glyph, and `--no-icons`, all four keeping the text column aligned.
- after: 05
- detail: `docs/specs/15-icons.md`

### 11 — Spec reference and component guide · done 2026-11-16
- `docs/spec-reference.md` listing every RON field, its wire name, meaning and default, the key-map merge rules, the icon slot contract, and a guide for adding a `Component`, both complete enough that a new picker is written from the doc alone — checkable by writing one new spec (a wifi or clipboard picker) using only the reference, with no source reading.
- after: 05
- detail: `docs/specs/11-docs.md`

### 16 — Conditional actions and document transitions
- Today an action has one fixed effect: `Accept` takes the selection, `Quit` exits, and a document is one layout. Rofi's behaviour on an empty answer is not that — with no match it hands the typed text back, the caller fails to run it, and escape returns to the same menu — and neither is a multi-step flow like `herdr-sessions.sh`, where each menu leads to the next. This item makes the *outcome of an action* data: a rule list per action, each rule a condition and an effect, where effects can stay, accept, cancel, go back, show a message in the status row, or open another document.
- The schema, additive and defaulting to today's behaviour: `rules: { Accept: [ (when: NoMatches, then: Message("no such entry")), (when: Always, then: Accept) ], Quit: [ (when: Stacked, then: Back), (when: Always, then: Cancel) ] }` — conditions `Always`/`HasSelection`/`NoMatches`/`QueryEmpty`, effects `Accept`/`Cancel`/`Back`/`Keep`/`Message(text)`/`Open(path)`. A message renders in the status row until the next key; `Open` pushes a document and `Back` pops it, which is the stack rofi gives for free when a script simply operates in a loop.
- Feasibility: high, and additive. The router already turns an `Action` into an `Outcome` (`Quit`/`Continue`/`Invoke`); this generalises that one match into a rule list evaluated against a small context, and turns `Outcome` into an effect the run loop applies. Transitions need a document stack in the binary and a loader that resolves `Open` relative to the current file — a document re-parses in 0.1 ms, so a transition costs nothing worth measuring. The two real decisions are whether `Back` restores the previous query or starts clean, and whether rules are keyed by action (proposed) or by key spelling.
- checkable by a layout with two documents where `enter` on no match shows a message and stays, `esc` goes back to the first document, and `esc` there exits — driven through `scripts/pty-check.py`, plus a test that a document without `rules` behaves exactly as it does now.
- after: 07
- detail: `docs/specs/16-rules.md`
- Sequenced after the base product: items 10 and 14 first.
### 17 — Revisit the herdr setup and the wifi flow
- Two call sites are migrated but not yet *designed*: `layouts/dmenu.ron` with `scripts/herdr-sessions.sh`, and `scripts/wifi.sh` with `layouts/wifi.ron`.
- herdr: the session rows carry their state as padded trailing text rather than an icon slot; `+ new session` and `- delete a session` are ordinary rows with a prefix; the message line restates what the rows say; and a herdr started from a shell could use `--inline` instead of a floating window. Decide each, and what the window size should be.
- wifi: `rofi-wifi-menu.sh` is a 359-line flow whose parts were only partly mapped (item 14, rows 5–8). Decide what `scripts/wifi.sh` keeps, what needs library work (a masked `Input`, a message-only document), and whether the two stages become one document with rules (item 16).
- checkable: `docs/specs/17-revisit.md` naming every behaviour of both call sites, what it becomes and the command that shows it, with `tuipalette --check` clean for both documents.
- after: 16 — conditional actions are where `-e`, `-auto-select` and the `+`/`-` rows land
- detail: `docs/specs/17-revisit.md`

### 18 — Ranking: substring before subsequence, and frecency as a tie-break
- Observed while using the migrated binds: typing `kni` in the app picker lists `KDE Connect Indicator` above `Arknights`, which contains the query as a literal substring. fzf ranks them the same way, so this is the algorithm rather than a defect in our use of nucleo — but it is not what a launcher should do.
- Proposal: (a) a literal substring hit outranks any scattered subsequence hit; (b) ties break by shorter display, then arrival order; (c) usage from `scripts/usage.sh` becomes a documented tie-break, so it also orders *filtered* results and not just the empty query.
- checkable: a before/after table for the queries `kni`, `fire`, `code` (top five rows each), a test pinning each row's order, and `scripts/usage.sh` still working unchanged.
- after: 17 — the revisit decides what stays in the providers
- detail: `docs/specs/18-ranking.md`

### 19 — Clipboard picker: a preview that follows the selection · done 2026-11-16
- Migrating `~/.config/hypr/scripts/cliphist-fzf.fish` (cliphist + fzf with a right-hand preview) exposed three things the schema could not express: a pane whose content is the *selected row's* payload, a pane without its own box, and a border around the whole window. All three are now declared data.
- `Item.detail` is a fifth field (JSON: `detail`) that no list row draws; `kind: Detail` shows it for the row the menu has selected, following `from: "<menu id>"`; the picker copies it on every selection or row change, so a preview is never a keystroke behind. `frame: false` removes a pane's box, `window.frame: true` draws one around everything.
- The layout is measured against the original, not eyeballed: fzf's border box in `hyprshot-window` capture `960x540` has its preview at rows 1–25 and columns 43–103 of a 105-column interior, i.e. the preview runs the *full* height beside the prompt and header, and one blank column is left before the right border. `layouts/clipboard.ron` reproduces that (the blank column is a one-cell `Spacer`), which is why the preview is not inside the left column's vertical split.
- checkable: `docs/specs/19-clipboard.md`; `cargo test --workspace` → `components__clipboard_layout` snapshots both selection states; `scripts/clipboard.sh` opens, filters and pastes (a real pick wrote `Done Deploy` back through `wl-copy`).
- after: 14 — it is the fourth migrated call site
- detail: `docs/specs/19-clipboard.md`

### 20 — Images: the kitty graphics protocol, as a host feature
- The cliphist picker's preview renders images with `kitty +kitten icat`. This library draws text into a cell buffer and promises to be terminal-agnostic (D1, D11), so an image is not a cell and cannot become a `Component` concept without breaking that promise.
- The decision to make: either a **host-level** escape passthrough — a pane declares a graphics area, the provider supplies a path and the *host* (kitty only) emits the sequence, with every other terminal falling back to the text preview — or keep images out of the library entirely and let the provider open a separate viewer.
- What is known: kitty's protocol needs a placement rectangle in cells, a delete/refresh discipline when rows move or the selection changes, and a teardown pass so no half-drawn image is left in the alternate screen; fzf solves this outside its own rendering, which is a hint about where it belongs.
- checkable: `docs/specs/20-images.md` choosing one of the two, and either a capture of an image preview in the clipboard picker or a recorded reason it stays out.
- after: 19 — the detail pane is the slot an image would fill
- detail: `docs/specs/20-images.md`

### 21 — Mouse: hover, click and wheel
- Every crossterm mouse event currently falls through the key path and is dropped (`tuipalette-tui/src/input.rs` translates keys only). A picker that a user drives with a mouse needs three things: hit-testing a point against the rects the layout produced, a `&Action` a component can answer for hover/click/scroll, and mouse capture enabled and disabled around the run (it takes text selection away from the terminal, so it must be released on every exit path, including a panic).
- Scope: moving the pointer highlights the row under it (a hover state the menu keeps), a click selects that row, a double click (or click on the selected row) accepts, the wheel scrolls the list, and clicking the query pane focuses it. Nothing changes for a document that does not want this: capture stays off unless the document asks (`window: (mouse: true)`), because a picker that steals the mouse in a pipeline is a bug.
- checkable: `docs/specs/21-mouse.md`; `scripts/pty-check.py` driving SGR mouse sequences (`ESC[<0;x;yM`/`m`) into a real run of `layouts/rofi-like.ron`: hover moves the highlight, click accepts the row under the pointer, wheel scrolls a 200-row list.
- after: 19 — the rects and the pane dispatch it needs are what 19 touched last
- detail: `docs/specs/21-mouse.md`
## Decisions
Full entries and rationale live in `docs/DECISIONS.md`, append-only; this is the digest of what still binds outstanding work.
- D1 — Render stack: ratatui 0.30 + crossterm, confined to `tuipalette-tui`; components render into `&mut Buffer` and never touch `Frame`, so they stay testable headless and backend-agnostic (binds all)
- D2 — No daemon in v1: one process per invocation, stdin/stdout contract, latency debt recorded rather than pre-paid (binds 07, 09, 10)
- D3 — Placement is the host's: the core never positions, sizes or styles a window; the adapter (compositor rule for floating, the terminal itself for inline) owns geometry (binds 08, 09)
- D4 — Declarative spec, not a scripting language: a data document only — no conditionals, no expressions, no templating, no embedded runtime. Anything dynamic is a provider writing lines to stdin (binds 03, 11)
- D5 — Extension is a trait, not a plugin ABI: built-in components only in v1; adding one means a compile (binds 05, 11)
- D6 — Spec format: RON as the hand-authored format, JSON as the machine format, both behind one `spec::Format` adapter; TOML and KDL both dropped (binds 01, 03, 11)
- D7 — Shipped layouts are data, never library code: `layouts/*.ron` are user-space artifacts the crates do not reference; rofi parity must be a customer of the schema, not a property of it (binds 03, 10, 13, 14)
- D8 — Key-map merge: per-pane entries win over the global map, global entries apply wherever a pane does not override them, and an explicit `none` clears an inherited bind (binds 04, 11)
- D9 — Names: project `terminal-ui-palette`; crates `tuipalette-core`, `tuipalette-tui`, binary `tuipalette` (binds 02, 11)
- D10 — Two hosts, no compositor assumption: floating (compositor-placed terminal window) and inline (`Viewport::Inline` in the invoking terminal) are both first-class; muxer popups stay benched (binds 07, 08, 09)
- D11 — Icons are a slot with a fallback, not a graphics feature: one declared-width slot per item, provider-supplied text, grapheme-width checked, placeholder on any failure; image protocols excluded (binds 05, 11, 15)
- D12 — Line protocol: `display<TAB>value<TAB>icon` per line, trailing fields optional, no TAB means the line is both display and value (binds 06, 07, 15)
- D13 — Matcher default: `nucleo-matcher` is the default scorer, `simple.rs` is the `--no-default-features` path (binds 06)
- D14 — Measurement basis: the 100 ms budget is exec → first flushed frame, spawn excluded and reported apart; rofi is compared whole-path (binds 09, 12)
- D15 — Field sanitation: display sanitized, value byte-exact, last declared field takes the remainder (extends D12; binds 06, 07, 15)
- D16 — Pipe escaping: TAB fields by default, `--in json`/`--out json` together for byte-exact values (binds 06, 07)

## Risks
- Cold start lands far worse than rofi and the picker feels sluggish on a keybind — trigger: 12 or 09 measuring above the 100 ms budget, or a visible delay against everything else on the compositor.
- RON floods the schema surface into the API: struct and enum names in the file are the Rust type names, so renaming a type silently breaks every shipped layout — trigger: any `layouts/*.ron` becoming invalid because a Rust type was renamed, which is why 03 pins every wire name.
- RON is verbose on deep trees and unfamiliar to non-Rust users; a spec that needs six nested splits may stop being hand-editable — trigger: `layouts/rofi-like.ron` growing past what can be skimmed in one screen.
- Icon width drift: emoji and Nerd Font glyphs are double-width or ambiguous-width depending on terminal and font, so a slot computed one way and rendered another shifts the whole text column — trigger: 15's snapshot matrix disagreeing with a real kitty/ghostty/alacritty run.
- Inline host fights the application that owns the terminal: scroll regions, cursor restoration and alternate-screen handling differ per terminal, and a crash leaves the terminal in a broken mode — trigger: scrollback damage or a lost cursor after an `--inline` run, or an uncaught panic path.
- Cell geometry vs pixel intent: the floating window rule sizes in pixels, the UI reasons in cells, so the card grows and shrinks with the terminal font and monitor scale — trigger: the picker's box changing proportions between monitors, terminals or font sizes.
- Spec language creep — trigger: the first conditional, template or expression appears in a RON spec, and users ask for "just one if".
- Component trait erodes into a framework: retained state plus dynamic lists pushes toward `Rc<RefCell<..>>` everywhere — trigger: the router needs interior mutability to dispatch a single action.
- ratatui 0.30 is a fresh multi-crate split (`ratatui-core` 0.1.2, `ratatui-widgets`, `ratatui-crossterm`), still unstable at the sub-crate versioning level — trigger: a dependency bump breaks `tuipalette-tui` compile.
- Terminal-specific feature requests arrive disguised as "customisability" (images, transparency, blur) — trigger: `tuipalette-tui` gaining an "if kitty" branch outside the icon backend boundary.
- Two overlapping surfaces of truth for the user's keybinds (rofi config and picker spec) while migration is partial — trigger: a rebind working in one picker and not the other.

## Open questions
- If the whole path (spawn included) misses the feel target while our exec-to-paint is well inside budget: is a lighter launch in scope (single-instance terminal, pre-warmed instance, daemon), or is the host simply the host — your call once 09 has both numbers.

## Acceptance criteria
- [ ] `cargo build --workspace && cargo test --workspace` passes from a fresh clone, with the spike's findings folded in or the spike deleted.
- [ ] One RON document fully describes a picker — panes, split sizes, theme tokens, icon slots, per-pane keys — with no code change to alter layout.
- [ ] `printf 'a\nb\n' | tuipalette --spec <file>` prints exactly one line and nothing else on stdout; cancelling prints nothing and exits 130.
- [ ] Both hosts work: the floating window on a keybind (verified with `hyprctl clients -j`) and `--inline` from a terminal with scrollback and cursor intact afterwards.
- [ ] Exec-to-first-paint for the floating-window path is measured at or under 100 ms on this machine, with terminal spawn reported separately and the whole-path figure sitting beside rofi's own.
- [ ] Icons degrade without moving the text column: the four-way snapshot matrix (native, double-width, absent glyph, `--no-icons`) passes on this machine's terminal.
- [ ] `layouts/rofi-like.ron` reproduces the current setup's look and behaviour for app launch and for the dmenu menus, verified side by side with rofi.
- [ ] Every `rofi` call site in `~/.config/hypr/scripts` is inventoried in `docs/specs/14-parity.md` as migrated, benched or impossible, and no `rofi` invocation remains on a Hyprland keybind.
- [ ] Components are covered by `TestBackend`/`insta` snapshots, so a rendering regression fails a test.
