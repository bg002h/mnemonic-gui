import json, subprocess
from cases3 import C, C2, C3, sub, run
res={r["label"]:r for r in json.load(open("channels.json"))}
res.update({r["label"]:r for r in json.load(open("channels2.json"))})
res.update({r["label"]:r for r in json.load(open("channels3.json"))})
cases={c["label"]:c for c in C}; cases.update({c["label"]:c for c in C2}); cases.update({c["label"]:c for c in C3})
del cases["ms combine <shares> (first)"]   # superseded by the group row (measure_groups.py)
def refused(c):
    a=[x for x in sub(c["argv"],c["S"]) if x!="--allow-argv-secret"]
    rc,so,se=run(a)
    return "refused" if "Refused BEFORE" in se else f"NOT refused (exit {rc})"
def cell(r,prefix):
    for k,v in r["channels"].items():
        if k.startswith(prefix) or prefix in k:
            if v=="OK": return "OK"
            if "DIFFERENT" in v: return f"**WRONG (exit {v.split()[1]}, literal)**"
            if "unexpected argument" in v: return "—"
            return "fails closed"
    return "n/a"
rows=[]
for label,c in cases.items():
    r=res.get(label)
    if r is None: continue
    if c["mode"] in ("node","pos"):
        s=cell(r,"stdin"); e=cell(r,"@env"); t="n/a"; f=cell(r,"--in F") if c["mode"]=="pos" else "n/a"
    else:
        s=cell(r,"stdin `"); e=cell(r,"@env"); t=cell(r,"-stdin`"); f1=cell(r,"-file F"); f2=cell(r,"--in F")
        f = "OK (`-file`)" if f1=="OK" else ("OK (`--in`)" if f2=="OK" else ("—" if f1=="—" and f2 in("—","n/a") else f"{f1}/{f2}"))
    valid = r["base_exit"] in (0,4) and r["depends"]
    rows.append((label, refused(c), s, e, t, f, "yes" if valid else f"NO (base exit {r['base_exit']}, depends={r['depends']})"))
print("| input | argv w/o opt-in | `-` / positional `-` | `@env:VAR` | `--X-stdin` | file (`--X-file` / `--in`) | measurement valid |")
print("|---|---|---|---|---|---|---|")
for row in rows: print("| `"+row[0]+"` | "+" | ".join(row[1:])+" |")
