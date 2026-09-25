"""Pure half of the plan table (R1 Nm3): plan.py applied to every shape in shapes.py on every
platform. Needs NO binaries, so check_design_tables.py regenerates §A5 from plan.py on every
run. Writes plans_pure.json and a5.md."""
import json, os
from plan import plan, describe, Refusal, POLICY
from shapes import SHAPES

HERE = os.path.dirname(os.path.abspath(__file__))
TABLE = json.load(open(os.path.join(HERE, "channel_table.json")))
PLATFORMS = ("linux", "macos", "windows")


def payloads(bindings, res):
    """The exact bytes each binding delivers (R2 NI3: T8 pins VALUES, not only shapes)."""
    out = []
    for b, s in zip(bindings, res):
        v = s["value"]
        if isinstance(v, list):
            out.append("\n".join(v))
        else:
            out.append(v + ("" if b["kind"] == "Argv" else b["terminator"]))
    return out


def value_cases(sh):
    """Per OS: the typed case, and each source typed as `@env:USER_SECRET` holding value + '\\n'."""
    cases = []
    variants = [("typed", sh["sources"], {})]
    for i, s in enumerate(sh["sources"]):
        if s["form"] != "group":
            srcs = [dict(x) for x in sh["sources"]]
            srcs[i]["value"] = "@env:USER_SECRET"
            variants.append((f"src{i} as @env:USER_SECRET", srcs, {"USER_SECRET": s["value"] + "\n"}))
    for name, srcs, uenv in variants:
        for p in PLATFORMS:
            try:
                b, prov, res = plan(srcs, TABLE, p, uenv)
                cases.append({"case": name, "os": p, "kinds": [x["kind"] for x in b], "provenance": prov,
                              "payloads": payloads(b, res)})
            except Refusal as e:
                cases.append({"case": name, "os": p, "refusal": e.code})
    return cases


def generate():
    rows = []
    for sh in SHAPES:
        row = {"name": sh["name"], "plans": {}, "value_cases": value_cases(sh)}
        for p in PLATFORMS:
            try:
                b, prov, _ = plan(sh["sources"], TABLE, p)
                row["plans"][p] = {"bindings": b}
            except Refusal as e:
                row["plans"][p] = {"refusal": e.code, "detail": str(e)}
        # what macOS/Windows plan once their private_channels_on flag flips (fd stays Linux-only)
        saved = list(POLICY["private_channels_on"])
        POLICY["private_channels_on"][:] = list(PLATFORMS)
        try:
            b, _, _ = plan(sh["sources"], TABLE, "macos")
            row["plans"]["other_once_enabled"] = {"bindings": b}
        except Refusal as e:
            row["plans"]["other_once_enabled"] = {"refusal": e.code, "detail": str(e)}
        finally:
            POLICY["private_channels_on"][:] = saved
        rows.append(row)
    return rows


def cell(p):
    return describe(p["bindings"]) if "bindings" in p else f"**refuse** ({p['refusal']})"


def a5_markdown(rows):
    from collections import Counter
    from plan import REINTERPRET
    rules = Counter(str(v.get("cli_env_rule")) for v in TABLE.values())
    head = (f"Policy (decided): private channels on {POLICY['private_channels_on']}, fd channel on {POLICY['fd_channel_on']}. "
            f"Derived (measured, CI-regenerated): per-input CLI `@env:` rule "
            + ", ".join(f"{k} ×{n}" for k, n in sorted(rules.items()))
            + "; argv re-reads "
            + ", ".join(f"{k} {v['version']}: {' '.join('`'+x+'`' for x in v['spellings'])}" for k, v in REINTERPRET.items())
            + ".\n\n")
    out = head + "| shape | Linux plan | macOS | Windows | macOS/Windows once their flag flips |\n|---|---|---|---|---|\n"
    for r in rows:
        mac = "same as Linux" if r["plans"]["macos"] == r["plans"]["linux"] else cell(r["plans"]["macos"])
        win = "same as macOS" if r["plans"]["windows"] == r["plans"]["macos"] else cell(r["plans"]["windows"])
        once = "same as Linux" if r["plans"]["other_once_enabled"] == r["plans"]["linux"] else cell(r["plans"]["other_once_enabled"])
        out += f"| {r['name']} | {cell(r['plans']['linux'])} | {mac} | {win} | {once} |\n"
    return out


if __name__ == "__main__":
    rows = generate()
    json.dump(rows, open(os.path.join(HERE, "plans_pure.json"), "w"), indent=1)
    open(os.path.join(HERE, "a5.md"), "w").write(a5_markdown(rows))
    print(open(os.path.join(HERE, "a5.md")).read())
