"""Pure tests of plan.py (no binaries): the refusal legs R1 Nm2 asked for, the per-OS switch
(NI2), the pass-through guard (Nm1), the permutation leg (M5), the lenient-channel rule (NI1),
payload-too-large, and the OS gate on the CI workflows. Run: python3 test_plan.py"""
import copy, glob, itertools, json, os, re, sys
import plan as P
from shapes import SHAPES

HERE = os.path.dirname(os.path.abspath(__file__))
TABLE = json.load(open(os.path.join(HERE, "channel_table.json")))
PLATFORMS = ("linux", "macos", "windows")
FAIL = []


def check(name, cond):
    if not cond:
        FAIL.append(name)


def refusal(fn):
    try:
        fn()
        return None
    except P.Refusal as e:
        return e.code


def with_value(sh, i, v):
    srcs = copy.deepcopy(sh["sources"])
    srcs[i]["value"] = [v] + srcs[i]["value"][1:] if srcs[i]["form"] == "group" else v
    return srcs


# C1 legs, every source of every shape, on EVERY platform (NI2: the interim path resolves too).
C1 = [("-", {}, "C1-dash"), ("@env:MNEMONIC_GUI_S0", {"MNEMONIC_GUI_S0": "x"}, "C1-reserved-name"),
      ("@env:MNEMONIC_GUI_OTHER", {"MNEMONIC_GUI_OTHER": "x"}, "C1-reserved-name"),
      ("@env:lower_case", {"lower_case": "x"}, "C1-bad-name"), ("@env:9LEAD", {"9LEAD": "x"}, "C1-bad-name"),
      ("@env:NOT_SET_ANYWHERE", {}, "C1-env-unset"), ("@env:EMPTY", {"EMPTY": ""}, "C1-env-empty")]
n = 0
for sh in SHAPES:
    for i, _ in enumerate(sh["sources"]):
        for spelling, uenv, code in C1:
            for p in PLATFORMS:
                n += 1
                got = refusal(lambda: P.plan(with_value(sh, i, spelling), TABLE, p, uenv))
                check(f"C1 {sh['name']} src{i} {spelling!r} {p}: got {got}, want {code}", got == code)
print(f"C1 refusal legs: {n}")

# Resolution: the target is env_value_rule(raw); provenance names the variable; both rules.
src = [{"key": "mnemonic restore --passphrase", "form": "value", "flag": "--passphrase", "prefix": "", "value": "@env:MY_PW"}]
for raw, rule, want in [("pw\n", "verbatim", "pw\n"), ("pw\r\n", "verbatim", "pw\r\n"), ("pw\n", "strip-one-trailing-newline", "pw"),
                        ("pw\r\n", "strip-one-trailing-newline", "pw"), ("pw\n\n", "strip-one-trailing-newline", "pw\n"),
                        ("a\nb", "strip-one-trailing-newline", "a\nb"), ("pw  ", "verbatim", "pw  ")]:
    res, prov = P.resolve(src, {"MY_PW": raw}, rule)
    check(f"resolve {raw!r} {rule}", res[0]["value"] == want and prov[0] == "$MY_PW")

# Nm1: the reserved prefix is refused in pass-through (non-secret) fields too.
check("guard refuses", refusal(lambda: P.guard_passthrough(["xpub=@env:MNEMONIC_GUI_S0"])) == "C1-reserved-name")
check("guard passes", refusal(lambda: P.guard_passthrough(["xpub=@env:MY_XPUB", "bip84"])) is None)

# no-table-entry refuses on every platform (no unmeasured source reaches argv on any OS).
fake = [{"key": "mnemonic verify-bundle --from phrase=", "form": "node", "flag": "--from", "prefix": "phrase=", "value": "x y"}]
for p in PLATFORMS:
    check(f"no-table-entry {p}", refusal(lambda: P.plan(fake, TABLE, p)) == "no-table-entry")

# NI2: an OS outside private_channels_on gets the interim plan (Argv) and nothing else.
for sh in SHAPES:
    for p in PLATFORMS:
        try:
            b, _, _ = P.plan(sh["sources"], TABLE, p)
        except P.Refusal:
            continue
        interim = p not in P.POLICY["private_channels_on"]
        check(f"interim {sh['name']} {p}", all(x["kind"] == "Argv" for x in b) == interim)

# M5: permuting the sources never changes plan-vs-refuse (Linux, and macOS once enabled).
saved = list(P.POLICY["private_channels_on"])
for plat, enabled in [("linux", saved), ("macos", list(PLATFORMS))]:
    P.POLICY["private_channels_on"][:] = enabled
    for sh in SHAPES:
        ok0 = refusal(lambda: P.plan(sh["sources"], TABLE, plat)) is None
        for perm in itertools.permutations(sh["sources"]):
            okp = refusal(lambda: P.plan(list(perm), TABLE, plat)) is None
            check(f"M5 {sh['name']} {plat}", okp == ok0)
P.POLICY["private_channels_on"][:] = saved

# NI1 lenient rule: a non-clean value on an input whose every channel is lenient refuses.
ms1 = [{"key": "mnemonic xpub-search path-of-xpub --ms1", "form": "value", "flag": "--ms1", "prefix": "", "value": "ms10abc\n"}]
check("lenient refuses non-clean", refusal(lambda: P.plan(ms1, TABLE, "linux")) == "value-not-byte-exact")
ms1[0]["value"] = "ms10abc"
check("lenient accepts clean", refusal(lambda: P.plan(ms1, TABLE, "linux")) is None)

# payload-too-large on the fd channel.
big = [{"key": "ms verify --phrase", "form": "value", "flag": "--phrase", "prefix": "", "value": "a b"},
       {"key": "ms verify <ms1>", "form": "pos", "flag": None, "prefix": "", "value": "m" * 5000}]
check("payload-too-large", refusal(lambda: P.plan(big, TABLE, "linux")) == "payload-too-large")

# OS gate (NI2): an OS may be in private_channels_on only if a CI job on that OS runs the
# real-binary tests (a job block with `runs-on:` naming the OS and MNEMONIC_BIN set).
RUNNER = {"linux": "ubuntu", "macos": "macos", "windows": "windows"}
wf = glob.glob(os.path.join(HERE, "..", "..", "..", ".github", "workflows", "*.yml"))
jobs = []
for f in wf:
    text = open(f).read()
    body = text.split("\njobs:", 1)[-1]
    for block in re.split(r"\n  (?=[A-Za-z0-9_-]+:\n)", body):
        m = re.search(r"runs-on:\s*(\S+)", block)
        if m:
            jobs.append((m.group(1), "MNEMONIC_BIN" in block))
check("os gate: workflows found", bool(jobs))
for osname in P.POLICY["private_channels_on"]:
    check(f"os gate: {osname} has a real-binary CI job", any(RUNNER[osname] in r and has for r, has in jobs))

# Copy gate (R1 Nm7, DESIGN §A7): Copy spells a $VAR-provenance binding as the CLI's own
# `@env:VAR` where the input has a measured-OK EnvRef, else as `printf '%s<terminator>' "$VAR" |`,
# which reproduces the target only while env_value_rule is verbatim. A rule change therefore
# requires every stdin-only secret input to have an EnvRef cell, or a new Copy rule.
if P.POLICY["env_value_rule"] != "verbatim":
    for k, v in TABLE.items():
        check(f"copy gate: {k} needs an EnvRef cell under {P.POLICY['env_value_rule']}",
              any(c["kind"] == "EnvRef" for c in v["channels"]))

print(f"{len(FAIL)} failures")
for f in FAIL[:20]:
    print("  FAIL", f)
sys.exit(1 if FAIL else 0)
