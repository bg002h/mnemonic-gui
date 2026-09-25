"""Byte fidelity per channel cell (DESIGN §A3c, R1 NI1).

For every single-input row and every channel measured OK for it, deliver the secret with each
ENDING appended and compare (exit, stdout) against argv carrying the EXACT same bytes
(+ --allow-argv-secret; argv is verbatim). For stdin and fd channels the GUI appends a
TERMINATOR after the value; the channel's own strip then removes it. This script finds, per
cell, the first terminator in TERMINATORS for which every ending matches — that is the
`terminator` channel_table.json records (table_build.py reads bytes.json). EnvRef gets no
terminator: the CLI's `@env:` value rule (`env_value_rule`, a single table entry) defines the
target bytes, and this script checks the CLI's own `@env:` against argv-exact too.

Run: BIN_DIR=... python3 run_bytes.py   -> bytes.json, bytes.txt"""
import json, os, subprocess
from concurrent.futures import ThreadPoolExecutor
from cases3 import C, C2, C3
from table_build import kind as kind_of
from channels import ENV

ENDINGS = ["", "\n", "\r\n", "\r", "  ", " \n", "\nX"]
RULES = [("verbatim", lambda r: r),
         ("strip-one-trailing-newline", lambda r: r[:-2] if r.endswith("\r\n") else r[:-1] if r.endswith("\n") else r)]
TERMINATORS = ["\r\n", "\n", ""]
STDIN_KINDS = ("DashValue", "StdinToggle", "PosDash")
FD_KINDS = ("FileFlag", "InFile")

cases = {c["label"]: c for c in C + C2 + C3}
cases.pop("ms combine <shares> (first)", None)
rows = {}
for f in ["channels.json", "channels2.json", "channels3.json"]:
    for r in json.load(open(f)):
        rows[r["label"]] = r


def run(argv, stdin=None, env_val=None, fds=()):
    env = dict(os.environ)
    if env_val is not None:
        env[ENV] = env_val
    # bytes mode: no newline translation anywhere (the point of this script)
    p = subprocess.run(argv, input=(stdin if stdin is not None else "").encode(), capture_output=True,
                       env=env, pass_fds=tuple(fds), timeout=180)
    for f in fds:
        os.close(f)
    p.stdout = p.stdout.decode("utf-8", "surrogateescape")
    return p


def invocation(case, ch, value, term):
    argv = list(case["argv"])
    i = next(k for k, a in enumerate(argv) if "{S}" in a)
    if not case.get("multi"):
        argv = [a for a in argv if a != "--allow-argv-secret"]
        i = next(k for k, a in enumerate(argv) if "{S}" in a)
    tok, mode, k = argv[i], case["mode"], ch["kind"]
    stdin, env_val, fds = None, None, []

    def pipe(payload):
        r, w = os.pipe(); os.write(w, payload.encode()); os.close(w); fds.append(r)
        return f"/dev/fd/{r}"
    if k == "EnvRef":
        argv[i] = tok.replace("{S}", "@env:" + ENV); env_val = value
    elif k in ("DashValue", "PosDash"):
        argv[i] = tok.replace("{S}", "-"); stdin = value + term
    elif k == "StdinToggle":
        argv[i - 1:i + 1] = [ch["flag"]]; stdin = value + term
    elif k == "FileFlag":
        argv[i - 1:i + 1] = [ch["flag"], pipe(value + term)]
    elif k == "InFile":
        if mode == "pos":
            del argv[i]
            if argv[-1] == "--":
                argv.pop()
            argv += ["--in", pipe(value + term)]
        else:
            argv[i - 1:i + 1] = ["--in", pipe(value + term)]
    return argv, stdin, env_val, fds


def classify(base, got):
    """How a channel result differs from argv-exact. Only BOTH-OK-DIFFERENT is dangerous
    (a different wallet at exit 0/4); the other two fail closed on one side."""
    if got == base:
        return None
    bok, gok = base[0] in (0, 4), got[0] in (0, 4)
    if bok and gok:
        return "BOTH-OK-DIFFERENT"
    return "argv-fails/channel-ok" if gok else ("argv-ok/channel-fails" if bok else "both-fail-differently")


