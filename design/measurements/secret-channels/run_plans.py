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


def baseline_argv(sh, values, env_spell=None, toggle=None, eq=False):
    """argv + --allow-argv-secret with each value verbatim. env_spell={i: NAME} spells source i
    as the CLI's own `@env:NAME`; toggle={i: FLAG} spells it as its `--X-stdin` toggle (the caller
    feeds stdin); eq = a set of source indices written as `--flag=value` (the interim form for a
    leading-dash value where that form measured exact, R3 Nm13)."""
    argv = [B + sh["cli"]] + sh["sub"] + ["--allow-argv-secret"] + sh["pre"]
    posit = []
    for i, (s, v) in enumerate(zip(sh["sources"], values)):
        if env_spell and i in env_spell:
            v = "@env:" + env_spell[i]
        if toggle and i in toggle:
            argv += [toggle[i]]
        elif s["form"] == "node":
            argv += [s["flag"], s["prefix"] + v]
        elif s["form"] == "value":
            argv += [f"{s['flag']}={v}"] if (eq is True or (eq and i in eq)) else [s["flag"], v]
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


def interim_argv(sh, bindings, res):
    """The interim invocation, built from the plan: every binding is Argv; the value is the
    resolved source's (DESIGN §A6)."""
    assert all(b["kind"] == "Argv" for b in bindings)
    return baseline_argv(sh, [x["value"] for x in res], eq={i for i, b in enumerate(bindings) if b.get("argv_form") == "eq"})


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
    # R3 Nm12: the passphrase goes over --passphrase-stdin (+ \r\n, stripped once), never argv —
    # so a passphrase that IS `-` or `@env:…` is recovered as the bytes it is, pre- and post-F-687.
    out = subprocess.run([B + "mnemonic", "slip39", "combine", "--allow-argv-secret", "--passphrase-stdin"]
                         + sum([["--share", x] for x in shares], []), capture_output=True, text=True,
                         input=pw + "\r\n", timeout=180)
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
    # ── INDEPENDENT ORACLES (R3 shape change): the expected answer never comes from the policy,
    # the derived data or plan.py. For a source whose input has a working CLI `@env:` (an OK
    # EnvRef cell), the oracle is the CLI's OWN `@env:USER_SECRET` run — the definition of the
    # target. Otherwise the GUI's target is the variable's bytes as typed, and the oracle delivers
    # those KNOWN bytes: on argv (verbatim) unless they start with `-` or `@env:`, which argv would
    # re-read or mis-parse; then through a non-lenient `--X-stdin` toggle (+ \r\n, stripped once).
    # None = no oracle; the leg then asserts only "never OTHER's wallet".
    def oracle(i, raw, extra_env=None):
        s_ = sh["sources"][i]
        chans = TABLE[s_["key"]]["channels"]
        envx = dict(extra_env or {})
        if any(c["kind"] == "EnvRef" for c in chans):
            envx["USER_SECRET"] = raw
            return run(baseline_argv(sh, values, {i: "USER_SECRET"}), envx)
        if not (raw.startswith("-") or raw.startswith("@env:")):
            vv = list(values); vv[i] = raw          # known bytes, on argv (verbatim there)
            return run(baseline_argv(sh, vv), envx)
        # argv would re-read or mis-parse these bytes: use a NON-lenient stdin toggle instead
        tog = next((c for c in chans if c["kind"] == "StdinToggle" and c["terminator"] is not None), None)
        if tog and s_["form"] == "value":
            return run(baseline_argv(sh, values, toggle={i: tog["flag"]}), envx, raw + "\r\n")
        return None

    # NI3: the INTERIM path executed on Linux (forced OS); `--flag=value` only for a leading dash (Nm13).
    row["interim"] = []
    for i, s in enumerate(sh["sources"]):
        if s["form"] == "group":
            continue
        for e in ["", "\n", "\r\n", "  "]:
            raw = s["value"] + e
            srcs = [dict(x) for x in sh["sources"]]
            srcs[i]["value"] = "@env:USER_SECRET"
            try:
                b4, prov4, res4 = plan(srcs, TABLE, "macos", {"USER_SECRET": raw})
            except Refusal as ex:
                row["interim"].append({"source": i, "ending": e, "refused": ex.code})
                continue
            got4 = run(interim_argv(sh, b4, res4))
            orc = oracle(i, raw)
            pw = res4[i]["value"] if sh["name"].startswith("slip39 split") and i == 1 else PW
            row["interim"].append({"source": i, "ending": e, "equal": orc is not None and same(sh, got4, orc, pw),
                                   "no_oracle": orc is None, "effect": effect(normalise(sh, got4, pw))})
    # NC1: the variable HOLDS `@env:OTHER` (OTHER = this source's fixture value) or `-`. Both paths
    # are EXECUTED when planned; each must equal the oracle and never be OTHER's wallet. The
    # expected refusal is NOT read from the derived data (R3 NI5): a stale reinterpret.json that
    # admits a value the CLI re-reads shows up here as OTHER's wallet.
    row["nc1"] = []
    for i, s in enumerate(sh["sources"]):
        if s["form"] == "group":
            continue
        for content in ("@env:OTHER", "-"):
            uenv = {"USER_SECRET": content, "OTHER": s["value"]}
            srcs = [dict(x) for x in sh["sources"]]
            srcs[i]["value"] = "@env:USER_SECRET"
            rec = {"source": i, "content": content}
            pw5 = content if sh["name"].startswith("slip39 split") and i == 1 else PW
            orc = oracle(i, content, {"OTHER": s["value"]})
            for path in ("linux", "macos"):
                try:
                    b5, _, res5 = plan(srcs, TABLE, path, uenv)
                except Refusal as ex:
                    rec[path] = ex.code
                    continue
                if path == "linux":
                    a5_, e5, s5, f5, _ = build(sh, b5, [x["value"] for x in res5])
                    e5 = dict(e5); e5["OTHER"] = s["value"]; e5["USER_SECRET"] = content   # child inherits the GUI env
                    g5 = run(a5_, e5, s5, f5)
                else:
                    g5 = run(interim_argv(sh, b5, res5), {"OTHER": s["value"], "USER_SECRET": content})
                rec[path] = "ran"
                rec[path + "_is_other_wallet"] = g5.returncode in (0, 4) and same(sh, g5, base, PW)
                if orc is not None:
                    rec[path + "_eq_oracle"] = same(sh, g5, orc, pw5)
            row["nc1"].append(rec)
    # Nm13: a TYPED value with a leading dash, value-form sources: private and interim vs oracle.
    row["dash"] = []
    for i, s in enumerate(sh["sources"]):
        if s["form"] != "value":
            continue
        for v in ("-leading", "--help", "-lead  "):
            srcs = [dict(x) for x in sh["sources"]]
            srcs[i]["value"] = v
            orc = oracle(i, v)
            rec = {"source": i, "value": v, "oracle": orc is not None}
            for path in ("linux", "macos"):
                try:
                    b6, _, res6 = plan(srcs, TABLE, path, {})
                except Refusal as ex:
                    rec[path] = ex.code
                    continue
                if path == "linux":
                    a6, e6, s6, f6, _ = build(sh, b6, [x["value"] for x in res6])
                    g6 = run(a6, e6, s6, f6)
                else:
                    g6 = run(interim_argv(sh, b6, res6))
                rec[path] = g6.returncode
                if orc is not None:
                    rec[path + "_eq_oracle"] = same(sh, g6, orc, v if sh["name"].startswith("slip39 split") and i == 1 else PW)
            row["dash"].append(rec)
    row["swaps"] = []
    for i in range(len(values)):
        for j in range(i + 1, len(values)):
            sw_vals = list(values)
            sw_vals[i], sw_vals[j] = values[j], values[i]
            a2, e2, s2, f2, _ = build(sh, lp["bindings"], sw_vals)
            sw = run(a2, e2, s2, f2)
            row["swaps"].append({"pair": [i, j], "same_as_baseline": same(sh, sw, base),
                                 "expected_same": (i, j) in sh["symmetric"], "exit": sw.returncode})
    # NI1: typed as @env:USER_SECRET with each ending; resolved by plan.py; compared with the oracle.
    row["env_endings"] = []
    for i, s in enumerate(sh["sources"]):
        if s["form"] == "group":
            continue
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
            orc = oracle(i, raw)
            row["env_endings"].append({"source": i, "ending": e, "channel": b2[i]["kind"], "provenance": prov[i],
                                       "exit": g.returncode, "effect": effect(normalise(sh, g, pw)),
                                       "eq_oracle": orc is not None and same(sh, g, orc, pw), "no_oracle": orc is None})
    return row


