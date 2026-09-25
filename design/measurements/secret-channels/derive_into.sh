#!/bin/bash
# derive_into.sh OUT_DIR — run the full §A1 derivation against $BIN_DIR in a scratch copy of the
# repo layout, and copy the derived cache into OUT_DIR (used by the demos; CI uses regen_check.py).
set -eu
: "${BIN_DIR:?set BIN_DIR}"; OUT=$1; mkdir -p "$OUT"
HERE=$(cd "$(dirname "$0")" && pwd); REPO=$(cd "$HERE/../../.." && pwd)
root=$(mktemp -d); trap 'rm -r "$root"' EXIT
mkdir -p "$root/design/measurements"
cp "$REPO/pinned-upstream.toml" "$root/"; cp -r "$REPO/.github" "$root/"
cp -r "$HERE" "$root/design/measurements/secret-channels"
cd "$root/design/measurements/secret-channels"
for s in run_all.py run2.py run3.py measure_groups.py run_bytes.py table_build.py measure_reinterpret.py probe_missing.py; do python3 "$s" > /dev/null; done
cp channel_table.json reinterpret.json measured_with.json groups.json bytes.json channels.json channels2.json channels3.json \
   secret_sources.txt missing_sources.txt missing_sources.md "$OUT/"