def checker(label, case):
    if label == "mnemonic slip39 split --passphrase":
        # the round trip must use the passphrase actually delivered, not the fixture's
        def mk(value):
            def chk(o):
                shares = [l for l in o.splitlines() if l.strip() and not l.startswith("#")][:2]
                r = subprocess.run([os.environ["BIN_DIR"].rstrip("/") + "/mnemonic", "slip39", "combine",
                                    "--allow-argv-secret", "--passphrase", value]
                                   + sum([["--share", x] for x in shares], []), capture_output=True, text=True)
                return r.stdout
            return chk
        return mk
    c = case.get("check", lambda o: o)
    return lambda value: c


def measure(label):
    case = cases[label]
    chk_for = checker(label, case)
    ok = [kind_of(n) for n, v in rows[label]["channels"].items() if v == "OK"]
    out = {"label": label, "channels": []}
    base = {}
    for e in ENDINGS:
        chk = chk_for(case["S"] + e)
        b = run([a.replace("{S}", case["S"] + e) for a in case["argv"]])
        base[e] = (b.returncode, chk(b.stdout) if b.returncode in (0, 4) else "")
    out["base_exits"] = {repr(e): base[e][0] for e in ENDINGS}
    # R3 NI7: the CLI's own `@env:` value rule, PER INPUT, derived here (never hand-kept). Where
    # the input has an OK EnvRef cell: run the CLI's `@env:VAR` with VAR = S + ending and find the
    # rule f for which it equals argv-exact(f(S + ending)) on every ending. None = the input has
    # no working CLI `@env:` (the GUI then treats the variable's bytes as typed).
    # R3 Nm13: is `--flag=VALUE` byte-identical to `--flag VALUE` on this input? (Measured: NOT on
    # ms 0.19.1 --passphrase, which trims in the `=` form.) Only value-form inputs; None otherwise.
    out["argv_eq_exact"] = None
    if case["mode"] == "value":
        i_ = next(k for k, a in enumerate(case["argv"]) if "{S}" in a)
        eq_ok = True
        for e in ENDINGS:
            v = case["S"] + e
            chk = chk_for(v)
            sep = run([a.replace("{S}", v) for a in case["argv"]])
            eqa = case["argv"][:i_ - 1] + [case["argv"][i_ - 1] + "=" + v] + case["argv"][i_ + 1:]
            eqr = run(eqa)
            if (sep.returncode, chk(sep.stdout) if sep.returncode in (0, 4) else "") != \
               (eqr.returncode, chk(eqr.stdout) if eqr.returncode in (0, 4) else ""):
                eq_ok = False
                break
        out["argv_eq_exact"] = eq_ok
    out["cli_env_rule"] = None
    if any(kind_of(n)["kind"] == "EnvRef" for n, v in rows[label]["channels"].items() if v == "OK"):
        env_ch = {"kind": "EnvRef"}
        for rname, f in RULES:
            rule_ok = True
            for e in ENDINGS:
                raw = case["S"] + e
                chk = chk_for(f(raw))
                argv, stdin, env_val, fds = invocation(case, env_ch, raw, "")
                g = run(argv, stdin, env_val, fds)
                b = run([a.replace("{S}", f(raw)) for a in case["argv"]])
                if (g.returncode, chk(g.stdout) if g.returncode in (0, 4) else "") != \
                   (b.returncode, chk(b.stdout) if b.returncode in (0, 4) else ""):
                    rule_ok = False
                    break
            if rule_ok:
                out["cli_env_rule"] = rname
                break
        else:
            out["cli_env_rule"] = "UNKNOWN"
    for ch in ok:
        # EnvRef tries "" first: today the CLI's @env: is verbatim. After F-687 a stripping @env:
        # shows up here as a non-empty terminator: a data change, not a code change.
        terms = ["", "\n", "\r\n"] if ch["kind"] == "EnvRef" else TERMINATORS
        found, detail = None, {}
        for t in terms:
            bad = []
            for e in ENDINGS:
                chk = chk_for(case["S"] + e)
                argv, stdin, env_val, fds = invocation(case, ch, case["S"] + e, t)
                g = run(argv, stdin, env_val, fds)
                got = (g.returncode, chk(g.stdout) if g.returncode in (0, 4) else "")
                how = classify(base[e], got)
                if how:
                    bad.append(f"{e!r}:{how}")
            detail[repr(t)] = bad
            if not bad:
                found = t
                break
        naive = None
        if ch["kind"] != "EnvRef":
            # NI1 evidence: the same channel with NO terminator (what fold 1 specified)
            naive = []
            for e in ENDINGS:
                chk = chk_for(case["S"] + e)
                argv, stdin, env_val, fds = invocation(case, ch, case["S"] + e, "")
                g = run(argv, stdin, env_val, fds)
                how = classify(base[e], (g.returncode, chk(g.stdout) if g.returncode in (0, 4) else ""))
                if how:
                    naive.append(f"{e!r}:{how}")
        out["channels"].append({**ch, "terminator": found, "mismatches": detail, "naive_no_terminator": naive})
    return out


