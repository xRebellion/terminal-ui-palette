#!/usr/bin/env bash
# TTY check, second attempt: the pty needs a size or the app draws into 0x0.
set -u
S="$HOME/Code/hobby/terminal-ui-lib/spike"
BIN="$S/target/debug/ratatui_spike"
SPEC="$S/spec.ron"
cd "$S" || exit 9
CMD="stty rows 22 cols 72 2>/dev/null; $BIN $SPEC"

echo "=== case 1: esc cancels ==="
rm -f /tmp/tty-cancel.ts
( sleep 1; printf '\033'; sleep 3 ) | SHELL=/bin/sh timeout 15 script -q -e -c "$CMD" /tmp/tty-cancel.ts >/dev/null 2>&1
echo "exit=$?  (expect 1)"
for s in firefox «search»; do :; done
for s in firefox search results; do
  printf 'text %-9s: ' "$s"; grep -c -- "$s" /tmp/tty-cancel.ts
done
printf 'alt-screen in/out: '; printf '%s/' "$(grep -c $'\033\[?1049h' /tmp/tty-cancel.ts)"; grep -c $'\033\[?1049l' /tmp/tty-cancel.ts
printf 'cursor re-shown:   '; grep -c $'\033\[?25h' /tmp/tty-cancel.ts
printf 'tail bytes: '; tail -c 60 /tmp/tty-cancel.ts | cat -v | tr '\n' ' '; echo

echo
echo "=== case 2: enter accepts ==="
rm -f /tmp/tty-accept.ts /tmp/tty-accept.value
( sleep 1; printf '\r'; sleep 3 ) | SHELL=/bin/sh timeout 15 script -q -e -c "stty rows 22 cols 72 2>/dev/null; $BIN $SPEC > /tmp/tty-accept.value" /tmp/tty-accept.ts >/dev/null 2>&1
echo "exit=$?  (expect 0)"
printf 'stdout value: [%s]\n' "$(cat /tmp/tty-accept.value)"
printf 'cursor re-shown in stdout: '; grep -c $'\033\[?25h' /tmp/tty-accept.value

echo
echo "=== case 3: esc twice / arrow keys do not crash ==="
rm -f /tmp/tty-keys.ts
( sleep 1; printf '\033[B\033[B\033[B\033[A'; sleep 0.5; printf '\033'; sleep 3 ) | SHELL=/bin/sh timeout 15 script -q -e -c "$CMD" /tmp/tty-keys.ts >/dev/null 2>&1
echo "exit=$?  (expect 1)"
pgrep -x ratatui_spike || echo "no stray picker process"
