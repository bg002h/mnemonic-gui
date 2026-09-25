#!/bin/bash
# R4 NI8 / NI9: CLI behaviour OUTSIDE the probe set, simulated by wrappers around the F-687 master
# builds, then a full and CORRECT re-derivation and the CI gate. Fold 5 must be safe regardless.
#   W2: ms trims a --passphrase value before recognising `-` / `@env:` (R4's NI8 wrapper)
#   W1: mnemonic's --passphrase @env: strips EVERY trailing newline (R4's NI9 wrapper)
set -u
cd "$(dirname "$0")"
: "${F687_BIN_DIR:?set F687_BIN_DIR}"
W=$(mktemp -d); trap 'rm -r "$W"' EXIT
mkdir -p "$W/w1" "$W/w2"
for d in w1 w2; do cp "$F687_BIN_DIR"/{md,mk,mnemonic,ms} "$W/$d/"; done
mv "$W/w2/ms" "$W/w2/ms.real"
cat > "$W/w2/ms" <<'SH'
#!/bin/bash
real="$(dirname "$0")/ms.real"; out=(); prev=""
trim() { local v="$1"; v="${v#"${v%%[![:space:]]*}"}"; v="${v%"${v##*[![:space:]]}"}"; printf '%s' "$v"; }
for a in "$@"; do
  if [ "$prev" = "--passphrase" ]; then t=$(trim "$a"); case "$t" in -|@env:*) a="$t";; esac; fi
  case "$a" in --passphrase=*) t=$(trim "${a#--passphrase=}"); case "$t" in -|@env:*) a="--passphrase=$t";; esac;; esac
  out+=("$a"); prev="$a"
done
exec "$real" "${out[@]}"
SH
mv "$W/w1/mnemonic" "$W/w1/mnemonic.real"
cat > "$W/w1/mnemonic" <<'SH'
#!/bin/bash
real="$(dirname "$0")/mnemonic.real"; out=(); prev=""
for a in "$@"; do
  if [ "$prev" = "--passphrase" ] && [[ "$a" == @env:* ]]; then
    name="${a#@env:}"; v="${!name-}"
    while [[ "$v" == *$'\n' ]]; do v="${v%$'\n'}"; v="${v%$'\r'}"; done
    export W1_STRIPPED="$v"; a="@env:W1_STRIPPED"
  fi
  out+=("$a"); prev="$a"
done
exec "$real" "${out[@]}"
SH
chmod +x "$W/w1/mnemonic" "$W/w2/ms"
P="abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
MS1=ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f
echo "=== wrappers behave as R4 describes (OTHER = hunter2-passphrase):"
echo "  W2 ms derive --passphrase ' @env:OTHER' (argv) : $(OTHER=hunter2-passphrase "$W/w2/ms" derive --allow-argv-secret --passphrase ' @env:OTHER' -- $MS1 2>/dev/null | grep -m1 master_fingerprint)   (OTHER's wallet is 45fbfbe6)"
echo "  real   ms derive --passphrase ' @env:OTHER' (argv) : $(OTHER=hunter2-passphrase "$F687_BIN_DIR/ms" derive --allow-argv-secret --passphrase ' @env:OTHER' -- $MS1 2>/dev/null | grep -m1 master_fingerprint)"
fp() { grep -o 'master fingerprint: [0-9a-f]*' | awk '{print $3}'; }
echo "  W1 restore --passphrase @env:X, X='hunter2-passphrase\\n\\n' : $(X=$'hunter2-passphrase\n\n' "$W/w1/mnemonic" restore --allow-argv-secret --from "phrase=$P" --template bip84 --passphrase @env:X 2>/dev/null | fp)"
echo "  real  restore --passphrase @env:X, same X                  : $(X=$'hunter2-passphrase\n\n' "$F687_BIN_DIR/mnemonic" restore --allow-argv-secret --from "phrase=$P" --template bip84 --passphrase @env:X 2>/dev/null | fp)"
for w in w2 w1; do
  echo; echo "=== $w: full correct re-derivation, then regen_check --data-from it --plans"
  D=$(mktemp -d)
  BIN_DIR="$W/$w" ./derive_into.sh "$D"
  echo "  derived: $(python3 -c "
import json; t=json.load(open('$D/channel_table.json')); r=json.load(open('$D/reinterpret.json'))
from collections import Counter
print('rules', dict(Counter(str(v['cli_env_rule']) for v in t.values())), '| ms re-reads', r['ms']['spellings'])")"
  BIN_DIR="$W/$w" python3 regen_check.py --data-from "$D" --plans | sed -E 's#/(tmp|scratch)[^ )]*#<path>#g' | cut -c1-400
  echo "  exit ${PIPESTATUS[0]}"
  rm -r "$D"
done
echo; echo "=== the planner on R4's cases, fold 5 (pure; any OS):"
python3 - <<'PY'
import json, plan as P
from shapes import SHAPES
T = json.load(open("channel_table.json"))
for name, i, content, rule in [("ms derive ms1+passphrase", 0, " @env:OTHER", None), ("restore phrase+passphrase", 1, "hunter2-passphrase\n\n", "strip-one-trailing-newline")]:
    sh = [s for s in SHAPES if s["name"] == name][0]
    srcs = [dict(x) for x in sh["sources"]]; srcs[i]["value"] = "@env:USER_SECRET"
    for osn in ("linux", "macos"):
        try:
            P.plan(srcs, T, osn, {"USER_SECRET": content, "OTHER": "hunter2-passphrase"}, rule); out = "PLANNED"
        except P.Refusal as e:
            out = "refused " + e.code
        print(f"  {name}, variable = {content!r}, {osn}: {out}")
PY
