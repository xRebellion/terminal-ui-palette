#!/usr/bin/env bash
# The app-launcher entry point. One line decides which picker runs, so the old
# one stays a single edit away (docs/PLAN.md, 13).
set -euo pipefail
cd "$(dirname "$0")/.." || exit 1

PICKER=${PICKER:-tuipalette}

if [ "$PICKER" = rofi ]; then
  choice=$(rofi -show drun -dmenu -i)
else
  choice=$(scripts/tuipalette-floating.sh layouts/rofi-like.ron scripts/apps.sh)
fi

[ -n "$choice" ] || exit 0
# Detached: the launcher exits, the app keeps running.
setsid sh -c "$choice" >/dev/null 2>&1 &
