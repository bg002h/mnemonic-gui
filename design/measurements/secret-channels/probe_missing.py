"""For every schema secret source with NO channel-table entry: does the CLI accept that input at
all? Run it once on argv (+ --allow-argv-secret) with a real value and record the verdict.
Input: secret_sources.txt (the schema enumeration). Output: missing_sources.txt."""
import json, os, re, subprocess
from shapes import P, MS1, WIF, BIP38_A, BPJ, FX
B = os.environ["BIN_DIR"].rstrip("/") + "/"
T = json.load(open("channel_table.json"))
V = {"phrase": P, "entropy": "00000000000000000000000000000000", "ms1": MS1, "wif": WIF, "bip38": BIP38_A,
     "seedqr": "000000000000000000000000000000000000000000000003", "minikey": "S6c56bnXQiBjk9mqSYE7ykVQ7NzrRy",
     "electrum-phrase": "wild father tree among universe such mobile favorite target dynamic credit identify",
     "xprv": "xprv9s21ZrQH143K3GJpoapnV8SFfukcVBSfeCficPSGfubmSFDxo1kuHnLisriDvSnRRuL2Qrg5ggqHKNVpxR86QEC8w35uxmGoggxtQTPvfUu"}
CTX = {"addresses": ["--address-type", "p2wpkh", "--count", "1"], "restore": ["--template", "bip84"],
       "bundle": ["--network", "mainnet", "--template", "bip84"], "export-wallet": ["--network", "mainnet", "--template", "bip84"],
       "verify-bundle": ["--network", "mainnet", "--template", "bip84", "--mk1"] + BPJ["mk1"] + ["--md1"] + BPJ["md1"],
       "import-wallet": ["--blob", os.path.join(FX, "blob-2of2.bsms"), "--format", "bsms", "--json"], "word-card": []}
rows = []
for key in [l.strip().replace(" [group]", "") for l in open("secret_sources.txt") if l.strip()]:
    if key in T:
        continue
    m = re.fullmatch(r"mnemonic (\S+) (--from|--slot) (?:@N\.)?(\S+)=", key)
    sub, flag, node = m.groups()
    tok = f"@0.{node}={V[node]}" if flag == "--slot" else f"{node}={V[node]}"
    r = subprocess.run([B + "mnemonic", sub, "--allow-argv-secret"] + CTX[sub] + [flag, tok], capture_output=True, text=True)
    err = next((l.strip() for l in r.stderr.splitlines() if l.strip().startswith("error")), "")
    rows.append((key, r.returncode, err[:150]))
with open("missing_sources.txt", "w") as f:
    for k, rc, e in rows:
        f.write(f"{k}\texit {rc}\t{e}\n")
print(open("missing_sources.txt").read())

REJECTS = ("not supported", "not a seed source", "watch-only by definition", "only the `phrase` subkey",
           "recognized HRP prefix")
with open("missing_sources.md", "w") as f:
    f.write("| input with no table entry | argv run (+opt-in) | verdict |\n|---|---|---|\n")
    for k, rc, e in rows:
        verdict = "CLI rejects this input itself" if any(x in e for x in REJECTS) else "**unmeasured**"
        f.write(f"| `{k}` | exit {rc}: {e.replace('|', '/') or '(no error line)'} | {verdict} |\n")
