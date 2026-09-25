"""ms combine takes N shares as ONE group: `-` reads every share from stdin (one per line), and
`--in F` reads them from a file. Measured here against the argv baseline, with dependence."""
import json, os, subprocess
B=os.environ["BIN_DIR"].rstrip("/")+"/"
SH=["ms12ec7pqsf34k729gl6yrsfk9q244kzsx62s9krxwywxv4jav","ms12ec7pp3txgcymeun2ws3tceqmggc83f2m3l2cj299w9l9vr"]
SH_OTHER=["ms12wah2q74tl23u0cm5dzxvrkdx67hu7cj2qm9xglv9ajf4vr","ms12wah2pymqlh20wp3nft97u5f9xgr0gp6hpthlstfqk23dmh"]  # 2-of-2 of "zoo … wrong"
def run(argv,stdin="",fds=()):
    return subprocess.run(argv,input=stdin,capture_output=True,text=True,pass_fds=fds)
base=run([B+"ms","combine","--allow-argv-secret","--"]+SH)
other=run([B+"ms","combine","--allow-argv-secret","--"]+SH_OTHER)
assert base.returncode==0 and other.returncode==0 and base.stdout!=other.stdout, "baseline must depend on the shares"
ok=[]
r=run([B+"ms","combine","--","-"],stdin="\n".join(SH))
print("StdinMulti:", "OK" if (r.returncode==0 and r.stdout==base.stdout) else f"no ({r.returncode}) {r.stderr[:120]}")
if r.returncode==0 and r.stdout==base.stdout: ok.append({"kind":"StdinMulti","terminator":"\n"})   # group: one share per line
rd,wr=os.pipe(); os.write(wr,("\n".join(SH)+"\n").encode()); os.close(wr)
r=run([B+"ms","combine","--in",f"/dev/fd/{rd}"],fds=(rd,)); os.close(rd)
print("InFile (pipe fd):", "OK" if (r.returncode==0 and r.stdout==base.stdout) else f"no ({r.returncode}) {r.stderr[:120]}")
if r.returncode==0 and r.stdout==base.stdout: ok.append({"kind":"InFile","flag":"--in","terminator":"\n"})
json.dump({"ms combine <shares>":ok},open("groups.json","w"),indent=1)
