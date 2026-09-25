"""Pure half of the plan table (R1 Nm3): plan.py applied to every shape in shapes.py on every
platform. Needs NO binaries, so check_design_tables.py regenerates §A5 from plan.py on every
run. Writes plans_pure.json and a5.md."""
import json, os
from plan import plan, describe, Refusal, POLICY
from shapes import SHAPES

HERE = os.path.dirname(os.path.abspath(__file__))
TABLE = json.load(open(os.path.join(HERE, "channel_table.json")))
PLATFORMS = ("linux", "macos", "windows")


def generate():
    rows = []
    for sh in SHAPES:
        row = {"name": sh["name"], "plans": {}}
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
    head = (f"Policy: private channels on {POLICY['private_channels_on']}, fd channel on "
            f"{POLICY['fd_channel_on']}, env_value_rule `{POLICY['env_value_rule']}` (channel_policy.json).\n\n")
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
