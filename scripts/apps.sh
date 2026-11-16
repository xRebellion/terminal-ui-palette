#!/usr/bin/env bash
# List applications as "name<TAB>command" — the drun half of the old rofi setup.
#
# Provider logic belongs to the provider, never to the library (docs/PLAN.md,
# D12): the picker only ever sees lines.
#
# One awk pass over every .desktop file. The first version forked four
# subprocesses per file and took 1.3 s for 106 rows, which the picker showed as
# a window that sat empty for over a second (item 14 follow-up).
set -euo pipefail

files=()
for dir in /usr/share/applications "$HOME/.local/share/applications"; do
  [ -d "$dir" ] || continue
  for file in "$dir"/*.desktop; do
    [ -e "$file" ] && files+=("$file")
  done
done
[ "${#files[@]}" -gt 0 ] || exit 0

awk '
  function emit() {
    if (name != "" && exec_line != "" && !nodisplay && !hidden) {
      if (terminal) exec_line = "kitty sh -c \x27" exec_line "\x27"
      gsub(/ ?%[fFuUdDnNickvm]/, "", exec_line)
      printf "%s\t%s\n", name, exec_line
    }
  }
  FNR == 1 { if (started) emit(); name = ""; exec_line = ""; nodisplay = 0; hidden = 0; terminal = 0; started = 1 }
  /^Name=/ && name == "" { name = substr($0, 6); next }
  /^Exec=/ && exec_line == "" { exec_line = substr($0, 6); next }
  /^NoDisplay=true/ { nodisplay = 1; next }
  /^Hidden=true/ { hidden = 1; next }
  /^Terminal=true/ { terminal = 1; next }
  END { if (started) emit() }
' "${files[@]}" | sort -u
