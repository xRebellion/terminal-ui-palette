# 11 — Spec reference and component guide

Detail for item 11 of `docs/PLAN.md`. The deliverable is `docs/spec-reference.md`, and the test of it is the item's checkable: **write a picker from the page alone.**

## What the page covers

| section | what it pins down |
|---|---|
| top level | `window`, `theme`, `layout`, `panes`, `keys`, `host` — every field, its default, and that unknown fields are errors |
| theme | the five tokens, their defaults, `#rrggbb`-only, and that `--theme` takes the same object |
| layout | leaf vs split, the size grammar, how the solver distributes cells, and that focus order is draw order |
| panes | the five kinds, what each draws, `icon_width`, per-pane `keys` |
| keys | the spelling grammar, the five document-visible actions, and the three-layer merge with `None` clearing an inherited binding |
| items | the `display\tvalue\ticon` line protocol, the JSON variant, and that items stream in before EOF |
| output | value on stdout, `--out json`, `--out-file`, the four exit codes |
| command line | every flag, and the two environment switches |
| writing a new picker | the five-step recipe, with the wifi picker as the worked example |
| adding a `Component` | the whole trait, the reaction contract, how to register, and how to test it |

## The checkable

Two files were written from the page without reading the source: `layouts/wifi.ron` and `scripts/wifi.sh`. Then:

```
$ tuipalette --check --spec layouts/wifi.ron
layouts/wifi.ron: ok — 3 panes, 2 bindings
$ tuipalette --spec layouts/wifi.ron --print-geometry
50x12
$ tuipalette --spec layouts/wifi.ron --print-class
tuipalette
$ scripts/wifi.sh --fake | tuipalette --spec layouts/wifi.ron   # typed cafe, enter
cafe
$ scripts/wifi.sh --fake | tuipalette --spec layouts/wifi.ron   # esc
exit=130, nothing on stdout
```

The provider prints the protocol (tabs shown as `^I` by `cat -A`):

```
  home-net   84%  wpa^Ihome-net
  cafe       61%  open^Icafe
  office     38%  wpa^Ioffice
```

## Errors the page prevents

Each of these is a document mistake with a specific result, all of them described on the page: a misspelled field (rejected — `deny_unknown_fields` at every level), a bad size string (rejected with `layout.sizes[i]`), a colour that is not `#rrggbb` (rejected), a key spelling the parser does not know (rejected with the expected modifiers), a pane id the layout never names (rejected), and the same pane drawn twice (rejected).

## Not in this item

The page documents the library as it is; it does not describe the matcher's scoring, the scroll-window derivation, or the snapshot workflow beyond one paragraph. Those live in the per-item specs, and a picker author never needs them.
