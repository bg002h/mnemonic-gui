#!/bin/bash
# R4 Nm15: adding a secret source with no channel-table entry must turn T4 (regen_check.py) red.
#  (1) the schema mirror gains a secret source (ms derive --account marked `secret: true` in a
#      scratch copy of the GUI repo) — caught by the Rust re-enumeration AND the coverage check;
#  (2) only the cache is edited (a source appended to secret_sources.txt) — caught by coverage.
set -u
cd "$(dirname "$0")"
: "${BIN_DIR:?set BIN_DIR}"
REPO=$(cd ../../.. && pwd)
S=$(mktemp -d); trap 'rm -r "$S"' EXIT
( cd "$REPO" && git ls-files -z | xargs -0 -I{} cp --parents {} "$S/" )
mkdir -p "$S/design/measurements/secret-channels"; cp -r ./. "$S/design/measurements/secret-channels/"
echo "=== (1) mirror change: ms derive --account marked secret (scratch copy of the repo)"
python3 - "$S/src/schema/ms.rs" <<'PY'
import sys
p = sys.argv[1]; s = open(p).read()
old = '''        name: "--account",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "Account index for --template (default 0).",
        secret: false,'''
assert s.count(old) == 1
open(p, "w").write(s.replace(old, old.replace("secret: false,", "secret: true,")))
PY
ENUM_TARGET_DIR="$REPO/target/secret-sources-enum" python3 "$S/design/measurements/secret-channels/regen_check.py" | sed -E 's#/(tmp|scratch)[^ )]*#<path>#g'
echo "exit ${PIPESTATUS[0]}"
echo; echo "=== (2) cache-only change: a source appended to secret_sources.txt, no table entry"
D=$(mktemp -d); cp ./*.json ./*.txt ./*.md "$D/"; echo "ms derive --account" >> "$D/secret_sources.txt"
python3 regen_check.py --data-from "$D" | sed -E 's#/(tmp|scratch)[^ )]*#<path>#g'
echo "exit ${PIPESTATUS[0]}"; rm -r "$D"
echo; echo "=== (3) R4 N4: a missing cache file is a named RED line, not a traceback"
D=$(mktemp -d); cp ./*.json ./*.txt ./*.md "$D/"; rm "$D/reinterpret.json"
python3 regen_check.py --data-from "$D" --no-enumerate 2>&1 | sed -E 's#/(tmp|scratch)[^ )]*#<path>#g'
echo "exit ${PIPESTATUS[0]}"; rm -r "$D"