if __name__ == "__main__":
    pure = gen_plans.generate()
    with ThreadPoolExecutor(12) as ex:
        rows = list(ex.map(measure, zip(SHAPES, pure)))
    json.dump(rows, open("plans.json", "w"), indent=1)
    lines = ["| shape | baseline exit | planned exit | planned == baseline | effect (baseline) | "
             "source values swapped (i↔j) vs baseline | T1 | NI1: `@env:` + endings == oracle | "
             "interim (NI3) == oracle | NC1: OTHER's wallet; == oracle; refused Linux/interim | "
             "Nm13 leading dash == oracle (Linux, interim) |",
             "|---|---|---|---|---|---|---|---|---|---|---|"]
    for r in rows:
        if "equal" not in r:
            lines.append(f"| {r['name']} | — | — | refused (expected: {r['expect']}) | — | — | — | — | — | — | — |")
            continue
        swc = "; ".join(f"{a}↔{b}: " + ("**same** (symmetric)" if x["same_as_baseline"] else f"differs (exit {x['exit']})")
                        for x in r["swaps"] for a, b in [x["pair"]]) or "n/a (one source)"
        ee = [x for x in r["env_endings"] if "refused" not in x]
        im = [x for x in r["interim"] if "refused" not in x]
        nc = r["nc1"]
        other = sum(x.get("linux_is_other_wallet", False) + x.get("macos_is_other_wallet", False) for x in nc)
        eqo = sum(x.get(p + "_eq_oracle", False) for x in nc for p in ("linux", "macos"))
        ran = sum(p + "_eq_oracle" in x for x in nc for p in ("linux", "macos"))
        refl = sum(x.get("linux") not in ("ran", None) for x in nc)
        refm = sum(x.get("macos") not in ("ran", None) for x in nc)
        dl = [x for x in r["dash"] if x["oracle"]]
        def dcell(p):
            ref = sum(isinstance(x.get(p), str) for x in dl)
            return f"{sum(x.get(p + '_eq_oracle', False) for x in dl)}/{len(dl) - ref}" + (f" (+{ref} refused)" if ref else "")
        dash = f"{dcell('linux')}, {dcell('macos')}" if r["dash"] else "n/a"
        lines.append(f"| {r['name']} | {r['base_exit']} | {r['plan_exit']} | {'**yes**' if r['equal'] else 'NO: ' + r['plan_err']} | "
                     f"`{r['base_effect']}` | {swc} | {'ok' if not r['t1_problems'] else r['t1_problems']} | "
                     f"{sum(x['eq_oracle'] for x in ee)}/{len(ee)} | {sum(x['equal'] for x in im)}/{len(im)} | "
                     f"{other} OTHER; {eqo}/{ran} eq; {refl}/{refm} refused | {dash} |")
    open("t3.md", "w").write("\n".join(lines) + "\n")
    print("\n".join(lines))
    # A shape that PLANS must pass every oracle leg; a refusal is always safe. shapes.py's
    # `expect` is informational only — it is a human expectation, and the F-687 bump legitimately
    # flips `ms derive … + --passphrase` from refused to planned (R3 shape change: no hand-kept
    # expectation is a gate).
    bad = [r["name"] for r in rows if ("bindings" in r["plans"]["linux"] and not r.get("equal"))
           or r.get("t1_problems")
           or any(x["same_as_baseline"] != x["expected_same"] for x in r.get("swaps", []))
           or any(not x["eq_oracle"] and not x["no_oracle"] for x in r.get("env_endings", []) if "refused" not in x)
           or any(not x["equal"] and not x["no_oracle"] for x in r.get("interim", []) if "refused" not in x)
           or any(x.get("linux_is_other_wallet") or x.get("macos_is_other_wallet")
                  or x.get("linux_eq_oracle") is False or x.get("macos_eq_oracle") is False for x in r.get("nc1", []))
           or any(x.get("linux_eq_oracle") is False or x.get("macos_eq_oracle") is False for x in r.get("dash", []))]
    refused = [(r["name"], x) for r in rows for x in r.get("env_endings", []) if "refused" in x]
    print("NI1 endings refused by the planner:", refused if refused else "none")
    print("FAILURES:", bad if bad else "none")
    sys.exit(1 if bad else 0)