labels = [l for l in cases if l in rows and rows[l]["base_exit"] in (0, 4) and rows[l]["depends"]]
with ThreadPoolExecutor(20) as ex:
    res = list(ex.map(measure, labels))
json.dump(res, open("bytes.json", "w"), indent=1)
with open("bytes.txt", "w") as f:
    for r in res:
        for c in r["channels"]:
            name = c["kind"] + (f"({c['flag']})" if "flag" in c else "")
            t = "NONE WORKS" if c["terminator"] is None else repr(c["terminator"])
            best = "" if c["terminator"] is not None else "  with '\\r\\n': " + ", ".join(c["mismatches"].get("'\\r\\n'", c["mismatches"].get("''", [])))
            nv = [x for x in (c["naive_no_terminator"] or []) if "BOTH-OK-DIFFERENT" in x]
            f.write(f"{r['label']:55s} {name:40s} terminator={t}{best}"
                    + (f"  | no terminator: {len(nv)} wrong-output endings" if nv else "") + "\n")
print(open("bytes.txt").read())

# Compact summary for the design (§A3c): terminators by channel kind, the lenient cells, and the
# cells where fold 1's "no terminator" delivery gives a different output at exit 0/4 (NI1).
from collections import Counter
cnt = Counter()
lenient, naive_wrong, dangerous = [], [], []
for r in res:
    for c in r["channels"]:
        name = c["kind"] + (f"({c['flag']})" if "flag" in c else "")
        cnt[(c["kind"], "lenient (null)" if c["terminator"] is None else repr(c["terminator"]))] += 1
        if c["terminator"] is None:
            lenient.append(f"`{r['label']}` {name}")
            # a lenient cell must never be BOTH-OK-DIFFERENT with the chosen fallback ('\r\n')
            dangerous += [f"{r['label']} {name} {m}" for m in c["mismatches"].get("'\\r\\n'", []) if "BOTH-OK-DIFFERENT" in m]
        if any("BOTH-OK-DIFFERENT" in m for m in (c["naive_no_terminator"] or [])):
            naive_wrong.append(f"`{r['label']}` {name}")
with open("bytes.md", "w") as f:
    f.write(f"Endings: {', '.join(repr(e) for e in ENDINGS)}; {sum(cnt.values())} channel cells.\n\n")
    f.write("| channel kind | measured terminator | cells |\n|---|---|---|\n")
    for (k, t), n in sorted(cnt.items()):
        f.write(f"| {k} | {t} | {n} |\n")
    f.write(f"\nLenient cells (terminator null; every mismatch is argv-fails/channel-ok, "
            f"{len(dangerous)} are both-ok-different): " + "; ".join(lenient) + ".\n")
    eqc = Counter(str(r.get("argv_eq_exact")) for r in res)
    f.write("\n`--flag=VALUE` byte-identical to `--flag VALUE` (R3 Nm13; None = not a value-form input): "
            + ", ".join(f"{k} {n}" for k, n in sorted(eqc.items())) + "; not exact: "
            + "; ".join(f"`{r['label']}`" for r in res if r.get("argv_eq_exact") is False) + ".\n")
    rc = Counter(str(r.get("cli_env_rule")) for r in res)
    f.write("\nPer-input CLI `@env:` value rule (R3 NI7; None = no working CLI `@env:`, the GUI treats the bytes as typed): "
            + ", ".join(f"{k} {n}" for k, n in sorted(rc.items())) + ".\n")
    f.write(f"\nWith NO terminator (fold 1's delivery), a wrong output at exit 0/4 on {len(naive_wrong)} cells: "
            + "; ".join(naive_wrong) + ".\n")
