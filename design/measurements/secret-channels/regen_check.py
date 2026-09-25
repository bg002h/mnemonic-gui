"""T4 (R3 shape change): every BEHAVIOURAL file is a cache of a measurement. CI installs the
binaries for the tags in pinned-upstream.toml, re-derives every behavioural file in a scratch copy
of the repo layout, and diffs against the committed copies; ANY difference is red. It also checks
binary IDENTITY by content: the installed binaries' sha256 must equal measured_with.json's.

  BIN_DIR=<pinned binaries> python3 regen_check.py [--data-from DIR] [--plans]

--data-from DIR : compare against the derived files in DIR instead of this folder (the NI5 demo
                  uses it to present a deliberately stale cache).
--plans         : additionally run test_plan.py and run_plans.py (T3', oracle legs) against the binaries with the
                  COMMITTED (or --data-from) data — the runtime half: stale data that admits a
                  value the binaries re-read shows up there as OTHER's wallet.
Exit 1 on any difference."""
import argparse, hashlib, json, os, re, shutil, subprocess, sys, tempfile, tomllib
HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
REL = os.path.relpath(HERE, REPO)
DERIVED = ["channel_table.json", "reinterpret.json", "measured_with.json", "groups.json", "bytes.json"]
PIPELINE = [["run_all.py"], ["run2.py"], ["run3.py"], ["measure_groups.py"], ["run_bytes.py"],
            ["table_build.py"], ["measure_reinterpret.py"]]
ap = argparse.ArgumentParser()
ap.add_argument("--data-from")
ap.add_argument("--plans", action="store_true")
a = ap.parse_args()
B = os.environ["BIN_DIR"].rstrip("/") + "/"
data_dir = os.path.abspath(a.data_from) if a.data_from else HERE


def norm(text):
    return re.sub(r"/(tmp|scratch)[^ )\"']*", "<path>", text)


def scratch(copy_data_from=None):
    root = tempfile.mkdtemp()
    shutil.copy(os.path.join(REPO, "pinned-upstream.toml"), root)
    shutil.copytree(os.path.join(REPO, ".github"), os.path.join(root, ".github"))
    os.makedirs(os.path.join(root, "design"))
    shutil.copy(os.path.join(REPO, "design", "DESIGN_secret_channels_and_new_forms.md"), os.path.join(root, "design"))
    d = os.path.join(root, REL)
    shutil.copytree(HERE, d, ignore=shutil.ignore_patterns("__pycache__"))
    if copy_data_from:
        for f in DERIVED + ["channels.json", "channels2.json", "channels3.json"]:
            if os.path.exists(os.path.join(copy_data_from, f)):
                shutil.copy(os.path.join(copy_data_from, f), d)
    return root, d


fails = []
# 1. identity by content
mw = json.load(open(os.path.join(data_dir, "measured_with.json")))
pins = {k: v["tag"] for k, v in tomllib.load(open(os.path.join(REPO, "pinned-upstream.toml"), "rb")).items()
        if isinstance(v, dict) and "tag" in v}
for cli, tag in pins.items():
    sha = hashlib.sha256(open(B + cli, "rb").read()).hexdigest()
    if mw[cli]["pinned_tag"] != tag:
        fails.append(f"identity: {cli} data measured under {mw[cli]['pinned_tag']}, pinned {tag}")
    if mw[cli]["sha256"] != sha:
        ver = subprocess.run([B + cli, "--version"], capture_output=True, text=True, stdin=subprocess.DEVNULL).stdout.strip()
        fails.append(f"identity: {cli} binary sha256 {sha[:16]}… ({ver}) != measured {mw[cli]['sha256'][:16]}… "
                     f"(same version string: {ver.split()[-1] == mw[cli]['version']})")
# 2. re-derive and diff
root, d = scratch()
try:
    for step in PIPELINE:
        p = subprocess.run([sys.executable] + step, cwd=d, capture_output=True, text=True, env=dict(os.environ, BIN_DIR=B))
        if p.returncode:
            fails.append(f"derive: {step[0]} exit {p.returncode}: {(p.stderr.strip().splitlines() or [''])[-1][:200]}")
    for f in DERIVED:
        new, old = norm(open(os.path.join(d, f)).read()), norm(open(os.path.join(data_dir, f)).read())
        if new != old:
            detail = ""
            if f == "reinterpret.json":
                jn, jo = json.loads(new), json.loads(old)
                detail = "; ".join(f"{k}: committed {jo.get(k, {}).get('spellings')} vs measured {jn[k]['spellings']}"
                                   for k in jn if jn[k]["spellings"] != jo.get(k, {}).get("spellings"))
            elif f == "channel_table.json":
                jn, jo = json.loads(new), json.loads(old)
                ch = [k for k in jn if jn[k] != jo.get(k)]
                detail = f"{len(ch)} inputs differ, e.g. {ch[:4]}"
            fails.append(f"derived: {f} differs from a regeneration. {detail}")
finally:
    shutil.rmtree(root, ignore_errors=True)
# 3. optional runtime half: T3' oracle legs against these binaries with the given data
if a.plans:
    root, d = scratch(copy_data_from=data_dir)
    try:
        subprocess.run([sys.executable, "gen_plans.py"], cwd=d, capture_output=True)
        tp = subprocess.run([sys.executable, "test_plan.py"], cwd=d, capture_output=True, text=True)
        if tp.returncode:
            fails.append("test_plan.py on this data: " + (tp.stdout.strip().splitlines() or ["?"])[-1]
                         + " — " + "; ".join(l.strip() for l in tp.stdout.splitlines() if l.strip().startswith("FAIL"))[:600])
        p = subprocess.run([sys.executable, "run_plans.py"], cwd=d, capture_output=True, text=True, env=dict(os.environ, BIN_DIR=B))
        last = [l for l in p.stdout.splitlines() if l.startswith("FAILURES:")]
        if p.returncode:
            fails.append(f"T3' run_plans.py: {last[0] if last else 'exit ' + str(p.returncode)}")
            nc = [(r["name"], x) for r in json.load(open(os.path.join(d, "plans.json"))) for x in r.get("nc1", [])
                  if x.get("macos_is_other_wallet") or x.get("linux_is_other_wallet")]
            for n_, x in nc:
                fails.append(f"  NC1 OTHER's wallet: {n_} source {x['source']} content {x['content']!r}")
    finally:
        shutil.rmtree(root, ignore_errors=True)
for f in fails:
    print("RED ", f)
print("regen_check:", "GREEN" if not fails else f"{len(fails)} red")
sys.exit(1 if fails else 0)
