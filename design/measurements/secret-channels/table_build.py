"""Build channel_table.json: for every measured secret input, the list of channels that were
measured OK (exit + stdout equal to the argv baseline, with a secret-dependent baseline).
Sources: channels.json, channels2.json, channels3.json (later files override earlier rows with
the same label), and groups.json (measure_groups.py). Rows whose baseline was invalid are dropped."""
import json, re
rows={}
for f in ["channels.json","channels2.json","channels3.json"]:
    for r in json.load(open(f)):
        rows[r["label"]]=r
def norm(label): return re.sub(r"@\d+\.", "@N.", label)
def kind(name):
    # map the harness's variant name to a Channel kind (+ the flag spelling it needs)
    m=re.fullmatch(r"stdin `(.+)=-`",name)
    if m: return {"kind":"DashValue"}                       # <node>=-  /  @N.<subkey>=-
    m=re.fullmatch(r"`(.+)=@env:VAR`",name)
    if m: return {"kind":"EnvRef"}
    m=re.fullmatch(r"stdin `(--[\w-]+) -`",name)
    if m: return {"kind":"DashValue"}                       # <flag> -
    m=re.fullmatch(r"`(--[\w-]+) @env:VAR`",name)
    if m: return {"kind":"EnvRef"}
    m=re.fullmatch(r"`(--[\w-]+)-stdin`",name)
    if m: return {"kind":"StdinToggle","flag":m.group(1)+"-stdin"}
    m=re.fullmatch(r"`(--[\w-]+)-file F`",name)
    if m: return {"kind":"FileFlag","flag":m.group(1)+"-file"}
    if name=="`--in F`": return {"kind":"InFile","flag":"--in"}
    if name=="stdin `-`": return {"kind":"PosDash"}
    if name=="`@env:VAR`": return {"kind":"EnvRef"}
    raise ValueError(name)
table={}
for label,r in rows.items():
    valid = r["base_exit"] in (0,4) and r["depends"]
    if not valid: continue
    if label=="ms combine <shares> (first)": continue   # superseded by the group row (measure_groups.py)
    ok=[kind(n) for n,v in r["channels"].items() if v=="OK"]
    table[norm(label)]={"channels":ok,"measured_in":"single-input harness"}
for label,chs in json.load(open("groups.json")).items():
    table[label]={"channels":chs,"measured_in":"measure_groups.py"}
json.dump(dict(sorted(table.items())),open("channel_table.json","w"),indent=1)
print(len(table),"inputs;",sum(1 for v in table.values() if not v["channels"]),"with no OK channel")
