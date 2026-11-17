#!/usr/bin/env bash
# The clipboard history picker: cliphist on the left, the selected entry on
# the right, and a paste into whatever had focus before it opened.
#
#   scripts/clipboard.sh           # pick an entry, copy it, paste it
#   scripts/clipboard.sh --list    # print the rows, no window
#   scripts/clipboard.sh --paste   # used internally, right after a pick
#
# The pick is one shell: the picker hands back an id, cliphist decodes it,
# wl-copy takes it. The paste is a second shell started by the compositor *after*
# the picker window is gone, because only then has focus returned to the window
# the user wants the paste in — pasting from this shell would paste into the
# picker. That split is why `--paste` is a mode here rather than four lines
# inline (the same trick as the fzf picker this replaces).
set -uo pipefail
here=$(cd "$(dirname "$(readlink -f "${BASH_SOURCE[0]}")")" && pwd)
adapter=${TUIPALETTE_ADAPTER:-$here/tuipalette-floating.sh}
spec=${CLIPBOARD_SPEC:-$here/../layouts/clipboard.ron}
script=$(readlink -f "${BASH_SOURCE[0]}")
# How much of an entry the preview shows. A 200 KB log would otherwise be read
# and thrown away on every keystroke, and the pane is 20 rows tall.
preview_bytes=${CLIPBOARD_PREVIEW_BYTES:-4000}

# --- paste mode -------------------------------------------------------------
# Started by the compositor, so it is not a child of the picker's terminal.
if [ "${1:-}" = "--paste" ]; then
  # Wait for the picker to lose focus. 2 s is generous: the window is already
  # gone by the time the compositor runs this.
  for _ in $(seq 1 100); do
    cls=$(hyprctl activewindow -j 2>/dev/null | jq -r '.class // ""')
    [ "$cls" != "tuipalette" ] && break
    sleep 0.02
  done
  cls=$(hyprctl activewindow -j 2>/dev/null | jq -r '.class // ""')
  [ -n "$cls" ] || exit 0
  # Terminals paste with Ctrl+Shift+V, everything else with Ctrl+V.
  terminals="kitty alacritty foot footclient wezterm org.wezfurlong.wezterm ghostty com.mitchellh.ghostty konsole gnome-terminal gnome-terminal-server xfce4-terminal tilix terminator urxvt xterm st yakuake ptyxis blackbox"
  if printf ' %s ' "$terminals" | grep -q " $cls "; then
    ydotool key -d 5 29:1 42:1 47:1 47:0 42:0 29:0 2>/dev/null
  else
    ydotool key -d 5 29:1 47:1 47:0 29:0 2>/dev/null
  fi
  exit 0
fi

# --- the rows ---------------------------------------------------------------
# One JSON object per entry: the entry's text as the row and as its search text,
# its id as the value, and the decoded body as the detail a Detail pane shows.
# cliphist list is already newest-first, so no sorting is needed to keep recent
# clips at the top.
rows() {
  cliphist list | while IFS=$'\t' read -r id text; do
    case $text in
      '[[ binary'*)
        # The image itself would need a graphics protocol, which this picker
        # does not speak (it stays terminal-agnostic); say so instead of
        # showing nothing.
        detail="$text (image preview needs a graphics protocol)"
        ;;
      *)
        # NULs are dropped: a binary-ish entry would otherwise make every later
        # shell warn about a byte it cannot hold, and no pane can draw one.
        detail=$(printf '%s' "$id" | cliphist decode 2>/dev/null | head -c "$preview_bytes" | tr -d '\000' | tr '\n' '\v')
        ;;
    esac
    printf '%s\t%s\t%s\n' "$id" "$text" "$detail"
  done | jq -Rc 'split("\t") | {display: .[1], value: .[0], search: .[1], detail: (.[2] // "" | gsub("\u000b"; "\n"))}'
}

if [ "${1:-}" = "--list" ]; then
  rows
  exit 0
fi

choice=$(rows | "$adapter" "$spec" --in json cat -) || exit 0
[ -n "$choice" ] || exit 0
if [ "${CLIPBOARD_PRINT_ONLY:-}" = 1 ]; then
  printf '%s\n' "$choice"
  exit 0
fi

printf '%s' "$choice" | cliphist decode 2>/dev/null | wl-copy 2>/dev/null
hyprctl dispatch "hl.dsp.exec_cmd('$script --paste')" >/dev/null 2>&1
exit 0
