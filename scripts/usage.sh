#!/usr/bin/env bash
# Usage memory for the launcher and the command runner: how often a line has
# been chosen, and when it was chosen last.
#
#   list | usage.sh order <key-field>   # order rows, most recently used first
#   usage.sh record <key>              # after the user accepted <key>
#
# This is provider logic, not library logic (D12): the picker sorts by match
# score and keeps arrival order for ties, so a list handed to it in usage order
# is shown in usage order until the user types.
set -uo pipefail

usage_file=${TUIPALETTE_USAGE:-${XDG_CACHE_HOME:-$HOME/.cache}/tuipalette/usage.tsv}

record() {
  local key=$1 now
  [ -n "$key" ] || return 0
  now=$(date +%s)
  mkdir -p "$(dirname "$usage_file")"
  touch "$usage_file"
  awk -F'\t' -v k="$key" -v now="$now" '
    $1 == k { print k "\t" ($2 + 1) "\t" now; found = 1; next }
    { print }
    END { if (!found) print k "\t1\t" now }
  ' "$usage_file" > "$usage_file.new" && mv "$usage_file.new" "$usage_file"
}

order() {
  local field=${1:-1} tmp
  tmp=$(mktemp) || return 1
  cat > "$tmp"
  if [ -s "$usage_file" ]; then
    # -s keeps arrival order for everything never used, so the list does not
    # shuffle itself between runs.
    awk -F'\t' -v field="$field" '
      NR == FNR { count[$1] = $2; last[$1] = $3; next }
      {
        key = $field
        c = (key in count) ? count[key] : 0
        l = (key in last) ? last[key] : 0
        printf "%012d\t%08d\t%s\n", l, c, $0
      }
    ' "$usage_file" "$tmp" | sort -s -t $'\t' -k1,1nr -k2,2nr | cut -f3-
  else
    cat "$tmp"
  fi
  rm -f "$tmp"
}

case ${1:-} in
  record) shift; record "${1:-}" ;;
  order) shift; order "${1:-1}" ;;
  *) echo "usage: usage.sh order <key-field> | record <key>" >&2; exit 2 ;;
esac
