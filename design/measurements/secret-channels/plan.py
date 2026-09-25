"""Reference model of the design's planner (DESIGN §A3a–§A4, §A6). Executable specification:
the design's A5 table is GENERATED from this file (gen_plans.py, no CLIs needed), the Rust
`form::channels::plan` must agree with it on every shape and platform (T8), and test_plan.py
pins its refusals. This is a measurement/spec tool, not GUI code.

Inputs are DATA: channel_table.json (measured channels + per-channel terminator) and
channel_policy.json (the hand-maintained single entries: env_value_rule, per-OS switches, …).

A source is one secret input the user filled:
  {"key":   channel-table key, e.g. "mnemonic restore --passphrase",
   "form":  "node" | "value" | "pos" | "group",
   "flag":  "--from" / "--passphrase" / … (None for pos/group),
   "prefix":"phrase=" / "@0.phrase=" / "" (node form only),
   "value": the text in the field (str; a list of str for a group)}
"""
import json, os, re

HERE = os.path.dirname(os.path.abspath(__file__))
POLICY = json.load(open(os.path.join(HERE, "channel_policy.json")))

STDIN_KINDS = ("StdinMulti", "StdinToggle", "DashValue", "PosDash")   # preference order
FD_KINDS = ("FileFlag", "InFile")                                       # preference order
ENV_PREFIX = POLICY["reserved_env_prefix"] + "S"                        # MNEMONIC_GUI_S<i>


class Refusal(Exception):
    def __init__(self, code, source, why):
        super().__init__(f"{code}: {source}: {why}")
        self.code, self.source, self.why = code, source, why


# ── §A3c: the TARGET bytes ────────────────────────────────────────────────────────────────
def env_value_rule(raw, rule=None):
    """What the pinned CLIs' own `@env:VAR` makes of the variable (channel_policy.json)."""
    rule = rule or POLICY["env_value_rule"]["value"]
    if rule == "verbatim":
        return raw
    if rule == "strip-one-trailing-newline":
        return raw[:-2] if raw.endswith("\r\n") else raw[:-1] if raw.endswith("\n") else raw
    raise ValueError(rule)


def is_clean(v):
    """A value a LENIENT channel (terminator null) carries exactly: no CR/LF, no edge whitespace."""
    return "\r" not in v and "\n" not in v and v == v.strip()


# ── §A3a: C1 ─────────────────────────────────────────────────────────────────────────────
def resolve(sources, user_env, rule=None):
    """C1. A secret field whose value is a channel spelling means that channel, never those
    characters:
      `@env:VAR` -> the GUI reads VAR from its OWN environment; the TARGET is
                    env_value_rule(raw) — byte-identical to what the CLI's own `@env:VAR` uses.
      `-`        -> the user's own stdin; the GUI has none to forward -> refuse.
    Runs on EVERY OS, before either Run path (private channels or the interim argv path).
    Returns (resolved sources, provenance per source)."""
    out, prov = [], []
    for s in sources:
        vals = s["value"] if s["form"] == "group" else [s["value"]]
        got, where = [], []
        for v in vals:
            if v == "-":
                raise Refusal("C1-dash", s["key"], "the GUI has no stdin to forward; type the value or use @env:VAR")
            if v.startswith("@env:"):
                name = v[len("@env:"):]
                if name.startswith(POLICY["reserved_env_prefix"]):
                    raise Refusal("C1-reserved-name", s["key"], f"{POLICY['reserved_env_prefix']}* names are the GUI's own")
                if not re.fullmatch(POLICY["env_name_rule"], name):
                    raise Refusal("C1-bad-name", s["key"], f"{name!r} is not a valid name ([A-Z_][A-Z0-9_]*)")
                if name not in user_env:
                    raise Refusal("C1-env-unset", s["key"], f"${name} is not set in the GUI's environment")
                target = env_value_rule(user_env[name], rule)
                if target == "":        # checked on the TARGET, after the rule (R2 Nit 1)
                    raise Refusal("C1-env-empty", s["key"], f"${name} is empty (after the CLI's @env: rule)")
                got.append(target); where.append(f"${name}")
            else:
                got.append(v); where.append("typed")
        r = dict(s)
        r["value"] = got if s["form"] == "group" else got[0]
        out.append(r)
        prov.append(where if s["form"] == "group" else where[0])
    return out, prov


def guard_passthrough(tokens):
    """R1 Nm1: no user-typed token in ANY field may name the GUI's reserved variables."""
    for t in tokens:
        if "@env:" + POLICY["reserved_env_prefix"] in t:
            raise Refusal("C1-reserved-name", t.split("@env:")[0] or "field", "reserved @env: name in a pass-through field")


