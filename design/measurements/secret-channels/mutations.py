"""T5 prototype for the planner (R3 NI6: a crash is never a kill; a no-op control must survive).

Each mutation is applied to plan.py inside a scratch copy that keeps the REPO LAYOUT the tests
need (pinned-upstream.toml, .github/workflows, the design doc) — fold 3 copied only this folder,
so test_plan.py crashed on the missing pin file and every mutation, a no-op included, "died".

A mutation counts as KILLED only when a test FAILS BY ASSERTION:
  test_plan.py exits 1 AND prints "<N> failures" with N > 0 AND no Python traceback, or
  the regenerated §A5 differs (gen_plans.py exited 0), or
  (with BIN_DIR, for mutations marked real) run_plans.py exits 1 printing "FAILURES: [...]" and no traceback.
A crash is reported as CRASH and counts as SURVIVED. The CONTROL (a no-op edit) must survive;
if it does not, the harness is broken and this script fails.
Run: python3 mutations.py   (BIN_DIR set: also the real-runner legs)"""
import os, re, shutil, subprocess, sys, tempfile
HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
REL = os.path.relpath(HERE, REPO)
M = [
    ("CONTROL: no-op (must SURVIVE)", '"""Reference model of the design', '"""Reference model  of the design', False, True),
    ("C1: `-` not refused", 'if v == "-":', 'if False:', False, False),
    ("C1: reserved name allowed", 'if name.startswith(POLICY["reserved_env_prefix"]):', 'if False:', False, False),
    ("C1: bad name allowed", 'if not re.fullmatch(POLICY["env_name_rule"], name):', 'if False:', False, False),
    ("C1: unset allowed", 'if name not in user_env:', 'if False:', False, False),
    ("C1: empty allowed", 'if target == "":', 'if False:', False, False),
    ("Nit: NUL allowed", 'if any("\\0" in v for v in vals):      # R2 Nit 1', 'if False:      # R2 Nit 1', False, False),
    ("NC1: interim re-read refusal off",
     'if ("-" in row["spellings"] and v == "-") or ("@env:" in row["spellings"] and v.startswith("@env:")):', 'if False:', True, False),
    ("NI3: interim sends the TYPED text (R2's mutation)",
     'for i, s in enumerate(res)], prov, res', 'for i, s in enumerate(res)], prov, sources', True, False),
    ("NI7: per-input rule ignored (always verbatim)", 'target = env_value_rule(user_env[name], r_)', 'target = env_value_rule(user_env[name], "verbatim")', False, False),
    ("Nm13: leading-dash refusal off",
     'if dash and not table[s["key"]].get("argv_eq_exact"):', 'if False:', True, False),
    ("Nm1: pass-through guard off", 'if "@env:" + POLICY["reserved_env_prefix"] in t:', 'if False:', False, False),
    ("NI2: interim path off", 'if platform not in POLICY["private_channels_on"]:', 'if False:', False, False),
    ("NI1: lenient filter off", 'if not all(is_clean(v) for v in vals):', 'if False:', False, False),
    ("M5: step 1 removed", 'if len(forced) > 1:', 'forced = []\n    if False:', False, False),
    ("M4: payload bound off", 'if len(payload.encode()) > POLICY["pipe_payload_max"]:', 'if False:', False, False),
    ("no-table-entry off", 'if s["key"] not in table:', 'if False:', False, False),
]


def scratch_copy():
    d = tempfile.mkdtemp()
    shutil.copy(os.path.join(REPO, "pinned-upstream.toml"), d)
    shutil.copytree(os.path.join(REPO, ".github"), os.path.join(d, ".github"))
    os.makedirs(os.path.join(d, "design"))
    shutil.copy(os.path.join(REPO, "design", "DESIGN_secret_channels_and_new_forms.md"), os.path.join(d, "design"))
    shutil.copytree(HERE, os.path.join(d, REL), ignore=shutil.ignore_patterns("__pycache__"))
    return d, os.path.join(d, REL)


def assertion_fail_test_plan(p):
    m = re.search(r"^(\d+) failures$", p.stdout, re.M)
    return p.returncode == 1 and m and int(m.group(1)) > 0 and "Traceback" not in p.stderr + p.stdout


def assertion_fail_run_plans(p):
    return p.returncode == 1 and re.search(r"^FAILURES: \[", p.stdout, re.M) and "Traceback" not in p.stderr + p.stdout


base_a5 = open(os.path.join(HERE, "a5.md")).read()
killed_real, broken = 0, False
for name, old, new, real, control in M:
    root, d = scratch_copy()
    try:
        pth = os.path.join(d, "plan.py")
        src = open(pth).read()
        assert src.count(old) == 1, f"{name}: anchor not found exactly once"
        open(pth, "w").write(src.replace(old, new))
        t = subprocess.run([sys.executable, "test_plan.py"], cwd=d, capture_output=True, text=True)
        g = subprocess.run([sys.executable, "gen_plans.py"], cwd=d, capture_output=True, text=True)
        a5_changed = g.returncode == 0 and open(os.path.join(d, "a5.md")).read() != base_a5
        crashed = "Traceback" in t.stderr + t.stdout or g.returncode != 0
        rp_kill = False
        if real and os.environ.get("BIN_DIR"):
            rp = subprocess.run([sys.executable, "run_plans.py"], cwd=d, capture_output=True, text=True)
            rp_kill = bool(assertion_fail_run_plans(rp))
            crashed = crashed or "Traceback" in rp.stderr + rp.stdout
        killed = bool(assertion_fail_test_plan(t)) or a5_changed or rp_kill
        how = ", ".join(x for x, c in [("test_plan assertion", assertion_fail_test_plan(t)), ("A5 changed", a5_changed),
                                        ("run_plans assertion", rp_kill), ("CRASH", crashed)] if c) or "nothing failed"
        if control:
            broken = killed or crashed
            print(f"{'OK  ' if not broken else 'BROKEN HARNESS'}  {name}: {how}")
        else:
            killed_real += killed and not crashed
            print(f"{'KILLED  ' if killed and not crashed else 'SURVIVED'}  {name}: {how}")
    finally:
        shutil.rmtree(root, ignore_errors=True)
n = sum(1 for m in M if not m[4])
print(f"{killed_real}/{n} mutations killed by assertion; control {'survived' if not broken else 'DID NOT SURVIVE'}")
sys.exit(1 if broken or killed_real != n else 0)
