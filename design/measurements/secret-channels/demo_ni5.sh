#!/bin/bash
# R3 NI5 demonstration: the CI gate (regen_check.py) against F-687 builds that report the SAME
# version strings as the releases (mnemonic 0.104.0, ms 0.19.1).
#   F687_BIN_DIR = mnemonic from toolkit 9846a784 + ms from mnemonic-secret e534917 (+ release md, mk)
# (A) the committed cache, measured on the releases, checked against the F-687 binaries;
# (B) R3's scenario: a bump that re-derived everything on F-687 but left ms's re-read row at `[-]`.
# (C) the bump done right (everything re-derived on F-687): must be GREEN — the bump is data-only.
set -u
cd "$(dirname "$0")"
: "${F687_BIN_DIR:?set F687_BIN_DIR}"
echo "F-687 binaries: $("$F687_BIN_DIR/mnemonic" --version), $("$F687_BIN_DIR/ms" --version)"
echo; echo "=== (A) committed cache vs F-687 binaries: BIN_DIR=F687 regen_check.py --plans"
BIN_DIR=$F687_BIN_DIR python3 regen_check.py --plans | sed -E 's#/(tmp|scratch)[^ )]*#<path>#g'
echo "exit ${PIPESTATUS[0]}"
echo; echo "=== (B) re-derived on F-687, ms row left at [-]: regen_check.py --data-from STALE --plans"
STALE=$(mktemp -d); trap 'rm -r "$STALE"' EXIT
root=$(mktemp -d); mkdir -p "$root/design/measurements"
cp ../../../pinned-upstream.toml "$root/"; cp -r ../../../.github "$root/"; cp -r . "$root/design/measurements/secret-channels"
( cd "$root/design/measurements/secret-channels" && export BIN_DIR=$F687_BIN_DIR &&
  python3 run_all.py >/dev/null && python3 run2.py >/dev/null && python3 run3.py >/dev/null &&
  python3 measure_groups.py >/dev/null && python3 run_bytes.py >/dev/null && python3 table_build.py >/dev/null &&
  python3 measure_reinterpret.py >/dev/null &&
  cp channel_table.json reinterpret.json measured_with.json groups.json bytes.json channels.json channels2.json channels3.json "$STALE/" )
rm -r "$root"
echo; echo "=== (C) the bump done right: everything re-derived on F-687: regen_check.py --data-from FRESH --plans"
FRESH=$(mktemp -d); cp "$STALE"/* "$FRESH/"
BIN_DIR=$F687_BIN_DIR python3 regen_check.py --data-from "$FRESH" --plans | sed -E 's#/(tmp|scratch)[^ )]*#<path>#g'
echo "exit ${PIPESTATUS[0]}"; rm -r "$FRESH"
echo; echo "=== (B) continued: the stale variant"
python3 - "$STALE/reinterpret.json" <<'PY'
import json, sys
p = sys.argv[1]; d = json.load(open(p))
print(f"  F-687 measured ms re-reads: {d['ms']['spellings']}; the stale bump leaves it at ['-']")
d["ms"]["spellings"] = ["-"]; d["ms"]["inputs"].pop("@env:", None)
json.dump(d, open(p, "w"), indent=1)
PY
BIN_DIR=$F687_BIN_DIR python3 regen_check.py --data-from "$STALE" --plans | sed -E 's#/(tmp|scratch)[^ )]*#<path>#g'
echo "exit ${PIPESTATUS[0]}"
