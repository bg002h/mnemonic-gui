import subprocess, json, os, sys, tempfile
B=os.environ["BIN_DIR"].rstrip("/")+"/"
P="abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
P2="zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo wrong"
MS1="ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f"
ENT="00000000000000000000000000000000"; ENT2="ffffffffffffffffffffffffffffffff"
XPRV="xprv9ybY78BftS5UGANki6oSifuQEjkpyAC8ZmBvBNTshQnCBcxnefjHS7buPMkkqhcRzmoGZ5bokx7GuyDAiktd5HemohAU4wV1ZPMDRmLpBMm"
WIF="KyZpNDKnfs94vbrwhJneDi77V6jF64PWPF8x5cdJb8ifgg2DUc9d"
SQR="000000000000000000000000000000000000000000000003"
MINI="S6c56bnXQiBjk9mqSYE7ykVQ7NzrRy"
NSEC1="0000000000000000000000000000000000000000000000000000000000000001"
NSEC2="0000000000000000000000000000000000000000000000000000000000000002"
PW="hunter2-passphrase"; PW2="other-passphrase"
ENV="GUI_SECRET_T"
NL=os.environ.get("NL","\n")
TMP=tempfile.mkdtemp()
import atexit, shutil
atexit.register(shutil.rmtree, TMP, True)
EXTRA={}  # filled by setup
def run(argv, stdin=None, env_val=None):
    env=dict(os.environ)
    if env_val is not None: env[ENV]=env_val
    p=subprocess.run(argv,input=(stdin if stdin is not None else ""),capture_output=True,text=True,env=env,timeout=120)
    return p.returncode,p.stdout,p.stderr
def first_err(e):
    for l in e.splitlines():
        l=l.strip()
        if l and not l.startswith("warning:") and not l.startswith("note:"): return l[:150]
    return ""
def sub(argv,S): return [a.replace("{S}",S) for a in argv]
def variants(case):
    argv,mode=case["argv"],case["mode"]
    i=next(k for k,a in enumerate(argv) if "{S}" in a)
    tok=argv[i]
    out=[]
    if mode=="node":
        pre=tok.split("{S}")[0]
        out.append(("stdin `"+pre+"-`", argv[:i]+[pre+"-"]+argv[i+1:], "stdin", None))
        out.append(("`"+pre+"@env:VAR`", argv[:i]+[pre+"@env:"+ENV]+argv[i+1:], None, "env"))
    elif mode=="value":
        F=argv[i-1]
        out.append(("stdin `"+F+" -`", argv[:i]+["-"]+argv[i+1:], "stdin", None))
        out.append(("`"+F+" @env:VAR`", argv[:i]+["@env:"+ENV]+argv[i+1:], None, "env"))
        out.append(("`"+F+"-stdin`", argv[:i-1]+[F+"-stdin"]+argv[i+1:], "stdin", None))
        out.append(("`"+F+"-file F`", argv[:i-1]+[F+"-file", "{FILE}"]+argv[i+1:], None, None))
        out.append(("`--in F`", argv[:i-1]+["--in", "{FILE}"]+argv[i+1:], None, None))
    elif mode=="pos":
        out.append(("stdin `-`", argv[:i]+["-"]+argv[i+1:], "stdin", None))
        out.append(("`@env:VAR`", argv[:i]+["@env:"+ENV]+argv[i+1:], None, "env"))
        rest=argv[:i]+argv[i+1:]
        if rest and rest[-1]=="--": rest=rest[:-1]
        out.append(("`--in F`", rest+["--in","{FILE}"], None, None))
    return out
def measure(case):
    S,C=case["S"],case["C"]
    allow=[case["argv"][0]]+case.get("pre",[])
    base=case["argv"]
    chk=case.get("check",lambda o:o)
    rc,so,se=run(sub(base,S))
    rc2,so2,se2=run(sub(base,C))
    so,so2=chk(so),chk(so2)
    r={"label":case["label"],"base_exit":rc,"base_err":first_err(se) if rc else "","depends":(so!=so2) or (rc!=rc2),"channels":{}}
    if rc not in (0,4): return r
    for name,argv,stdin,env in variants(case):
        f=tempfile.mktemp(dir=TMP,suffix=".txt"); open(f,"w").write(S+"\n")
        a=[x.replace("{FILE}",f) for x in argv]
        if not case.get("multi"): a=[x for x in a if x!="--allow-argv-secret"]
        crc,cso,cse=run(a, stdin=(S+NL) if stdin else None, env_val=S if env else None)
        cso=chk(cso) if crc in (0,4) else cso
        if crc==rc and cso==so: v="OK"
        elif crc in (0,4): v=f"exit {crc} but DIFFERENT output"
        else: v=f"no (exit {crc}: {first_err(cse)})"
        r["channels"][name]=v
    return r
