#!/usr/bin/env bash
# The app-launcher entry point. One line decides which picker runs, so the old
# one stays a single edit away (docs/PLAN.md, 13).
set -euo pipefail
cd "$(dirname "$0")/.." || exit 1

PICKER=${PICKER:-tuipalette}

if [ "$PICKER" = rofi ]; then
  choice=$(rofi -show drun -dmenu -i)
else
  choice=$(scripts/apps.sh | scripts/usage.sh order 2 \
    | scripts/tuipalette-floating.sh layouts/rofi-like.ron cat -)
fi

[ -n "$choice" ] || exit 0
scripts/usage.sh record "$choice"
# Detached: the launcher exits, the app keeps running.
setsid sh -c "$choice" >/dev/null 2>&1 &
