"""fold-1 single-input rows: R0 M2 inventory + the previously unmeasured list + verify-bundle
with a MATCHING baseline (R0 Q4). Same measure() as run_all.py."""
from cases2 import *
import json as _json
C3=[]
def c3(label,argv,mode,S,Cc,multi=False,check=None):
    d=dict(label=label,argv=argv,mode=mode,S=S,C=Cc,multi=multi)
    if check: d["check"]=check
    C3.append(d)
PW="hunter2-passphrase"
BIP38_A="6PYP8fdoVaE3ThLmEnYcGo3nJeBqd8PvB7CRvTz3TX5L9ojoPHKCg7QXG6"   # wif KyZp… under PW
BIP38_B="6PRRANN4U1afA3TLpAnVtwP85rSyzZvtXy98hLW91T2g2RZfad31jdsgtb"   # wif 5JPy… under PW
PRE1=open(SC+"pre1.ms1").read().strip(); PRE2=open(SC+"pre2.ms1").read().strip()
BIE1=SC+"electrum-bie1-storage-bip84.txt"
BJ=_json.load(open(SC+"bundle.json")); BPJ=_json.load(open(SC+"bundle_pw.json"))
def vb_cards(j): return ["--mk1"]+j["mk1"]+["--md1"]+j["md1"]
MS1_ALT="ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7g"
# M2 inventory
c3("mnemonic convert --from bip38=",[m,"convert",A,"--from","bip38={S}","--bip38-passphrase",PW,"--to","wif"],"node",BIP38_A,BIP38_B,multi=True)
c3("mnemonic slip39 split --from entropy=",[m,"slip39","split",A,"--from","entropy={S}","--group-threshold","1","--group","3,2"],"node",ENT,ENT2,check=split_check_slip39)
c3("mnemonic ms-shares split --from entropy=",[m,"ms-shares","split",A,"--from","entropy={S}","--threshold","2","--shares","2"],"node",ENT,ENT2,check=check_msshares)
c3("mnemonic xpub-search passphrase-of-xpub --ms1",[m,"xpub-search","passphrase-of-xpub",A,"--target-xpub",XPUB84_PW,"--passphrase",PW,"--ms1","{S}"],"value",MS1,MS1_ALT,multi=True)
c3("ms hashlock <ms1>",[s,"hashlock",A,"--kind","sha256","--","{S}"],"pos",PRE1,PRE2)
# previously unmeasured
c3("mnemonic bundle --slot @0.ms1=",[m,"bundle",A,"--network","mainnet","--template","bip84","--slot","@0.ms1={S}"],"node",MS1,"ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqcwugpdxtfme2w")
c3("mnemonic bundle --slot @0.wif=",[m,"bundle",A,"--network","mainnet","--template","bip84","--slot","@0.wif={S}"],"node",WIF,"5JPy8Zg7z4P7RSLsiqcqyeAF1935zjNUdMxcDeVrtU1oarrgnB7")
c3("mnemonic addresses --from seedqr=",[m,"addresses",A,"--address-type","p2wpkh","--count","1","--from","seedqr={S}"],"node",SQR,"000100000000000000000000000000000000000000000003")
c3("mnemonic import-wallet --decrypt-password",[m,"import-wallet",A,"--blob",BIE1,"--json","--decrypt-password","{S}"],"value","satoshi","wrong-password")
# verify-bundle, MATCHING baseline (result: ok). Every row carries the other two secrets on argv (multi).
c3("mnemonic verify-bundle --slot @0.phrase=",[m,"verify-bundle",A,"--network","mainnet","--template","bip84","--ms1",BJ["ms1"][0],"--slot","@0.phrase={S}"]+vb_cards(BJ),"node",P,P2,multi=True)
c3("mnemonic verify-bundle --ms1",[m,"verify-bundle",A,"--network","mainnet","--template","bip84","--slot","@0.phrase="+P,"--ms1","{S}"]+vb_cards(BJ),"value",BJ["ms1"][0],MS1_ALT,multi=True)
c3("mnemonic verify-bundle --passphrase",[m,"verify-bundle",A,"--network","mainnet","--template","bip84","--ms1",BPJ["ms1"][0],"--slot","@0.phrase="+P,"--passphrase","{S}"]+vb_cards(BPJ),"value",PW,"wrong-passphrase",multi=True)
# fold-1 second pass: schema sources the CLI accepts that had no row (probe_missing.py)
c3("mnemonic addresses --from electrum-phrase=",[m,"addresses",A,"--address-type","p2wpkh","--count","1","--from","electrum-phrase={S}"],"node",
   "wild father tree among universe such mobile favorite target dynamic credit identify",
   "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about")
for sk,v,alt in [("entropy",ENT,ENT2),("seedqr",SQR,"000100000000000000000000000000000000000000000003"),("ms1",MS1,MS1_ALT)]:
    c3(f"mnemonic verify-bundle --slot @0.{sk}=",[m,"verify-bundle",A,"--network","mainnet","--template","bip84","--ms1",BJ["ms1"][0],"--slot",f"@0.{sk}={{S}}"]+vb_cards(BJ),"node",v,alt,multi=True)
