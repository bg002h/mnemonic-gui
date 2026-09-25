"""MEASURED half of the plan table (the pure half is gen_plans.py). For every shape in shapes.py,
on Linux, through a REAL runner (env, stdin, pipe fds written and closed before spawn):

  T1  (structural)  no secret byte in argv; each binding sits at its own source's flag;
                    env names unique; at most one stdin.
  T3' (baseline)    planned exit + stdout == argv + --allow-argv-secret baseline.
  swap (mutation)   every source pair's values exchanged — what a runner that wires bindings to
                    the wrong source would send. Must differ from the baseline, except the
                    shape's listed symmetric pairs.
  NI1 (bytes)       every source typed as `@env:USER_SECRET`, its variable set to the fixture
                    value + each of ENDINGS (trailing \\n, \\r\\n, interior newline, trailing
                    spaces), resolved by plan.py and run == (a) argv carrying the exact target
                    bytes and (b) where the cell has a measured-OK EnvRef, the CLI's OWN
                    `@env:USER_SECRET` spelling. Effect lines (fingerprint/address/…) recorded.
  interim           macOS/Windows (not in private_channels_on) run resolved bytes on argv: that
                    IS the baseline invocation, so T3' for the interim path is the baseline
                    itself; the C1 legs for it are in test_plan.py (pure).

Run:  BIN_DIR=<dir with mnemonic/md/ms/mk> python3 run_plans.py   -> plans.json, t3.md"""
import json, os, subprocess, sys
from concurrent.futures import ThreadPoolExecutor
from plan import plan, Refusal, ENV_PREFIX, POLICY
from shapes import SHAPES, effect, PW
import gen_plans

B = os.environ["BIN_DIR"].rstrip("/") + "/"
TABLE = gen_plans.TABLE
ENDINGS = ["", "\n", "\r\n", "\nX", "  "]


def clean_env(extra):
    env = {k: v for k, v in os.environ.items() if not k.startswith(POLICY["reserved_env_prefix"])}
    env.update(extra)
    return env


def baseline_argv(sh, values, env_spell=None):
    """argv + --allow-argv-secret with each value verbatim. env_spell={i: NAME} spells source i
    as the CLI's own `@env:NAME` instead (NI1 comparison)."""
    argv = [B + sh["cli"]] + sh["sub"] + ["--allow-argv-secret"] + sh["pre"]
    posit = []
    for i, (s, v) in enumerate(zip(sh["sources"], values)):
        if env_spell and i in env_spell:
            v = "@env:" + env_spell[i]
        if s["form"] == "node":
            argv += [s["flag"], s["prefix"] + v]
        elif s["form"] == "value":
            argv += [s["flag"], v]
        elif s["form"] == "pos":
            posit.append(v)
        else:
            posit += v
    argv += sh["post"]
    if posit:
        argv += ["--"] + posit
    return argv


def build(sh, bindings, values):
    """Turn a plan into an invocation. Non-group payload = value + the channel's measured
    terminator (DESIGN §A3c). Pipe fds are fully written and closed BEFORE spawn (§A6)."""
    argv = [B + sh["cli"]] + sh["sub"] + sh["pre"]
    posit, env, stdin, fds, index = [], {}, None, [], {}
    for i, (s, b, v) in enumerate(zip(sh["sources"], bindings, values)):
        k = b["kind"]
        data = "\n".join(v) if s["form"] == "group" else v + b["terminator"]

        def pipe(payload):
            r, w = os.pipe()
            os.write(w, payload.encode())
            os.close(w)
            fds.append(r)
            return f"/dev/fd/{r}"
        if k == "EnvRef":
            env[b["env"]] = data
            ref = "@env:" + b["env"]
            if s["form"] == "node":
                argv += [s["flag"], s["prefix"] + ref]; index[i] = len(argv) - 1
            elif s["form"] == "value":
                argv += [s["flag"], ref]; index[i] = len(argv) - 1
            else:
                posit.append(ref)
        elif k == "DashValue":
            stdin = data
            argv += [s["flag"], (s["prefix"] if s["form"] == "node" else "") + "-"]; index[i] = len(argv) - 1
        elif k == "StdinToggle":
            stdin = data
            argv += [b["flag"]]; index[i] = len(argv) - 1
        elif k in ("PosDash", "StdinMulti"):
            stdin = data
            posit.append("-")
        elif k == "FileFlag":
            argv += [b["flag"], pipe(data)]; index[i] = len(argv) - 1
        elif k == "InFile":
            argv += ["--in", pipe(data + ("\n" if s["form"] == "group" else ""))]; index[i] = len(argv) - 1
    argv += sh["post"]
    if posit:
        argv += ["--"] + posit
    return argv, env, stdin, fds, index


