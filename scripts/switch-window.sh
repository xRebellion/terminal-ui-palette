#!/usr/bin/env bash
# The window switcher behind "rofi -show window": every mapped toplevel as one
# row, the address as the value so the choice survives a title with spaces.
#
#   scripts/switch-window.sh            # pick a window and focus it
#   scripts/switch-window.sh --list     # just print the rows
#
# A picker that switches windows needs the compositor to enumerate them, which
# is a provider's job (D12), not the library's.
set -uo pipefail
here=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
adapter=${TUIPALETTE_ADAPTER:-$here/tuipalette-floating.sh}
spec=${WINDOW_SPEC:-$here/../layouts/dmenu.ron}

rows() {
  hyprctl clients -j 2>/dev/null | jq -r '
    .[]
    | select(.mapped and .hidden == false)
    | [ .title, .address ]
    | @tsv
  ' | awk -F'\t' '
    { title = ($1 == "" ? $3 : $1); printf "%s\t%s\n", title, $2 }
  '
}

if [ "${1:-}" = "--list" ]; then
  rows
  exit 0
fi

choice=$(rows | "$adapter" "$spec" --prompt 'window' --no-sort --message 'enter focuses, esc cancels' cat - ) || exit 0
[ -n "$choice" ] || exit 0
address=${choice##*$'\t'}
if [ "${WINDOW_PRINT_ONLY:-}" = 1 ]; then
  printf '%s\n' "$address"
  exit 0
fi
hyprctl dispatch "hl.dsp.focus({ window = { address = '$address' } })" >/dev/null
