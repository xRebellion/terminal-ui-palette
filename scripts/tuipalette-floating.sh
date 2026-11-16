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

layout=${1:?usage: tuipalette-floating.sh <layout> [provider command...]}
shift

# Anything left is the provider: a command whose stdout is piped into the
# picker *inside* the terminal. A floating window has no stdin of its own, so
# this is how items reach it (docs/PLAN.md, 08).
provider=("$@")

binary=${TUIPALETTE_BIN:-tuipalette}
terminal=${TUIPALETTE_TERMINAL:-kitty}
class=$("$binary" --spec "$layout" --print-class)

geometry=$("$binary" --spec "$layout" --print-geometry || true)
sizing=()
if [ -n "$geometry" ]; then
  width=${geometry%x*}
  height=${geometry#*x}
  # remember_window_size would let the previous window size win over the spec.
  sizing=(--override remember_window_size=no --override "initial_window_width=${width}c" --override "initial_window_height=${height}c")
fi

result=$(mktemp -t tuipalette-result.XXXXXX)
trap 'rm -f "$result"' EXIT

inner=$(printf '%q ' "$binary" --spec "$layout" --out-file "$result")
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