def run(argv, env=None, stdin=None, fds=()):
    r = subprocess.run(argv, input=(stdin if stdin is not None else "").encode(), capture_output=True,
                       env=clean_env(env or {}), pass_fds=tuple(fds), timeout=180)
    for f in fds:
        os.close(f)
    r.stdout = r.stdout.decode("utf-8", "surrogateescape")
    r.stderr = r.stderr.decode("utf-8", "surrogateescape")
    return r


def normalise(sh, r, pw=PW):
    """slip39 split is randomised: compare what the shares recover, not the shares."""
    if not sh["name"].startswith("slip39 split") or r.returncode != 0:
        return r.stdout
    shares = [l for l in r.stdout.splitlines() if l.strip() and not l.startswith("#")][:2]
    out = subprocess.run([B + "mnemonic", "slip39", "combine", "--allow-argv-secret", "--passphrase", pw]
                         + sum([["--share", x] for x in shares], []), capture_output=True, text=True)
    return out.stdout


def same(sh, a, b, pw=PW):
    return a.returncode == b.returncode and normalise(sh, a, pw) == normalise(sh, b, pw)


def t1(sh, bindings, argv, index, values):
    problems = []
    secrets = []
    for v in values:
        secrets += v if isinstance(v, list) else [v]
    if any(sec in tok for sec in secrets for tok in argv):
        problems.append("secret byte in argv")
    for i, b in enumerate(bindings):
        s = sh["sources"][i]
        if i in index and s["flag"] and b["kind"] not in ("StdinToggle", "FileFlag", "InFile"):
            if argv[index[i] - 1] != s["flag"]:
                problems.append(f"source {i} not at its flag")
    envs = [b["env"] for b in bindings if "env" in b]
    if len(envs) != len(set(envs)):
        problems.append("env name reused")
    if len([b for b in bindings if b["kind"] in ("StdinToggle", "DashValue", "PosDash", "StdinMulti")]) > 1:
        problems.append("two stdin bindings")
    return problems


