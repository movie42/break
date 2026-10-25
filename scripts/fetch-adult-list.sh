#!/bin/sh
set -eu

BASE="https://raw.githubusercontent.com/StevenBlack/hosts/master/alternates"
OUT="$(cd "$(dirname "$0")/.." && pwd)/src/system/break-app/resources/adult-hosts.txt"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

curl -fsSL "$BASE/porn-only/hosts" -o "$TMP/porn"
curl -fsSL "$BASE/gambling-only/hosts" -o "$TMP/gambling"

mkdir -p "$(dirname "$OUT")"
{
  echo "# StevenBlack/hosts porn-only + gambling-only (MIT, parts CC BY-SA)"
  echo "# fetched $(date -u +%Y-%m-%dT%H:%M:%SZ)"
  cat "$TMP/porn" "$TMP/gambling" \
    | sed 's/#.*//' \
    | awk '$1 == "0.0.0.0" && $2 != "0.0.0.0" { print $2 }' \
    | sort -u
} > "$OUT"

echo "$(grep -vc '^#' "$OUT") hosts -> $OUT"
