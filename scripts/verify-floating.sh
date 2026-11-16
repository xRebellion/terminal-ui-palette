#!/usr/bin/env bash
# Verify the floating host for real, without taking the user's focus.
#
# The window is launched with a class the compositor is told to keep off screen:
#   hl.window_rule({ match = { class = "^(tuipalette-verify)$" },
#                    workspace = "special:tuipalette-verify silent",
#                    float = true, no_initial_focus = true })
# so it maps on a hidden special workspace and never holds focus. The script
# then asserts that focus really did not move.
#
#   scripts/verify-floating.sh [layout]
set -uo pipefail
cd "$(dirname "$0")/.." || exit 1

layout=${1:-layouts/rofi-like.ron}
binary=${TUIPALETTE_BIN:-$PWD/target/release/tuipalette}
[ -x "$binary" ] || { echo "no binary at $binary — cargo build --release first"; exit 2; }

if ! hyprctl version >/dev/null 2>&1; then
  echo "no Hyprland session — nothing to verify here"
  exit 0
fi

work=$(mktemp -d)
class=tuipalette-verify
before_window=$(hyprctl activewindow -j | jq -r '.class // ""')
before_workspace=$(hyprctl activeworkspace -j | jq -r '.name // ""')

cleanup() {
  pkill -f "kitty --class [t]uipalette" >/dev/null 2>&1
  rm -rf "$work"
}
trap cleanup EXIT

# A copy of the document whose window class is the one the rule matches.
sed "s/class: \"tuipalette\"/class: \"$class\"/" "$layout" > "$work/layout.ron"
printf 'alpha\nbeta\ngamma\n' > "$work/items"
cat > "$work/run.sh" <<RUN
#!/usr/bin/env bash
export TUIPALETTE_BIN="$binary"
export PATH="/usr/local/bin:/usr/bin:/bin"
cd "$PWD" || exit 1
exec scripts/tuipalette-floating.sh "$work/layout.ron" cat "$work/items" > "$work/value" 2> "$work/err"
RUN
chmod +x "$work/run.sh"

hyprctl dispatch "hl.dsp.exec_cmd('$work/run.sh')" >/dev/null
sleep 2.5

pid=$(pgrep -f "kitty --class [t]uipalette" | tail -1)
client=$(hyprctl clients -j | jq -c --arg pid "${pid:-0}" '.[] | select(.pid == ($pid|tonumber)) | {class,floating,size,workspace:.workspace.name,mapped}')
echo "client: ${client:-none}"
size=$(hyprctl clients -j | jq -r --arg pid "${pid:-0}" '.[] | select(.pid == ($pid|tonumber)) | .size | @tsv' 2>/dev/null)
echo "document geometry: $("$binary" --spec "$work/layout.ron" --print-geometry || echo missing); window pixels: ${size:-none}"

mid_window=$(hyprctl activewindow -j | jq -r '.class // ""')
mid_workspace=$(hyprctl activeworkspace -j | jq -r '.name // ""')
shown=$(hyprctl monitors -j | jq -r '[.[] | select(.specialWorkspace.name == "special:tuipalette-verify")] | length')
echo "focus while open: ${mid_window:-?}/${mid_workspace:-?}; workspace visible: $shown"

if [ -n "${pid:-}" ]; then
  kitten @ --to "unix:/tmp/kitty-$pid" send-text --all beta >/dev/null 2>&1
  sleep 0.3
  kitten @ --to "unix:/tmp/kitty-$pid" send-key --all enter >/dev/null 2>&1
  sleep 1.5
fi

value=$(cat "$work/value" 2>/dev/null || true)
echo "value: ${value:-<none>}"
echo "stderr: $(head -2 "$work/err" 2>/dev/null | tr '\n' ' ')"

after_window=$(hyprctl activewindow -j | jq -r '.class // ""')
after_workspace=$(hyprctl activeworkspace -j | jq -r '.name // ""')
left=$(hyprctl clients -j | jq -c '[.[] | select(.class|test("tuipalette"))] | length')
echo "focus before: ${before_window:-?}/${before_workspace:-?} after: ${after_window:-?}/${after_workspace:-?}; windows left: $left"

status=0
[ "$value" = "beta" ] || { echo "FAIL: expected the chosen value back"; status=1; }
[ "$left" = "0" ] || { echo "FAIL: a picker window is still open"; status=1; }
[ "$shown" = "0" ] || { echo "FAIL: the hidden workspace became visible"; status=1; }
[ "${before_window:-x}" = "${after_window:-x}" ] || { echo "FAIL: the active window changed"; status=1; }
[ "${before_workspace:-x}" = "${after_workspace:-x}" ] || { echo "FAIL: the active workspace changed"; status=1; }
[ "${mid_window:-x}" = "${before_window:-x}" ] || { echo "FAIL: the launch took focus"; status=1; }
echo "result: $([ $status = 0 ] && echo ok || echo failed)"
exit $status
