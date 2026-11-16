# 07 — CLI contract: dmenu-compatible stdin/stdout, exit codes

Detail for item 07 of `docs/PLAN.md`.

## The contract

```
printf 'firefox\nherdr\n' | tuipalette --spec dmenu.ron    # → herdr on stdout, exit 0
                                                          # esc → nothing, exit 130
```

Items arrive on stdin, one per line (or one JSON object per line with `--in json`), keys come from the terminal, and exactly one value leaves on stdout. Exit codes: **0** accepted, **130** cancelled, **1** the document is wrong, **2** unusable host or flags.

## Why the fd handling exists

The three streams a picker needs are not the three fds a shell gives it. When stdout is a pipe, drawing on stdout would corrupt the value; when stdin is a pipe of items, crossterm cannot read keys from it. `host::Host::prepare` therefore:

1. duplicates the real stdin (the items) and the real stdout (where the value goes),
2. points **both** fd 0 and fd 1 at `/dev/tty`,
3. leaves the duplicates for the source thread and for the final write.

When stdin and stdout are already the terminal, nothing is moved. When `/dev/tty` cannot be opened, the answer is exit 2 — that is the "unusable host" case.

## Flags

| flag | what it does |
|---|---|
| `--spec <file>` | the document (required) |
| `--format ron` or `json` | overrides the extension |
| `--check` | validate and print the shape, exit 0/1 (item 03) |
| `--inline` | draw in the invoking terminal instead of the alternate screen |
| `--height <n>` | inline height in lines, else the document's `host.height` |
| `--prompt <text>` | replaces the query pane's title |
| `--theme <file>` | theme tokens from a RON file |
| `--icons` / `--no-icons` | force or drop the icon column |
| `--in line` or `json` | how items arrive (JSON is D16's escape for values with newlines) |
| `--out line` or `json` | how the value leaves |
| `--print-query` | print the query first, then the value |
| `--print-geometry` | `<width>x<height>` from the document, for the floating adapter |
| `--print-class` | the window class the floating adapter asks its terminal for |

Flags take `--flag value` and `--flag=value`. Everything is hand-parsed: a picker on a keybind pays for every argument library in start-up time.

## Evidence (pty harness, `scripts/pty-check.py`)

Items on a pipe, keys on a real terminal, value on a pipe — the shape a dmenu call has.

```
down, enter   firefox/herdr/zathura  → exit 0    value 'herdr\n'
esc           firefox/herdr          → exit 130  value empty
'her', enter  firefox/herdr/zathura  → exit 0    value 'herdr\n'   (typing filters)
--inline --height 8, 'e', enter      → exit 0    value 'beta\n', no 1049h in the output
TUIPALETTE_TIMING=1, enter           → phases: spec-parse 0.6ms, terminal-init 1.3ms, first-frame 1.7ms
```

The last line is item 12's raw material: exec to first flushed frame is under 2 ms for the inner path, with terminal spawn excluded (D14).

## Not in this item

`--out json` prints `{"query":..,"value":..}`; a caller that wants the value only, byte-exact, is a follow-up. No `--verify` mode, no config file discovery: the spec path is the config.
