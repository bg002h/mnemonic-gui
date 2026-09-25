"""Generate the design's A5 plan table from plan.py (the rule) + channel_table.json (the
measurements), and MEASURE every multi-secret plan the rule produces on Linux:

  T1  (structural)  no secret byte in argv; each binding sits at its own source's flag;
                    env names unique; at most one stdin.
  T3' (baseline)    the planned invocation's exit code and stdout == the argv +
                    --allow-argv-secret baseline (effect line shown: fingerprint/address/…).
  swap (mutation)   the same invocation with the sources' values exchanged — what a runner
                    that wires bindings to the wrong source would send. Must differ from the
                    baseline, except on the symmetric combines, where it is output-invisible.
  C1                every source typed as `-` is refused; every source typed as `@env:VAR` (VAR
                    set to the real secret) is resolved GUI-side and its run == baseline.

Run:  BIN_DIR=<dir with mnemonic/md/ms/mk> python3 run_plans.py
Writes plans.json and plans.md."""
import json, os, subprocess, sys
from plan import plan, describe, resolve, Refusal, ENV_PREFIX
from shapes import SHAPES, effect, PW

B = os.environ["BIN_DIR"].rstrip("/") + "/"
TABLE = json.load(open(os.path.join(os.path.dirname(os.path.abspath(__file__)), "channel_table.json")))


def clean_env(extra):
    env = {k: v for k, v in os.environ.items() if not k.startswith(ENV_PREFIX)}
    env.update(extra)
    return env


def baseline_argv(sh, values):
    argv = [B + sh["cli"]] + sh["sub"] + ["--allow-argv-secret"] + sh["pre"]
    posit = []
    for s, v in zip(sh["sources"], values):
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
    """Turn a plan into an invocation. Pipe fds are fully written and their write ends closed
    BEFORE spawn (DESIGN §A6 / R0 M4)."""
    argv = [B + sh["cli"]] + sh["sub"] + sh["pre"]
    posit, env, stdin, fds, index = [], {}, None, [], {}
    for i, (s, b, v) in enumerate(zip(sh["sources"], bindings, values)):
        k = b["kind"]
        data = "\n".join(v) if s["form"] == "group" else v

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
    r = subprocess.run(argv, input=stdin if stdin is not None else "", capture_output=True, text=True,
                       env=clean_env(env or {}), pass_fds=tuple(fds), timeout=180)
    for f in fds:
        os.close(f)
    return r


def normalise(sh, r):
    """slip39 split is randomised: compare what the shares recover, not the shares."""
    if not sh["name"].startswith("slip39 split") or r.returncode != 0:
        return r.stdout
    shares = [l for l in r.stdout.splitlines() if l.strip() and not l.startswith("#")][:2]
    out = subprocess.run([B + "mnemonic", "slip39", "combine", "--allow-argv-secret", "--passphrase", PW]
                         + sum([["--share", x] for x in shares], []), capture_output=True, text=True)
    return out.stdout


def t1(sh, bindings, argv, index):
    problems = []
    secrets = []
    for s in sh["sources"]:
        secrets += s["value"] if s["form"] == "group" else [s["value"]]
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
    stdins = [b for b in bindings if b["kind"] in ("StdinToggle", "DashValue", "PosDash", "StdinMulti")]
    if len(stdins) > 1:
        problems.append("two stdin bindings")
    return problems


