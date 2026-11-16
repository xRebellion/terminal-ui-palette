#!/usr/bin/env bash
# Launch a picker in its own terminal window, sized from the document's host
# section, with a class a compositor rule can match.
#
#   tuipalette-floating.sh layouts/rofi-like.ron [provider command...]
#
#   scripts/tuipalette-floating.sh layouts/rofi-like.ron scripts/apps.sh
#
# The window is placed by the compositor, not by the picker (docs/PLAN.md, D3).
# Put the rule in your config:
#
#   windowrule = float, class:^(tuipalette)$
#   windowrule = center, class:^(tuipalette)$
#
# TUIPALETTE_DRY_RUN=1 prints the command instead of running it, which is what
# the tests use. TUIPALETTE_TERMINAL and TUIPALETTE_BIN override the defaults so
# a different terminal or a development build can be tried without editing this.
set -euo pipefail

layout=${1:?usage: tuipalette-floating.sh <layout> [picker flags...] [provider command...]}
shift

# Picker flags come first (--prompt, --message, --no-sort, ...); everything
# else is the provider — a command whose stdout is piped into the picker
# *inside* the terminal. A floating window has no stdin of its own, so this
# is how items reach it (docs/PLAN.md, 08).
picker_flags=()
while [ $# -gt 0 ]; do
  case $1 in
    --prompt | --message | --theme | --height) picker_flags+=("$1" "${2:?$1 needs a value}"); shift 2 ;;
    --no-sort | --accept-query | --print-query | --icons | --no-icons) picker_flags+=("$1"); shift ;;
    *) break ;;
  esac
done
provider=("$@")

# A compositor-spawned script inherits a minimal PATH and the binary is not
# installed anywhere, so the default is the build next to this checkout. Checked
# before anything else: without it the terminal opens, the shell inside fails
# with 127 and the window disappears before anyone can read why.
here=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
repo=$(dirname "$here")
binary=${TUIPALETTE_BIN:-$repo/target/release/tuipalette}
terminal=${TUIPALETTE_TERMINAL:-kitty}

if [ ! -x "$binary" ]; then
  echo "tuipalette-floating: no picker at $binary — run 'cargo build --release' or set TUIPALETTE_BIN" >&2
  exit 2
fi
class=$("$binary" --spec "$layout" --print-class)

geometry=$("$binary" --spec "$layout" --print-geometry || true)
sizing=()
if [ -n "$geometry" ]; then
  width=${geometry%x*}
  height=${geometry#*x}
  # remember_window_size would let the previous window size win over the spec.
  # close_on_child_death: kitty keeps a window open after its shell exits by
  # default, so every cancel would leave a dead window behind.
  sizing=(--override remember_window_size=no --override close_on_child_death=yes --override "initial_window_width=${width}c" --override "initial_window_height=${height}c")
fi

result=$(mktemp -t tuipalette-result.XXXXXX)
trap 'rm -f "$result"' EXIT

# `cat -` is the idiom for "the items are on my stdin", but inside the
# terminal `-` names the terminal itself: cat would eat the keystrokes and
# feed them to the picker as items, so typing and Escape would do nothing.
# Read that stdin here, over the pipe, and give the window a file instead.
if [ "${provider[0]:-}" = cat ] && [ "${provider[1]:-}" = - ] && [ "${#provider[@]}" -eq 2 ]; then
  items=$(mktemp -t tuipalette-items.XXXXXX)
  cat > "$items"
  trap 'rm -f "$result" "$items"' EXIT
  provider=(cat "$items")
fi

inner=$(printf '%q ' "$binary" --spec "$layout" --out-file "$result" "${picker_flags[@]}")
if [ "${#provider[@]}" -gt 0 ]; then
  inner="$(printf '%q ' "${provider[@]}") | $inner"
fi

command=("$terminal" --class "$class" --title "$class" "${sizing[@]}" sh -c "$inner")
if [ -n "${TUIPALETTE_DRY_RUN:-}" ]; then
  printf '%s\n' "${command[*]}"
  exit 0
fi

# A terminal has no stdout to hand the picker, so the adapter reads the file the
# picker was told to write and answers with it — the caller sees the same
# "one value on stdout" as the inline host.
"${command[@]}"
cat "$result"