# ── §A4.3: the rule ──────────────────────────────────────────────────────────────────────
def plan(sources, table, platform="linux", user_env=None, rule=None):
    """Resolve (C1), then plan for `platform`. Returns (bindings, provenance, resolved sources).
    On an OS outside private_channels_on this is the INTERIM path: every source, once resolved
    and measured, goes on argv with --allow-argv-secret (DESIGN §A6)."""
    res, prov = resolve(sources, user_env or {}, rule)
    for s in res:
        if s["key"] not in table:
            raise Refusal("no-table-entry", s["key"], "input not measured")
        vals = s["value"] if s["form"] == "group" else [s["value"]]
        if any("\0" in v for v in vals):      # R2 Nit 1: argv and env cannot carry NUL; one message on every OS
            raise Refusal("nul-in-value", s["key"], "the value contains a NUL byte")
    if platform not in POLICY["private_channels_on"]:
        # INTERIM path: resolved bytes on argv. A CLI re-interprets some argv VALUES as channel
        # spellings (argv_reinterprets, measured per CLI version); such a value would be resolved a
        # second time — a different wallet at exit 0 (R2 NC1). Refuse it.
        for s in res:
            cli = s["key"].split()[0]
            row = POLICY["argv_reinterprets"].get(cli, {"spellings": [], "version": "?"})
            vals = s["value"] if s["form"] == "group" else [s["value"]]
            for v in vals:
                if ("-" in row["spellings"] and v == "-") or ("@env:" in row["spellings"] and v.startswith("@env:")):
                    raise Refusal("value-is-a-channel-spelling", s["key"],
                                  f"{cli} {row['version']} reads {v[:5]!r}… on the command line as a channel, not as the secret")
        return [{"source": i, "key": s["key"], "kind": "Argv", "terminator": ""} for i, s in enumerate(res)], prov, res
    try:
        return _plan(res, table, platform), prov, res
    except Refusal as e:
        if platform not in POLICY["fd_channel_on"] and e.code in ("two-stdin", "no-channel-left", "no-channel-on-platform"):
            try:
                _plan(res, table, POLICY["fd_channel_on"][0])
            except Refusal:
                raise e
            raise Refusal("fd-not-on-platform", e.source, f"needs a pipe fd; not enabled on {platform}")
        raise


def _plan(sources, table, platform):
    avail = []
    for s in sources:
        vals = s["value"] if s["form"] == "group" else [s["value"]]
        chans = list(table[s["key"]]["channels"])
        if platform not in POLICY["fd_channel_on"]:
            chans = [c for c in chans if c["kind"] not in FD_KINDS]
        if not chans:
            raise Refusal("no-channel-on-platform", s["key"], platform)
        if not all(is_clean(v) for v in vals):                 # lenient channels need a clean value
            chans = [c for c in chans if c["terminator"] is not None]
            if not chans:
                raise Refusal("value-not-byte-exact", s["key"],
                              "every channel for this input trims whitespace; the value has CR/LF or edge whitespace")
        avail.append(chans)

    def pick(chans, kinds):
        for k in kinds:
            for c in chans:
                if c["kind"] == k:
                    return c
        return None

    assigned = [None] * len(sources)
    stdin_owner = None
    # Step 1 — forced stdin: sources whose every channel is a stdin channel.
    forced = [i for i, ch in enumerate(avail) if all(c["kind"] in STDIN_KINDS for c in ch)]
    if len(forced) > 1:
        raise Refusal("two-stdin", " + ".join(sources[i]["key"] for i in forced), "each has stdin as its only channel")
    if forced:
        i = forced[0]
        assigned[i] = pick(avail[i], STDIN_KINDS)
        stdin_owner = i
    # Step 2 — if stdin is still free, the first source (argv order) with a --X-stdin toggle.
    if stdin_owner is None:
        for i, ch in enumerate(avail):
            c = pick(ch, ("StdinToggle",))
            if c:
                assigned[i], stdin_owner = c, i
                break
    # Step 3 — the rest, argv order: env, else stdin if free, else fd, else refuse.
    for i, ch in enumerate(avail):
        if assigned[i]:
            continue
        c = pick(ch, ("EnvRef",))
        if not c and stdin_owner is None:
            c = pick(ch, STDIN_KINDS)
            if c:
                stdin_owner = i
        if not c:
            c = pick(ch, FD_KINDS)
        if not c:
            raise Refusal("no-channel-left", sources[i]["key"], "stdin already used and no env/fd channel")
        assigned[i] = c

    bindings, fd_next = [], 3
    for i, c in enumerate(assigned):
        b = {"source": i, "key": sources[i]["key"], "kind": c["kind"], "terminator": c["terminator"] or ""}
        if "flag" in c:
            b["flag"] = c["flag"]
        if c["kind"] == "EnvRef":
            b["env"] = f"{ENV_PREFIX}{i}"
        if c["kind"] in FD_KINDS:
            b["fd"] = fd_next
            fd_next += 1
            v = sources[i]["value"]
            payload = ("\n".join(v) if isinstance(v, list) else v) + b["terminator"]
            if len(payload.encode()) > POLICY["pipe_payload_max"]:
                raise Refusal("payload-too-large", sources[i]["key"], f"> {POLICY['pipe_payload_max']} bytes")
        bindings.append(b)
    return bindings


def describe(bindings, prov=None):
    out = []
    for b in bindings:
        k = b["kind"]
        where = {"EnvRef": f"env {b.get('env')}", "StdinToggle": f"stdin via {b.get('flag')}",
                 "DashValue": "stdin via `-`", "PosDash": "stdin via positional `-`",
                 "StdinMulti": "stdin via one `-` (all, one per line)", "Argv": "argv + --allow-argv-secret (interim)",
                 "FileFlag": f"pipe fd via {b.get('flag')}", "InFile": "pipe fd via --in"}[k]
        if b.get("terminator") and k != "StdinMulti":
            where += f" + {b['terminator']!r}"
        out.append(f"{b['key'].split(' ', 2)[-1]} ← {where}")
    return "; ".join(out)
