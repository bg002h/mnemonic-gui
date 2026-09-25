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

# T10 OS gate (R2 Nm8), hardened: parsed YAML, no `if:` jobs, matrix-aware, must run the named
# real-binary targets with MNEMONIC_BIN in env. Pinned against decoys and genuine fixtures.
import os_gate
TARGETS = P.POLICY["real_binary_test_targets"]
CI = os.path.join(HERE, "fixtures", "ci")
for f, want in [("decoy_comment", {}), ("decoy_if_false", {}), ("decoy_missing_target", {}),
                ("genuine_matrix", {"linux", "macos", "windows"}), ("genuine_linux", {"linux"})]:
    for o in PLATFORMS:
        check(f"os gate fixture {f} {o}", os_gate.gate([os.path.join(CI, f + ".yml")], o, TARGETS) == (o in want))
# The repo's own workflows: the named targets do not exist until the implementing change adds
# them, so this is REPORTED here; the Rust T10 makes it a hard failure from that change on.
REPO_GATE = {o: os_gate.gate(os_gate.repo_workflows(HERE), o, TARGETS) for o in P.POLICY["private_channels_on"]}
print("os gate on this repo (pending until the implementing change):", REPO_GATE)

# NI3 (R2): the INTERIM path must carry the RESOLVED target, never the typed text — pure leg,
# every shape, every source, macOS and Windows, both rules.
for sh in SHAPES:
    for i, s in enumerate(sh["sources"]):
        if s["form"] == "group":
            continue
        for rule in ("verbatim", "strip-one-trailing-newline"):
            raw = s["value"] + "\n"
            for plat in ("macos", "windows"):
                try:
                    b, prov, res = P.plan(with_value(sh, i, "@env:USER_SECRET"), TABLE, plat, {"USER_SECRET": raw}, rule)
                except P.Refusal as e:
                    check(f"NI3 {sh['name']} src{i} {plat} {rule}: refused {e.code}", False)
                    continue
                check(f"NI3 {sh['name']} src{i} {plat} {rule}: interim value",
                      res[i]["value"] == P.env_value_rule(raw, rule) and b[i]["kind"] == "Argv" and prov[i] == "$USER_SECRET")

# NC1 (R2): a resolved value the CLI would re-interpret on argv is refused on the interim path,
# per the measured argv_reinterprets row; the private path (Linux) carries it as bytes.
nc1 = 0
for sh in SHAPES:
    for i, s in enumerate(sh["sources"]):
        if s["form"] == "group":
            continue
        cli = s["key"].split()[0]
        sp = P.POLICY["argv_reinterprets"].get(cli, {"spellings": []})["spellings"]
        for content, spelling in (("@env:OTHER", "@env:"), ("-", "-")):
            uenv = {"USER_SECRET": content, "OTHER": "hunter2"}
            srcs = with_value(sh, i, "@env:USER_SECRET")
            for plat in ("macos", "windows"):
                nc1 += 1
                got = refusal(lambda: P.plan(srcs, TABLE, plat, uenv))
                want = "value-is-a-channel-spelling" if spelling in sp else None
                check(f"NC1 {sh['name']} src{i} {content!r} {plat}: got {got}, want {want}", got == want)
print(f"NC1 interim legs: {nc1}")

# R2 Nit 1: C1-env-empty is judged on the TARGET; NUL refuses on every OS.
check("env-empty after rule", refusal(lambda: P.resolve(src, {"MY_PW": "\n"}, "strip-one-trailing-newline")) == "C1-env-empty")
check("env not empty verbatim", refusal(lambda: P.resolve(src, {"MY_PW": "\n"}, "verbatim")) is None)
nul = [{"key": "mnemonic restore --passphrase", "form": "value", "flag": "--passphrase", "prefix": "", "value": "a\0b"}]
for p_ in PLATFORMS:
    check(f"nul-in-value {p_}", refusal(lambda: P.plan(nul, TABLE, p_)) == "nul-in-value")

# Pin check (controller): every measured artifact and every CLI-behaviour policy row must carry
# the versions the GUI pins; a pin bump without a re-measure goes red here.
import tomllib
pins = {sec: v["tag"].rsplit("-v", 1)[1]
        for sec, v in tomllib.load(open(os.path.join(HERE, "..", "..", "..", "pinned-upstream.toml"), "rb")).items()
        if isinstance(v, dict) and "tag" in v}
mw = json.load(open(os.path.join(HERE, "measured_with.json")))
check(f"pin: measured_with {mw} == pinned {pins}", mw == pins)
for cli, v in P.POLICY["env_value_rule"]["measured_with"].items():
    check(f"pin: env_value_rule measured with {cli} {v}, pinned {pins.get(cli)}", v == pins.get(cli))
RE = json.load(open(os.path.join(HERE, "reinterpret.json")))
for cli, row in P.POLICY["argv_reinterprets"].items():
    check(f"pin: argv_reinterprets {cli} {row['version']}, pinned {pins.get(cli)}", row["version"] == pins.get(cli))
    check(f"argv_reinterprets {cli} == measurement", row["spellings"] == RE[cli]["spellings"] and row["version"] == RE[cli]["version"])

# Copy gate (R1 Nm7, DESIGN §A7): Copy spells a $VAR-provenance binding as the CLI's own
# `@env:VAR` where the input has a measured-OK EnvRef, else as `printf '%s<terminator>' "$VAR" |`,
# which reproduces the target only while env_value_rule is verbatim. A rule change therefore
# requires every stdin-only secret input to have an EnvRef cell, or a new Copy rule.
if P.POLICY["env_value_rule"]["value"] != "verbatim":
    for k, v in TABLE.items():
        check(f"copy gate: {k} needs an EnvRef cell under {P.POLICY["env_value_rule"]["value"]}",
              any(c["kind"] == "EnvRef" for c in v["channels"]))

print(f"{len(FAIL)} failures")
for f in FAIL[:20]:
    print("  FAIL", f)
sys.exit(1 if FAIL else 0)
