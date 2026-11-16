#!/usr/bin/env bash
# List applications as "name<TAB>command" — the drun half of the old rofi setup.
#
# Provider logic belongs to the provider, never to the library (docs/PLAN.md,
# D12): the picker only ever sees lines.
set -euo pipefail

for dir in /usr/share/applications "$HOME/.local/share/applications"; do
  [ -d "$dir" ] || continue
  for file in "$dir"/*.desktop; do
    [ -e "$file" ] || continue
    name=$(sed -n 's/^Name=//p' "$file" | head -1)
    exec_line=$(sed -n 's/^Exec=//p' "$file" | head -1)
    [ -n "$name" ] && [ -n "$exec_line" ] || continue
    grep -q '^NoDisplay=true' "$file" && continue
    grep -q '^Hidden=true' "$file" && continue
    term=false
    grep -q '^Terminal=true' "$file" && term=true
    if $term; then
      exec_line="kitty sh -c '$exec_line'"
    fi
    # Field codes (%f, %U, ...) are the app's business, not ours.
    exec_line=$(printf '%s' "$exec_line" | sed -E 's/ ?%[fFuUdDnNickvm]//g')
    printf '%s\t%s\n' "$name" "$exec_line"
  done
done | sort -u