rows = []
for sh in SHAPES:
    row = {"name": sh["name"], "symmetric": sh["symmetric"], "expect": sh["expect"], "plans": {}}
    for platform in ("linux", "macos", "windows"):
        try:
            row["plans"][platform] = {"bindings": plan(sh["sources"], TABLE, platform)}
        except Refusal as e:
            row["plans"][platform] = {"refusal": e.code, "detail": str(e)}
    # C1 leg (DESIGN §A3a): every source, typed as `-` (must refuse) and as `@env:VAR` with VAR set
    # to the real secret in the GUI's environment (must RESOLVE and, below, run equal to baseline).
    c1 = []
    for i, s in enumerate(sh["sources"]):
        srcs = [dict(x) for x in sh["sources"]]
        srcs[i]["value"] = ["-"] + s["value"][1:] if s["form"] == "group" else "-"
        try:
            resolve(srcs, {})
            c1.append(f"source {i} '-': NOT refused")
        except Refusal as e:
            if e.code != "C1-dash":
                c1.append(f"source {i} '-': refused as {e.code}")
    row["c1_failures"] = c1
    lp = row["plans"]["linux"]
    if "bindings" in lp:
        values = [s["value"] for s in sh["sources"]]
        base = run(baseline_argv(sh, values))
        argv, env, stdin, fds, index = build(sh, lp["bindings"], values)
        row["t1_problems"] = t1(sh, lp["bindings"], argv, index)
        got = run(argv, env, stdin, fds)
        row["planned_argv"] = [a.replace(B, "") for a in argv]
        row["base_exit"], row["plan_exit"] = base.returncode, got.returncode
        row["base_effect"], row["plan_effect"] = effect(normalise(sh, base)), effect(normalise(sh, got))
        row["equal"] = base.returncode == got.returncode and normalise(sh, base) == normalise(sh, got)
        row["plan_err"] = "" if row["equal"] else got.stderr.strip().splitlines()[-1][:160] if got.stderr.strip() else ""
        # C1 resolved-env runs: source i typed as `@env:USER_SECRET_i`, resolved GUI-side.
        row["c1_env_runs"] = []
        for i, s in enumerate(sh["sources"]):
            srcs = [dict(x) for x in sh["sources"]]
            uenv = {}
            if s["form"] == "group":
                uenv["USER_SECRET_0"] = s["value"][0]
                srcs[i]["value"] = ["@env:USER_SECRET_0"] + s["value"][1:]
            else:
                uenv[f"USER_SECRET_{i}"] = s["value"]
                srcs[i]["value"] = f"@env:USER_SECRET_{i}"
            res, prov = resolve(srcs, uenv)
            b2 = plan(res, TABLE, "linux")
            argv3, env3, stdin3, fds3, _ = build(sh, b2, [x["value"] for x in res])
            r3 = run(argv3, env3, stdin3, fds3)
            ok = r3.returncode == base.returncode and normalise(sh, r3) == normalise(sh, base)
            row["c1_env_runs"].append({"source": i, "provenance": prov[i], "equal_baseline": ok})
        row["swaps"] = []
        for i in range(len(values)):
            for j in range(i + 1, len(values)):
                swapped = list(values)
                swapped[i], swapped[j] = values[j], values[i]
                argv2, env2, stdin2, fds2, _ = build(sh, lp["bindings"], swapped)
                sw = run(argv2, env2, stdin2, fds2)
                same = sw.returncode == base.returncode and normalise(sh, sw) == normalise(sh, base)
                row["swaps"].append({"pair": [i, j], "same_as_baseline": same,
                                     "expected_same": (i, j) in sh["symmetric"],
                                     "exit": sw.returncode, "effect": effect(normalise(sh, sw))})
    rows.append(row)

json.dump(rows, open("plans.json", "w"), indent=1)


def cell(p):
    return describe(p["bindings"]) if "bindings" in p else f"**refuse** ({p['refusal']})"


with open("plans.md", "w") as f:
    f.write("| shape | Linux plan | macOS | Windows |\n|---|---|---|---|\n")
    for r in rows:
        mac = "same as Linux" if r["plans"]["macos"] == r["plans"]["linux"] else cell(r["plans"]["macos"])
        win = "same as Linux" if r["plans"]["windows"] == r["plans"]["linux"] else cell(r["plans"]["windows"])
        f.write(f"| {r['name']} | {cell(r['plans']['linux'])} | {mac} | {win} |\n")
    f.write("\n| shape | baseline exit | planned exit | planned == baseline | effect (baseline) | "
            "source values swapped (i↔j) vs baseline | T1 | C1: `-` refused; `@env:VAR` resolved run == baseline |\n|---|---|---|---|---|---|---|---|\n")
    for r in rows:
        if "equal" not in r:
            f.write(f"| {r['name']} | — | — | refused (expected: {r['expect']}) | — | — | — | "
                    f"{'ok' if not r['c1_failures'] else r['c1_failures']}; n/a |\n")
            continue
        swc = "; ".join(f"{a}↔{b}: " + ("**same** (symmetric)" if x["same_as_baseline"] else f"differs (exit {x['exit']})")
                        for x in r["swaps"] for a, b in [x["pair"]]) or "n/a (one source)"
        f.write(f"| {r['name']} | {r['base_exit']} | {r['plan_exit']} | {'**yes**' if r['equal'] else 'NO: ' + r['plan_err']} | "
                f"`{r['base_effect']}` | {swc} | {'ok' if not r['t1_problems'] else r['t1_problems']} | "
                f"{'ok' if not r['c1_failures'] else r['c1_failures']}; "
                f"{sum(x['equal_baseline'] for x in r['c1_env_runs'])}/{len(r['c1_env_runs'])} |\n")

bad = [r["name"] for r in rows if (r["expect"] == "run" and not r.get("equal"))
       or (r["expect"] == "refuse" and "bindings" in r["plans"]["linux"])
       or r.get("t1_problems") or r["c1_failures"]
       or any(not x["equal_baseline"] for x in r.get("c1_env_runs", []))
       or any(x["same_as_baseline"] != x["expected_same"] for x in r.get("swaps", []))]
print(open("plans.md").read())
print("FAILURES:", bad if bad else "none")
sys.exit(1 if bad else 0)
