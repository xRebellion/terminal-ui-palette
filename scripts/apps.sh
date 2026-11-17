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

# The fourth field is the matcher's and never appears on screen. rofi's drun
# matches Name, GenericName, Exec, Categories and Keywords while showing
# the Name, which is why typing "seahorse" there finds "Passwords and
# Keys" (source/modes/drun.c, matching_entry_fields).
awk '
  function emit() {
    if (name != "" && exec_line != "" && !nodisplay && !hidden) {
      if (terminal) exec_line = "kitty sh -c \x27" exec_line "\x27"
      gsub(/ ?%[fFuUdDnNickvm]/, "", exec_line)
      search = generic " " categories " " keywords " " exec_line
      gsub(/;/, " ", search)
      gsub(/[[:space:]]+/, " ", search)
      printf "%s\t%s\t\t%s\n", name, exec_line, search
    }
  }
  FNR == 1 {
    if (started) emit()
    name = ""; exec_line = ""; generic = ""; categories = ""; keywords = ""
    nodisplay = 0; hidden = 0; terminal = 0; started = 1
  }
  /^Name=/ && name == "" { name = substr($0, 6); next }
  /^Exec=/ && exec_line == "" { exec_line = substr($0, 6); next }
  /^GenericName=/ && generic == "" { generic = substr($0, 13); next }
  /^Categories=/ { categories = substr($0, 12); next }
  /^Keywords=/ { keywords = substr($0, 10); next }
  /^NoDisplay=true/ { nodisplay = 1; next }
  /^Hidden=true/ { hidden = 1; next }
  /^Terminal=true/ { terminal = 1; next }
  END { if (started) emit() }
' "${files[@]}" | sort -u
