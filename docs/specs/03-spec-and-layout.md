# 03 — Spec schema in RON, layout resolution, theme tokens

Detail for item 03 of `docs/PLAN.md`. This is the contract item 11 documents.

## Wire shape

```ron
(
    window: (title: "launcher"),
    theme: (accent: "#89b4fa", selected_bg: "#313244", text: "#cdd6f4"),
    layout: (
        dir: Vertical,
        sizes: ["3", "fill", "1"],
        children: [
            (pane: "query"),
            (dir: Vertical, sizes: ["50%", "50%"], children: [(pane: "results"), (pane: "preview")]),
            (pane: "status"),
        ],
    ),
    panes: [
        (id: "query", kind: Input, title: "search"),
        (id: "results", kind: Menu, title: "results", items: ["firefox", "kitty"]),
        (id: "preview", kind: Text, items: ["firefox — 130 MB"]),
        (id: "status", kind: Text, items: ["enter: run"]),
    ],
    keys: { "esc": Some(Quit), "tab": Some(FocusNext), "ctrl-j": None },
)
```

JSON is the same document with the same field names: `{"dir": "Vertical", ...}`, with `null` for an unset binding. One set of derives serves both, because serde's default enum representation is external tagging, which RON's `Name(...)` and JSON's `{"Name": ...}` both express.

## Decisions taken inside this item

- **A layout node is one struct, not an enum of variants.** `(dir: Vertical, children: [...])` rather than `Vertical(...)`. The enum reads better, but serde cannot deny unknown fields inside an enum variant, so a typo'd `childrens:` would parse as an empty split and render nothing. The struct shape buys that error class back for one extra word — test `a_typo_inside_a_node_is_named_too`.
- **RON needs `Extensions::IMPLICIT_SOME`.** Without it every `Option` field has to be written `Some(...)`, and `dir: Vertical` fails with `Expected option`. With it, `dir: Vertical` reads naturally *and* `None` stays explicit, so D8's unset keeps a spelling.
- **Two error shapes, one type.** serde knows a line and a column; validation knows a field path. `SpecError` carries either and `Display` prints whichever it has, so a caller never matches on provenance.
- **Undrawn panes are allowed silently.** A spec may declare a pane it does not place yet; only the reverse — a reference to an undeclared pane — is an error.
- **`--check` exists now, not in 07,` because item 03's own checkable runs the binary. Exit codes: 0 spec fine, 1 spec wrong, 2 invocation wrong. Item 07 layers the picker's 0/130 pair on top.

## The size grammar

| written | means |
|---|---|
| `3` | exactly 3 cells |
| `50%` | 50 percent of the area; above 100 is refused |
| `fill` | share the remainder equally with other fills |
| `min:5` | at least 5, grows if there is room |
| `max:20` | at most 20 |

Anything else is refused with its index named, e.g. `layout.sizes[0]: `wide` is not a size — expected `N` cells, `N%`, `fill`, `min:N` or `max:N``. An empty `sizes` means all fill.

## Validation rules (each covered by a test)

theme colours are `#rrggbb` or the spec is refused · pane ids non-empty and unique · `icon_width` at least 1 · a leaf takes only `pane` · a split needs `dir` · `sizes` length matches `children` · every size parses · every referenced pane is declared · no pane is drawn twice.

## Evidence

```
cargo test --workspace
  tuipalette-core  tests/spec.rs    16 passed
  tuipalette       tests/check.rs    5 passed
  tuipalette       unit              2 passed

./target/debug/tuipalette --check --spec .../launcher.ron
  …launcher.ron: ok — 4 panes, 3 bindings            exit 0
./target/debug/tuipalette --check --spec .../launcher.json
  …launcher.json: ok — 4 panes, 3 bindings           exit 0
--check /tmp/bad.ron     exit 1   layout.sizes[0]: `wide` is not a size — expected `N` cells, `N%`, …
--check /tmp/typo.ron    exit 1   line 1, column 63: Unexpected field named `tittle` in `Pane`, expected one of …
```

Golden geometry, asserted in both formats from the same document at 40×10: `query (0,0,40,3)` · `results (0,3,40,3)` · `preview (0,6,40,3)` · `status (0,9,40,1)`.

## Not in this item

Overflow behaviour when a spec asks for more cells than it has (ratatui clamps; nothing warns yet), the remaining CLI flags (07), and the reference doc (11) — the tests above are the source of truth until then.
