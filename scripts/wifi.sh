#!/usr/bin/env bash
# The wifi provider: the SSID is the value, signal and security are decoration.
#
#   scripts/wifi.sh | tuipalette --spec layouts/wifi.ron
#
# Everything the picker needs is the protocol on stdout, so this stays a shell
# script and the library stays ignorant of wifi (item 11's checkable).
set -uo pipefail

if [ "${1:-}" = "--fake" ]; then
  # What the test feeds in when there is no radio to ask.
  printf '  home-net   84%%  wpa\thome-net\n  cafe       61%%  open\tcafe\n  office     38%%  wpa\toffice\n'
  exit 0
fi

# nmcli -t escapes ':' inside field values with a backslash.
nmcli -t -f IN-USE,SSID,SIGNAL,SECURITY device wifi list 2>/dev/null | awk -F: '
  {
    in_use = $1; ssid = $2; signal = $3; sec = $4
    if (ssid == "") next
    mark = (in_use == "*") ? "*" : " "
    lock = (sec == "" || sec == "--") ? "open" : "wpa"
    printf "%s %-28s %3s%%  %s\t%s\n", mark, ssid, signal, lock, ssid
  }'
