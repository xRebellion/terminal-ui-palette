#!/usr/bin/env bash
# Launch a picker in its own terminal window, sized from the document's host
# section, with a class a compositor rule can match.
#
#   tuipalette-floating.sh layouts/rofi-like.ron [extra tuipalette flags]
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

spec=${1:?usage: tuipalette-floating.sh <spec> [tuipalette args]}
shift

binary=${TUIPALETTE_BIN:-tuipalette}
terminal=${TUIPALETTE_TERMINAL:-kitty}
class=$("$binary" --spec "$spec" --print-class)

geometry=$("$binary" --spec "$spec" --print-geometry || true)
sizing=()
if [ -n "$geometry" ]; then
  width=${geometry%x*}
  height=${geometry#*x}
  sizing=(--override "initial_window_width=${width}c" --override "initial_window_height=${height}c")
fi

command=("$terminal" --class "$class" --title "$class" "${sizing[@]}" "$binary" --spec "$spec" "$@")
if [ -n "${TUIPALETTE_DRY_RUN:-}" ]; then
  printf '%s\n' "${command[*]}"
  exit 0
fi
exec "${command[@]}"
