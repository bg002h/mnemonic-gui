"""T5 prototype for the pure planner: apply each mutation to a scratch copy of plan.py and show
that test_plan.py fails or the regenerated §A5 differs. Run: python3 mutations.py"""
import os, shutil, subprocess, sys, tempfile
HERE = os.path.dirname(os.path.abspath(__file__))
M = [
    ("C1: `-` not refused", 'if v == "-":', 'if False:'),
    ("C1: reserved name allowed", 'if name.startswith(POLICY["reserved_env_prefix"]):', 'if False:'),
    ("C1: bad name allowed", 'if not re.fullmatch(POLICY["env_name_rule"], name):', 'if False:'),
    ("C1: unset allowed", 'if name not in user_env:', 'if False:'),
    ("C1: empty allowed", 'if user_env[name] == "":', 'if False:'),
    ("NI1: env_value_rule ignored", 'got.append(env_value_rule(user_env[name], rule))', 'got.append(user_env[name].rstrip("\\n"))'),
    ("Nm1: pass-through guard off", 'if "@env:" + POLICY["reserved_env_prefix"] in t:', 'if False:'),
    ("NI2: interim path off", 'if platform not in POLICY["private_channels_on"]:', 'if False:'),
    ("NI1: lenient filter off", 'if not all(is_clean(v) for v in vals):', 'if False:'),
    ("M5: step 1 removed", 'if len(forced) > 1:', 'forced = []\n    if False:'),
    ("M4: payload bound off", 'if len(payload.encode()) > POLICY["pipe_payload_max"]:', 'if False:'),
    ("no-table-entry off", 'if s["key"] not in table:', 'if False:'),
]
base_a5 = open(os.path.join(HERE, "a5.md")).read()
bad = 0
for name, old, new in M:
    d = tempfile.mkdtemp()
    try:
        for f in os.listdir(HERE):
            if f.endswith((".py", ".json")) or f == "fixtures":
                src = os.path.join(HERE, f)
                (shutil.copytree if os.path.isdir(src) else shutil.copy)(src, os.path.join(d, f))
        p = os.path.join(d, "plan.py")
        s = open(p).read()
        assert s.count(old) == 1, f"{name}: anchor not found exactly once"
        open(p, "w").write(s.replace(old, new))
        t = subprocess.run([sys.executable, "test_plan.py"], cwd=d, capture_output=True, text=True)
        g = subprocess.run([sys.executable, "gen_plans.py"], cwd=d, capture_output=True, text=True)
        a5_changed = g.returncode != 0 or open(os.path.join(d, "a5.md")).read() != base_a5 if os.path.exists(os.path.join(d, "a5.md")) else True
        red = t.returncode != 0 or a5_changed
        bad += not red
        print(f"{'RED ' if red else 'GREEN (mutation survived)'}  {name}: test_plan {'fail' if t.returncode else 'pass'}, A5 {'changed' if a5_changed else 'same'}")
    finally:
        shutil.rmtree(d, ignore_errors=True)
print(f"{len(M) - bad}/{len(M)} mutations killed")
sys.exit(1 if bad else 0)
