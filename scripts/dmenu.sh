#!/usr/bin/env bash
# The dmenu entry point: lines in, one line out, and the same one-line switch.
#
#   printf 'a\nb\n' | scripts/dmenu.sh
#   scripts/dmenu.sh -p "pick one"
set -euo pipefail
cd "$(dirname "$0")/.." || exit 1

PICKER=${PICKER:-tuipalette}

if [ "$PICKER" = rofi ]; then
  exec rofi -dmenu -i "$@"
fi
prompt="select"
if [ "${1:-}" = "-p" ] && [ -n "${2:-}" ]; then
  prompt=$2
  shift 2
fi
exec "${TUIPALETTE_BIN:-tuipalette}" --spec layouts/dmenu.ron --prompt "$prompt" "$@"
