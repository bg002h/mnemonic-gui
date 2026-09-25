"""Reference model of the design's planner (DESIGN §A4). Executable specification: the design's
A5 plan table is GENERATED from this file (run_plans.py), and the Rust `form::channels::plan`
must agree with it on every shape (the design's T8 gate). This is a measurement/spec tool,
not GUI code.

A source is one secret input the user filled:
  {"key":   channel-table key, e.g. "mnemonic restore --passphrase",
   "form":  "node" | "value" | "pos" | "group",
   "flag":  "--from" / "--passphrase" / … (None for pos/group),
   "prefix":"phrase=" / "@0.phrase=" / "" (node form only),
   "value": the secret (str; a list of str for a group)}
"""

STDIN_KINDS = ("StdinMulti", "StdinToggle", "DashValue", "PosDash")   # preference order
FD_KINDS = ("FileFlag", "InFile")                                       # preference order
ENV_PREFIX = "MNEMONIC_GUI_S"

# Platforms on which the fd channel (an inherited pipe named /dev/fd/N) is enabled. Linux only:
# it is measured there (run_plans.py). macOS is refused until a macOS real-binary job measures it;
# Windows has no /dev/fd and is refused (DESIGN §A6, Q3).
FD_PLATFORMS = ("linux",)


class Refusal(Exception):
    def __init__(self, code, source, why):
        super().__init__(f"{code}: {source}: {why}")
        self.code, self.source, self.why = code, source, why


def resolve(sources, user_env):
    """C1 (DESIGN §A3a, operator ruling on F-687). A secret field whose value is a channel
    spelling means that channel, never those characters:
      `@env:VAR` -> the GUI reads VAR from its OWN environment, and the bytes go through the
                    planned private channel like any typed value. Unset or empty -> refuse.
      `-`        -> the user's own stdin. The GUI has none to forward -> refuse.
    Resolution is unconditional: it does not depend on whether the pinned CLI implements
    F-687, so a pin bump changes only channel_table.json, never this function.
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
                if name.startswith(ENV_PREFIX):
                    raise Refusal("C1-reserved-name", s["key"], f"{ENV_PREFIX}* names are the GUI's own")
                if not name or name not in user_env:
                    raise Refusal("C1-env-unset", s["key"], f"${name} is not set in the GUI's environment")
                if user_env[name] == "":
                    raise Refusal("C1-env-empty", s["key"], f"${name} is empty")
                got.append(user_env[name]); where.append(f"${name}")
            else:
                got.append(v); where.append("typed")
        r = dict(s)
        r["value"] = got if s["form"] == "group" else got[0]
        out.append(r)
        prov.append(where if s["form"] == "group" else where[0])
    return out, prov


def plan(sources, table, platform="linux"):
    """The rule. On a platform without the fd channel, a refusal that the fd channel would have
    avoided is reported as `fd-not-on-platform` (DESIGN §A6), not as its proximate cause."""
    try:
        return _plan(sources, table, platform)
    except Refusal as e:
        if platform not in FD_PLATFORMS and e.code in ("two-stdin", "no-channel-left", "no-channel-on-platform"):
            try:
                _plan(sources, table, FD_PLATFORMS[0])
            except Refusal:
                raise e
            raise Refusal("fd-not-on-platform", e.source, f"needs a pipe fd; not enabled on {platform}")
        raise


def _plan(sources, table, platform):
    for s in sources:          # plan() is only ever called on RESOLVED sources (see resolve())
        vals = s["value"] if s["form"] == "group" else [s["value"]]
        assert not any(v == "-" or v.startswith("@env:") for v in vals), "resolve() first"
    avail = []
    for s in sources:
        if s["key"] not in table:
            raise Refusal("no-table-entry", s["key"], "input not measured")
        chans = list(table[s["key"]]["channels"])
        if platform not in FD_PLATFORMS:
            chans = [c for c in chans if c["kind"] not in FD_KINDS]
        vals = s["value"] if s["form"] == "group" else [s["value"]]
        if any("\0" in v for v in vals):                  # env cannot carry NUL (§A8.4)
            chans = [c for c in chans if c["kind"] != "EnvRef"]
        if not chans:
            raise Refusal("no-channel-on-platform", s["key"], platform)
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
        raise Refusal("two-stdin", " + ".join(sources[i]["key"] for i in forced),
                      "each has stdin as its only channel")
    if forced:
        i = forced[0]
        assigned[i] = pick(avail[i], STDIN_KINDS)
        stdin_owner = i

    # Step 2 — if stdin is still free, the first source (argv order) with a `--X-stdin` toggle
    # takes it. These toggles read raw bytes (NUL-preserving) — the passphrase-class channel.
    if stdin_owner is None:
        for i, ch in enumerate(avail):
            c = pick(ch, ("StdinToggle",))
            if c:
                assigned[i] = c
                stdin_owner = i
                break

    # Step 3 — everything else, argv order: env, else stdin if free, else fd, else refuse.
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
            raise Refusal("no-channel-left", sources[i]["key"],
                          "stdin already used and no env/fd channel")
        assigned[i] = c

    bindings = []
    fd_next = 3
    for i, c in enumerate(assigned):
        b = {"source": i, "key": sources[i]["key"], "kind": c["kind"]}
        if "flag" in c:
            b["flag"] = c["flag"]
        if c["kind"] == "EnvRef":
            b["env"] = f"{ENV_PREFIX}{i}"
        if c["kind"] in FD_KINDS:
            b["fd"] = fd_next
            fd_next += 1
        bindings.append(b)
    return bindings


def describe(bindings):
    out = []
    for b in bindings:
        k = b["kind"]
        where = {"EnvRef": f"env {b.get('env')}", "StdinToggle": f"stdin via {b.get('flag')}",
                 "DashValue": "stdin via `-`", "PosDash": "stdin via positional `-`",
                 "StdinMulti": "stdin via one `-` (all, one per line)",
                 "FileFlag": f"pipe fd via {b.get('flag')}", "InFile": "pipe fd via --in"}[k]
        out.append(f"{b['key'].split(' ', 2)[-1]} ← {where}")
    return "; ".join(out)
