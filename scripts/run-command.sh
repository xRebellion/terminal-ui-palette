#!/usr/bin/env bash
# The command runner behind "rofi -show run": type a command, run it.
#
#   scripts/run-command.sh              # pick a command and run it detached
#   scripts/run-command.sh --list       # just print the candidates
#
# Rofi completed PATH entries and ran whatever was in the box. Ours offers the
# same candidates and, because --accept-query is on, also runs whatever is
# typed when nothing matches -- which is exactly rofi's run mode.
set -uo pipefail
here=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
adapter=${TUIPALETTE_ADAPTER:-$here/tuipalette-floating.sh}
spec=${RUN_SPEC:-$here/../layouts/dmenu.ron}

commands() {
  # Walk PATH for executable files: compgen also lists builtins and keywords
  # ('!', '[', '[[', ':') that cannot be run from a detached shell.
  printf '%s\n' "$PATH" | tr ':' '\n' | while read -r dir; do
    [ -d "$dir" ] || continue
    for file in "$dir"/*; do
      [ -f "$file" ] && [ -x "$file" ] && printf '%s\n' "${file##*/}"
    done
  done | sort -u
}

if [ "${1:-}" = "--list" ]; then
  commands
  exit 0
fi

choice=$(commands | "$adapter" "$spec" --prompt 'run' --message 'enter runs, esc cancels' --accept-query cat - ) || exit 0
[ -n "$choice" ] || exit 0
if [ "${RUN_PRINT_ONLY:-}" = 1 ]; then
  printf '%s\n' "$choice"
  exit 0
fi
exec sh -c "$choice"
