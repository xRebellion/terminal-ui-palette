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
  # .hidden is absent on some clients, so ask for "not true" rather than
  # "== false". The class is the fallback title: a window with no title
  # (a splash, a scratchpad) still has to be choosable.
  hyprctl clients -j 2>/dev/null | jq -r '
    .[]
    | select(.mapped and (.hidden != true))
    | [ .title, .address, .class ]
    | @tsv
  ' | awk -F'\t' '
    { title = ($1 == "" ? $3 : $1); printf "%s\t%s\n", title, $2 }
  '
}

if [ "${1:-}" = "--list" ]; then
  rows
  exit 0
fi

choice=$(rows | "$adapter" "$spec" --prompt 'window' --no-sort cat - ) || exit 0
[ -n "$choice" ] || exit 0
address=${choice##*$'\t'}
if [ "${WINDOW_PRINT_ONLY:-}" = 1 ]; then
  printf '%s\n' "$address"
  exit 0
fi
# The dispatcher takes a window object or a *string* selector — a table of
# fields like { address = … } is rejected with "expected a window object or
# selector", and hyprctl still exits 0, so the answer is checked, not the
# status. This is why the first version "worked" and did nothing.
answer=$(hyprctl dispatch "hl.dsp.focus({ window = 'address:$address' })" 2>&1)
case $answer in
  ok*) ;;
  *) echo "switch-window: focus failed: $answer" >&2; exit 1 ;;
esac
