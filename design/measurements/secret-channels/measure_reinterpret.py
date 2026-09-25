"""R2 NC1: which spellings does each CLI RE-INTERPRET when they arrive as an argv VALUE (the
interim path puts resolved bytes on argv)? For every single-input row, with --allow-argv-secret:
  `@env:REINT_VAR` on argv, REINT_VAR = the fixture secret  -> re-interpreted iff the output
                                                             equals the secret's own output
  `-` on argv, stdin = the fixture secret                    -> likewise
Aggregated per CLI (a spelling re-interpreted on ANY input counts for that CLI: conservative).
Writes reinterpret.json; channel_policy.json's `argv_reinterprets` must equal its aggregate
(test_plan.py), and both carry the CLI version they were measured with."""
import json, os, subprocess
from concurrent.futures import ThreadPoolExecutor
from cases3 import C, C2, C3

B = os.environ["BIN_DIR"].rstrip("/") + "/"
cases = {c["label"]: c for c in C + C2 + C3}
cases.pop("ms combine <shares> (first)", None)
rows = {}
for f in ["channels.json", "channels2.json", "channels3.json"]:
    for r in json.load(open(f)):
        rows[r["label"]] = r


def run(argv, stdin="", extra_env=None):
    env = dict(os.environ); env.update(extra_env or {})
    p = subprocess.run(argv, input=stdin.encode(), capture_output=True, env=env, timeout=180)
    return p.returncode, p.stdout


def measure(label):
    c = cases[label]
    chk = c.get("check", lambda o: o)
    def out(argv, **kw):
        rc, so = run(argv, **kw)
        return rc, chk(so.decode("utf-8", "surrogateescape")) if rc in (0, 4) else ""
    base = out([a.replace("{S}", c["S"]) for a in c["argv"]])
    env_tok = out([a.replace("{S}", "@env:REINT_VAR") for a in c["argv"]], extra_env={"REINT_VAR": c["S"]})
    dash = out([a.replace("{S}", "-") for a in c["argv"]], stdin=c["S"] + "\n")
    return {"label": label, "cli": label.split()[0], "@env:": env_tok == base, "-": dash == base}


labels = [l for l in cases if l in rows and rows[l]["base_exit"] in (0, 4) and rows[l]["depends"]]
with ThreadPoolExecutor(20) as ex:
    res = list(ex.map(measure, labels))
ver = {}
for cli in ("mnemonic", "md", "ms", "mk"):
    ver[cli] = subprocess.run([B + cli, "--version"], capture_output=True, text=True, stdin=subprocess.DEVNULL).stdout.split()[1]
agg = {}
for r in res:
    a = agg.setdefault(r["cli"], {"version": ver[r["cli"]], "spellings": set(), "inputs": {}})
    for sp in ("-", "@env:"):
        if r[sp]:
            a["spellings"].add(sp)
            a["inputs"].setdefault(sp, []).append(r["label"])
out = {k: {"version": v["version"], "spellings": sorted(v["spellings"]),
           "inputs": {sp: sorted(ls) for sp, ls in v["inputs"].items()}} for k, v in sorted(agg.items())}
json.dump(out, open("reinterpret.json", "w"), indent=1)
for k, v in out.items():
    print(k, v["version"], v["spellings"], {sp: len(ls) for sp, ls in v["inputs"].items()})
