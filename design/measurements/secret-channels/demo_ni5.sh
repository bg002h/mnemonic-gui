#!/bin/bash
# R3 NI5 demonstration: the CI gate (regen_check.py) against F-687 builds that report the SAME
# version strings as the releases (mnemonic 0.104.0, ms 0.19.1).
#   F687_BIN_DIR = F-687 master: mnemonic from toolkit 9da32e2f + ms from mnemonic-secret d1ab447 (+ release md, mk)
# (A) the committed cache, measured on the releases, checked against the F-687 binaries;
# (B) R3's scenario: a bump that re-derived everything on F-687 but left ms's re-read row at `[-]`.
# (C) the bump done right (everything re-derived on F-687): must be GREEN — the bump is data-only.
# Fold 5 (R4 NI8/NI9): the planner's safety no longer reads the derived data, so a stale cache is
# caught by identity and the diff, and can no longer produce OTHER's wallet at runtime.
set -u
cd "$(dirname "$0")"
: "${F687_BIN_DIR:?set F687_BIN_DIR}"
echo "F-687 binaries: $("$F687_BIN_DIR/mnemonic" --version), $("$F687_BIN_DIR/ms" --version)"
echo; echo "=== (A) committed cache vs F-687 binaries: BIN_DIR=F687 regen_check.py --plans"
BIN_DIR=$F687_BIN_DIR python3 regen_check.py --plans | sed -E 's#/(tmp|scratch)[^ )]*#<path>#g'
echo "exit ${PIPESTATUS[0]}"
echo; echo "=== (B) re-derived on F-687, ms row left at [-]: regen_check.py --data-from STALE --plans"
STALE=$(mktemp -d); trap 'rm -r "$STALE"' EXIT
BIN_DIR=$F687_BIN_DIR ./derive_into.sh "$STALE"
echo; echo "=== (C) the bump done right: everything re-derived on F-687: regen_check.py --data-from FRESH --plans"
FRESH=$(mktemp -d); cp "$STALE"/* "$FRESH/"
BIN_DIR=$F687_BIN_DIR python3 regen_check.py --data-from "$FRESH" --plans | sed -E 's#/(tmp|scratch)[^ )]*#<path>#g'
echo "exit ${PIPESTATUS[0]}"; rm -r "$FRESH"
echo; echo "=== (B) continued: the stale variant"
python3 - "$STALE/reinterpret.json" <<'PY'
import json, sys
p = sys.argv[1]; d = json.load(open(p))
print(f"  F-687 measured ms re-reads: {d['ms']['spellings']}; the stale bump leaves it at ['-']")
d["ms"]["spellings"] = ["-"]; d["ms"]["inputs"] = {"-": d["ms"]["inputs"].get("-", [])}
json.dump(d, open(p, "w"), indent=1)
PY
BIN_DIR=$F687_BIN_DIR python3 regen_check.py --data-from "$STALE" --plans | sed -E 's#/(tmp|scratch)[^ )]*#<path>#g'
echo "exit ${PIPESTATUS[0]}"
