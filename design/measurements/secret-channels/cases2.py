from cases import *
C2=[]
def c2(label,argv,mode,S,Cc,multi=False,check=None):
    d=dict(label=label,argv=argv,mode=mode,S=S,C=Cc,multi=multi)
    if check: d["check"]=check
    C2.append(d)
S39B=[l for l in open(SC+"s39b.all").read().splitlines() if l.strip()]
c2("mnemonic slip39 combine --share",[m,"slip39","combine",A,"--share","{S}","--share",S39[1]],"value",S39[0],S39B[0],multi=True)
CARDS=[x for x in bundle_lines if x.startswith(("ms1","mk1","md1"))]
PWCARDS=[x for x in open(SC+"bundle_pw.out").read().split() if x.startswith(("ms1","mk1","md1"))]
c2("mnemonic verify-bundle --slot @0.phrase=",[m,"verify-bundle",A,"--network","mainnet","--template","bip84","--slot","@0.phrase={S}","--"]+CARDS,"node",P,P2,multi=True)
c2("mnemonic verify-bundle --passphrase",[m,"verify-bundle",A,"--network","mainnet","--template","bip84","--slot","@0.phrase="+P,"--passphrase","{S}","--"]+PWCARDS,"value","hunter2-passphrase",PW2,multi=True)
for mode in ["path-of-xpub"]:
    c2(f"mnemonic xpub-search {mode} --passphrase",[m,"xpub-search",mode,A,"--target-xpub",XPUB84_PW,"--phrase",P,"--passphrase","{S}"],"value","hunter2-passphrase",PW2,multi=True)
c2("mnemonic xpub-search account-of-descriptor --passphrase",[m,"xpub-search","account-of-descriptor",A,"--descriptor",f"wpkh({XPUB84_PW}/0/*)","--phrase",P,"--passphrase","{S}"],"value","hunter2-passphrase",PW2,multi=True)