def measure(pair):
    sh, pure = pair
    row = {"name": sh["name"], "symmetric": sh["symmetric"], "expect": sh["expect"], "plans": pure["plans"]}
    lp = pure["plans"]["linux"]
    if "bindings" not in lp:
        return row
    values = [s["value"] for s in sh["sources"]]
    base = run(baseline_argv(sh, values))
    argv, env, stdin, fds, index = build(sh, lp["bindings"], values)
    row["t1_problems"] = t1(sh, lp["bindings"], argv, index, values)
    got = run(argv, env, stdin, fds)
    row["base_exit"], row["plan_exit"] = base.returncode, got.returncode
    row["base_effect"] = effect(normalise(sh, base))
    row["equal"] = same(sh, base, got)
    row["plan_err"] = "" if row["equal"] else (got.stderr.strip().splitlines() or [""])[-1][:160]
    row["swaps"] = []
    for i in range(len(values)):
        for j in range(i + 1, len(values)):
            sw_vals = list(values)
            sw_vals[i], sw_vals[j] = values[j], values[i]
            a2, e2, s2, f2, _ = build(sh, lp["bindings"], sw_vals)
            sw = run(a2, e2, s2, f2)
            row["swaps"].append({"pair": [i, j], "same_as_baseline": same(sh, sw, base),
                                 "expected_same": (i, j) in sh["symmetric"], "exit": sw.returncode})
    # NI1: typed as @env:USER_SECRET with each ending; resolved by plan.py.
    row["env_endings"] = []
    for i, s in enumerate(sh["sources"]):
        if s["form"] == "group":
            continue
        env_ok = any(c["kind"] == "EnvRef" for c in TABLE[s["key"]]["channels"])
        for e in ENDINGS:
            raw = s["value"] + e
            srcs = [dict(x) for x in sh["sources"]]
            srcs[i]["value"] = "@env:USER_SECRET"
            try:
                b2, prov, res = plan(srcs, TABLE, "linux", {"USER_SECRET": raw})
            except Refusal as ex:
                row["env_endings"].append({"source": i, "ending": e, "refused": ex.code})
                continue
            vals = [x["value"] for x in res]
            pw = vals[i] if sh["name"].startswith("slip39 split") and i == 1 else PW
            a3, e3, s3, f3, _ = build(sh, b2, vals)
            g = run(a3, e3, s3, f3)
            exact = run(baseline_argv(sh, vals))
            rec = {"source": i, "ending": e, "channel": b2[i]["kind"], "provenance": prov[i],
                   "exit": g.returncode, "effect": effect(normalise(sh, g, pw)),
                   "eq_argv_exact": same(sh, g, exact, pw)}
            if env_ok:
                cli = subprocess.run(baseline_argv(sh, values, {i: "USER_SECRET"}), capture_output=True,
                                     env=clean_env({"USER_SECRET": raw}), timeout=180)
                cli.stdout = cli.stdout.decode("utf-8", "surrogateescape")
                rec["eq_cli_env"] = same(sh, g, cli, pw)
                rec["cli_env_effect"] = effect(normalise(sh, cli, pw))
            row["env_endings"].append(rec)
    return row


if __name__ == "__main__":
    pure = gen_plans.generate()
    with ThreadPoolExecutor(12) as ex:
        rows = list(ex.map(measure, zip(SHAPES, pure)))
    json.dump(rows, open("plans.json", "w"), indent=1)
    lines = ["| shape | baseline exit | planned exit | planned == baseline | effect (baseline) | "
             "source values swapped (i↔j) vs baseline | T1 | NI1: `@env:` + endings == argv-exact / == CLI's own `@env:` |",
             "|---|---|---|---|---|---|---|---|"]
    for r in rows:
        if "equal" not in r:
            lines.append(f"| {r['name']} | — | — | refused (expected: {r['expect']}) | — | — | — | — |")
            continue
        swc = "; ".join(f"{a}↔{b}: " + ("**same** (symmetric)" if x["same_as_baseline"] else f"differs (exit {x['exit']})")
                        for x in r["swaps"] for a, b in [x["pair"]]) or "n/a (one source)"
        ee = [x for x in r["env_endings"] if "refused" not in x]
        cl = [x for x in ee if "eq_cli_env" in x]
        ni1 = f"{sum(x['eq_argv_exact'] for x in ee)}/{len(ee)}; {sum(x['eq_cli_env'] for x in cl)}/{len(cl)}"
        lines.append(f"| {r['name']} | {r['base_exit']} | {r['plan_exit']} | {'**yes**' if r['equal'] else 'NO: ' + r['plan_err']} | "
                     f"`{r['base_effect']}` | {swc} | {'ok' if not r['t1_problems'] else r['t1_problems']} | {ni1} |")
    open("t3.md", "w").write("\n".join(lines) + "\n")
    print("\n".join(lines))
    bad = [r["name"] for r in rows if (r["expect"] == "run" and not r.get("equal"))
           or (r["expect"] == "refuse" and "bindings" in r["plans"]["linux"])
           or r.get("t1_problems")
           or any(x["same_as_baseline"] != x["expected_same"] for x in r.get("swaps", []))
           or any(not x.get("eq_argv_exact", True) or not x.get("eq_cli_env", True) for x in r.get("env_endings", []))]
    refused = [(r["name"], x) for r in rows for x in r.get("env_endings", []) if "refused" in x]
    print("NI1 endings refused by the planner:", refused if refused else "none")
    print("FAILURES:", bad if bad else "none")
    sys.exit(1 if bad else 0)
