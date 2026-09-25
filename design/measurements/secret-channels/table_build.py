"""Build channel_table.json: for every measured secret input, the list of channels that were
measured OK (exit + stdout equal to the argv baseline, with a secret-dependent baseline).
Sources: channels.json, channels2.json, channels3.json (later files override earlier rows with
the same label), and groups.json (measure_groups.py). Rows whose baseline was invalid are dropped."""
import json, re
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
def main():
    rows={}
    for f in ["channels.json","channels2.json","channels3.json"]:
        for r in json.load(open(f)):
            rows[r["label"]]=r
    table={}
    for label,r in rows.items():
        valid = r["base_exit"] in (0,4) and r["depends"]
        if not valid: continue
        if label=="ms combine <shares> (first)": continue   # superseded by the group row (measure_groups.py)
        ok=[kind(n) for n,v in r["channels"].items() if v=="OK"]
        table[norm(label)]={"channels":ok,"measured_in":"single-input harness"}
    # R1 NI1: per-channel terminator from run_bytes.py (bytes.json). null = lenient: the channel
    # trims whitespace where argv would not, so it may carry only a CLEAN value (DESIGN §A3c).
    terms={}
    for r in json.load(open("bytes.json")):
        for c in r["channels"]:
            terms[(norm(r["label"]), c["kind"], c.get("flag"))]=c["terminator"]
    for label,v in table.items():
        for c in v["channels"]:
            c["terminator"]=terms[(label,c["kind"],c.get("flag"))]
    for label,chs in json.load(open("groups.json")).items():
        table[label]={"channels":chs,"measured_in":"measure_groups.py"}
    json.dump(dict(sorted(table.items())),open("channel_table.json","w"),indent=1)
    print(len(table),"inputs;",sum(1 for v in table.values() if not v["channels"]),"with no OK channel")


def record_versions():
    """measured_with.json: the CLI versions of BIN_DIR, the binaries every measuring script in the
    §A1 pipeline ran against. test_plan.py's pin check compares it with pinned-upstream.toml."""
    import os, subprocess
    b=os.environ["BIN_DIR"].rstrip("/")+"/"
    v={c: subprocess.run([b+c,"--version"],capture_output=True,text=True).stdout.split()[1] for c in ("mnemonic","md","ms","mk")}
    json.dump(v,open("measured_with.json","w"),indent=1)

if __name__=="__main__":
    main()
    record_versions()
