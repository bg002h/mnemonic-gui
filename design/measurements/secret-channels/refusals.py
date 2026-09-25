import subprocess, json, re, sys, os
B=os.environ["BIN_DIR"].rstrip("/")+"/"
P="abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
MS1="ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f"
ENT="00000000000000000000000000000000"
XPRV="xprv9ybY78BftS5UGANki6oSifuQEjkpyAC8ZmBvBNTshQnCBcxnefjHS7buPMkkqhcRzmoGZ5bokx7GuyDAiktd5HemohAU4wV1ZPMDRmLpBMm"
WIF="KyZpNDKnfs94vbrwhJneDi77V6jF64PWPF8x5cdJb8ifgg2DUc9d"
SQR="000000000000000000000000000000000000000000000003"
MINI="S6c56bnXQiBjk9mqSYE7ykVQ7NzrRy"
BIP38="6PYNKZ1EAgYgmQfmNVamxyXVWHzK5s6DGhwP4J5o44cvXdoY7sRzhtpUeo"
ELEC="wild father tree among universe such mobile favorite target dynamic credit identify"
SLIP="duckling enlarge academic academic agency result length solution fridge kidney coal piece deal husband erode duke ajar critical decision keyboard"
PW="hunter2-passphrase"
nodes={"phrase":P,"seedqr":SQR,"entropy":ENT,"xprv":XPRV,"wif":WIF,"ms1":MS1,"bip38":BIP38,"minikey":MINI,"electrum-phrase":ELEC}
cases=[]
def add(cli, sub, flag, val, label=None):
    cases.append((cli, sub, flag, val, label or flag))
m="mnemonic"
for n in ["phrase","ms1","entropy","seedqr"]: add(m,["addresses"],"--from",f"{n}={nodes[n]}",f"--from {n}=")
add(m,["addresses"],"--passphrase",PW)
for sk in ["phrase","seedqr","entropy","ms1","xprv","wif"]: add(m,["bundle"],"--slot",f"@0.{sk}={nodes[sk]}",f"--slot @0.{sk}=")
add(m,["bundle"],"--passphrase",PW)
add(m,["verify-bundle"],"--passphrase",PW)
add(m,["verify-bundle"],"--from",f"ms1={MS1}","--from ms1=")
add(m,["verify-bundle"],"--ms1",MS1)
add(m,["verify-bundle"],"--slot",f"@0.phrase={P}","--slot @0.phrase=")
for n in nodes: add(m,["convert"],"--from",f"{n}={nodes[n]}",f"--from {n}=")
add(m,["convert"],"--passphrase",PW); add(m,["convert"],"--bip38-passphrase",PW)
add(m,["export-wallet"],"--slot",f"@0.phrase={P}","--slot @0.phrase=")
for n in ["ms1","phrase","entropy","seedqr"]: add(m,["restore"],"--from",f"{n}={nodes[n]}",f"--from {n}=")
add(m,["restore"],"--passphrase",PW)
for n in ["xprv","phrase"]: add(m,["derive-child"],"--from",f"{n}={nodes[n]}",f"--from {n}=")
add(m,["derive-child"],"--passphrase",PW)
add(m,["electrum-decrypt"],"--decrypt-password",PW)
add(m,["nostr"],"--secret","nsec1vl029mgpspedva04g90vltkh6fvh240zqtv9k0t9af8935ke9laqsnlfe5")
add(m,["silent-payment"],"--passphrase",PW); add(m,["silent-payment"],"--secret",P)
add(m,["final-word"],"--from",f"phrase={P}","--from phrase=")
add(m,["seed-xor","split"],"--from",f"phrase={P}","--from phrase=")
add(m,["seed-xor","combine"],"--share",f"phrase={P}","--share phrase=")
add(m,["seedqr","encode"],"--from",f"phrase={P}","--from phrase=")
add(m,["seedqr","decode"],"--from",f"seedqr={SQR}","--from seedqr=")
add(m,["seedqr","decode"],"--digits",SQR)
for n in ["phrase","entropy"]: add(m,["slip39","split"],"--from",f"{n}={nodes[n]}",f"--from {n}=")
add(m,["slip39","split"],"--passphrase",PW)
add(m,["slip39","combine"],"--share",SLIP); add(m,["slip39","combine"],"--passphrase",PW)
for n in ["phrase","entropy"]: add(m,["ms-shares","split"],"--from",f"{n}={nodes[n]}",f"--from {n}=")
add(m,["ms-shares","combine"],"--share",MS1)
add(m,["repair"],"--ms1",MS1); add(m,["inspect"],"--ms1",MS1)
add(m,["import-wallet"],"--ms1",MS1); add(m,["import-wallet"],"--decrypt-password",PW)
add(m,["import-wallet"],"--slot",f"@0.phrase={P}","--slot @0.phrase=")
for mode in ["path-of-xpub","account-of-descriptor","passphrase-of-xpub"]:
    add(m,["xpub-search",mode],"--phrase",P); add(m,["xpub-search",mode],"--ms1",MS1); add(m,["xpub-search",mode],"--passphrase",PW)
add(m,["word-card"],"--from",f"ms1={MS1}","--from ms1=")
add(m,["word-card"],"--from",f"phrase={P}","--from phrase=")
s="ms"
for sub in ["inspect","decode","verify","derive"]: add(s,[sub],None,MS1,"<ms1>")
add(s,["encode"],"--phrase",P); add(s,["encode"],"--hex",ENT)
add(s,["verify"],"--phrase",P)
add(s,["derive"],"--hex",ENT); add(s,["derive"],"--phrase",P); add(s,["derive"],"--passphrase",PW)
add(s,["repair"],"--ms1",MS1)
add(s,["split"],"--phrase",P); add(s,["split"],"--hex",ENT)
add(s,["combine"],None,MS1,"<shares>")
add(s,["hashlock"],None,"correct horse battery staple","<positional>")
add(s,["hashlock"],"--phrase","correct horse battery staple")
res=[]
for cli,sub,flag,val,label in cases:
    argv=[B+cli]+sub+([flag,val] if flag else ["--",val])
    p=subprocess.run(argv,capture_output=True,text=True,stdin=subprocess.DEVNULL,timeout=30)
    err=p.stderr
    refused = "Refused BEFORE" in err
    chans=[]
    if "these exist today:" in err:
        tail=err.split("these exist today:",1)[1]
        for line in tail.splitlines()[1:]:
            if line.startswith("          ") and line.strip():
                chans.append(line.strip())
            elif line.strip()=="" : continue
            else: break
    res.append(dict(cli=cli,sub=" ".join(sub),input=label,exit=p.returncode,refused=refused,channels=chans,first=err.strip().splitlines()[0][:160] if err.strip() else ""))
json.dump(res,open("refusals.json","w"),indent=1)
for r in res:
    print(f"{r['cli']} {r['sub']} | {r['input']} | exit {r['exit']} | refused={r['refused']} | {' ; '.join(r['channels']) or r['first']}")
