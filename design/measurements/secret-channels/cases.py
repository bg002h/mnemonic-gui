from channels import *
m=B+"mnemonic"; s=B+"ms"; A="--allow-argv-secret"
C=[]
def c(label,argv,mode,S,Cc,multi=False): C.append(dict(label=label,argv=argv,mode=mode,S=S,C=Cc,multi=multi))
nodes={"phrase":(P,P2),"entropy":(ENT,ENT2),"seedqr":(SQR,"000100000000000000000000000000000000000000000003"),"ms1":(MS1,None)}
# addresses
c("mnemonic addresses --from phrase=",[m,"addresses",A,"--address-type","p2wpkh","--count","1","--from","phrase={S}"],"node",P,P2)
c("mnemonic addresses --from entropy=",[m,"addresses",A,"--address-type","p2wpkh","--count","1","--from","entropy={S}"],"node",ENT,ENT2)
c("mnemonic addresses --passphrase",[m,"addresses",A,"--address-type","p2wpkh","--count","1","--from","phrase="+P,"--passphrase","{S}"],"value",PW,PW2,multi=True)
# bundle
for sk,(v,v2) in [("phrase",(P,P2)),("entropy",(ENT,ENT2)),("seedqr",(SQR,"000100000000000000000000000000000000000000000003"))]:
    c(f"mnemonic bundle --slot @0.{sk}=",[m,"bundle",A,"--network","mainnet","--template","bip84","--slot",f"@0.{sk}={{S}}"],"node",v,v2)
c("mnemonic bundle --passphrase",[m,"bundle",A,"--network","mainnet","--template","bip84","--slot","@0.phrase="+P,"--passphrase","{S}"],"value",PW,PW2,multi=True)
# convert
c("mnemonic convert --from phrase=",[m,"convert",A,"--from","phrase={S}","--to","xpub","--template","bip84"],"node",P,P2)
c("mnemonic convert --from entropy=",[m,"convert",A,"--from","entropy={S}","--to","xpub","--template","bip84"],"node",ENT,ENT2)
c("mnemonic convert --from seedqr=",[m,"convert",A,"--from","seedqr={S}","--to","xpub","--template","bip84"],"node",SQR,"000100000000000000000000000000000000000000000003")
c("mnemonic convert --from xprv=",[m,"convert",A,"--from","xprv={S}","--to","xpub"],"node",XPRV,"xprv9s21ZrQH143K3QTDL4LXw2F7HEK3wJUD2nW2nRk4stbPy6cq3jPPqjiChkVvvNKmPGJxWUtg6LNnd5vjUBGRTb5rYs2PwEAaRS9CBYg5ZEDc")
c("mnemonic convert --from wif=",[m,"convert",A,"--from","wif={S}","--to","address"],"node",WIF,"5JPy8Zg7z4P7RSLsiqcqyeAF1935zjNUdMxcDeVrtU1oarrgnB7")
c("mnemonic convert --from minikey=",[m,"convert",A,"--from","minikey={S}","--to","wif"],"node",MINI,"S6c56bnXQiBjk9mqSYE7ykVQ7NzrRz")
c("mnemonic convert --from ms1=",[m,"convert",A,"--from","ms1={S}","--to","phrase"],"node",MS1,"ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7g")
c("mnemonic convert --from electrum-phrase=",[m,"convert",A,"--from","electrum-phrase={S}","--to","entropy"],"node","wild father tree among universe such mobile favorite target dynamic credit identify","abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about")
c("mnemonic convert --passphrase",[m,"convert",A,"--from","phrase="+P,"--to","xpub","--template","bip84","--passphrase","{S}"],"value",PW,PW2,multi=True)
c("mnemonic convert --bip38-passphrase",[m,"convert",A,"--from","wif="+WIF,"--to","bip38","--bip38-passphrase","{S}"],"value",PW,PW2,multi=True)
# restore
for n,(v,v2) in [("phrase",(P,P2)),("entropy",(ENT,ENT2)),("seedqr",(SQR,"000100000000000000000000000000000000000000000003")),("ms1",(MS1,"ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7g"))]:
    c(f"mnemonic restore --from {n}=",[m,"restore",A,"--from",f"{n}={{S}}","--template","bip84"],"node",v,v2)
c("mnemonic restore --passphrase",[m,"restore",A,"--from","phrase="+P,"--template","bip84","--passphrase","{S}"],"value",PW,PW2,multi=True)
# derive-child
c("mnemonic derive-child --from phrase=",[m,"derive-child",A,"--from","phrase={S}","--application","bip39","--length","12","--index","0"],"node",P,P2)
c("mnemonic derive-child --from xprv=",[m,"derive-child",A,"--from","xprv={S}","--application","bip39","--length","12","--index","0"],"node","xprv9s21ZrQH143K3QTDL4LXw2F7HEK3wJUD2nW2nRk4stbPy6cq3jPPqjiChkVvvNKmPGJxWUtg6LNnd5vjUBGRTb5rYs2PwEAaRS9CBYg5ZEDc",XPRV)
c("mnemonic derive-child --passphrase",[m,"derive-child",A,"--from","phrase="+P,"--application","bip39","--length","12","--index","0","--passphrase","{S}"],"value",PW,PW2,multi=True)
# nostr / silent-payment
c("mnemonic nostr --secret",[m,"nostr",A,"--secret","{S}"],"value",NSEC1,NSEC2)
c("mnemonic silent-payment --secret",[m,"silent-payment",A,"--secret","{S}"],"value",P,P2)
c("mnemonic silent-payment --passphrase",[m,"silent-payment",A,"--secret",P,"--passphrase","{S}"],"value",PW,PW2,multi=True)
# final-word, seed-xor, seedqr
c("mnemonic final-word --from phrase=",[m,"final-word",A,"--from","phrase={S}"],"node"," ".join(["abandon"]*11)," ".join(["zoo"]*11))
c("mnemonic seed-xor split --from phrase=",[m,"seed-xor","split",A,"--from","phrase={S}","--shares","2","--deterministic-from-master"],"node",P,P2)
c("mnemonic seed-xor combine --share phrase=",[m,"seed-xor","combine",A,"--share","phrase={S}","--share","phrase="+P2,"--shares","2"],"node",P,"legal winner thank year wave sausage worth useful legal winner thank yellow",multi=True)
c("mnemonic seedqr encode --from phrase=",[m,"seedqr","encode",A,"--from","phrase={S}"],"node",P,P2)
c("mnemonic seedqr decode --from seedqr=",[m,"seedqr","decode",A,"--from","seedqr={S}"],"node",SQR,"000100000000000000000000000000000000000000000003")
c("mnemonic seedqr decode --digits",[m,"seedqr","decode",A,"--digits","{S}"],"value",SQR,"000100000000000000000000000000000000000000000003")
# repair/inspect
c("mnemonic repair --ms1",[m,"repair",A,"--ms1","{S}"],"value",MS1,"ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7g")
c("mnemonic inspect --ms1",[m,"inspect",A,"--ms1","{S}"],"value",MS1,"ms10testsxxxxxxxxxxxxxxxxxxxxxxxxxx4nzvca9cmczlw")
# ms
c("ms encode --phrase",[s,"encode",A,"--phrase","{S}"],"value",P,P2)
c("ms encode --hex",[s,"encode",A,"--hex","{S}"],"value",ENT,ENT2)
for sub_ in ["inspect","decode","verify","derive"]:
    extra=["--phrase",P] if sub_=="verify" else []
    c(f"ms {sub_} <ms1>",[s,sub_,A]+extra+["--","{S}"],"pos",MS1,"ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7g",multi=bool(extra))
c("ms verify --phrase",[s,"verify",A,"--phrase","{S}","--",MS1],"value",P,P2,multi=True)
c("ms derive --hex",[s,"derive",A,"--hex","{S}"],"value",ENT,ENT2)
c("ms derive --phrase",[s,"derive",A,"--phrase","{S}"],"value",P,P2)
c("ms derive --passphrase",[s,"derive",A,"--phrase",P,"--passphrase","{S}"],"value",PW,PW2,multi=True)
c("ms repair --ms1",[s,"repair",A,"--ms1","{S}"],"value",MS1,"ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7g")
import subprocess as _sp
SC=os.path.join(os.path.dirname(os.path.abspath(__file__)),"fixtures")+"/"
S39=[l for l in open(SC+"s39.all").read().splitlines() if l.strip()]
T24="abandon "*23+"art"; T24_MS1="ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqcwugpdxtfme2w"
BLOB=SC+"blob.bsms"
bundle_lines=open(SC+"bundle.out").read().split()
MKMD=[x for x in bundle_lines if x.startswith("mk1") or x.startswith("md1")]
XPUB84="xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V"
XPUB84_PW="xpub6CPUCVp94gpNs3bS1eGiwSWwZMLNmfr2Uo1t5v8YtY4XVoxhUraBH7sRyVfgwSNCxRVpX1bDREtc5KriC7smtjYco8w57agUrg1nLawsNFG"
XPRV2="xprv9yZC9RZoFia6XiN4rFcyb3KJUxMrq8NWmzytjjmLa8aSZgnxHnYc9vzBaquLEJXXA5UzU8epwCiHcagwhwpzu3kHPXXLYPR9oRS4Lr8brMC"
# fix-ups of earlier broken baselines
C[:]=[x for x in C if x["label"] not in ("mnemonic derive-child --from xprv=","mnemonic convert --from wif=")]
c("mnemonic derive-child --from xprv=",[m,"derive-child",A,"--from","xprv={S}","--application","bip39","--length","12","--index","0"],"node",XPRV,XPRV2)
c("mnemonic convert --from wif=",[m,"convert",A,"--from","wif={S}","--to","xpub"],"node",WIF,"5JPy8Zg7z4P7RSLsiqcqyeAF1935zjNUdMxcDeVrtU1oarrgnB7")
# verify-bundle
c("mnemonic verify-bundle --slot @0.phrase=",[m,"verify-bundle",A,"--network","mainnet","--template","bip84","--slot","@0.phrase={S}","--"]+MKMD,"node",P,P2)
c("mnemonic verify-bundle --passphrase",[m,"verify-bundle",A,"--network","mainnet","--template","bip84","--slot","@0.phrase="+P,"--passphrase","{S}","--"]+MKMD,"value",PW,PW2,multi=True)
c("mnemonic verify-bundle --ms1",[m,"verify-bundle",A,"--network","mainnet","--template","bip84","--ms1","{S}","--"]+MKMD,"value",MS1,"ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7g")
# import-wallet
c("mnemonic import-wallet --ms1",[m,"import-wallet",A,"--blob",BLOB,"--format","bsms","--json","--ms1","{S}"],"value",T24_MS1,MS1)
c("mnemonic import-wallet --slot @0.phrase=",[m,"import-wallet",A,"--blob",BLOB,"--format","bsms","--json","--slot","@0.phrase={S}"],"node",T24.strip(),P)
# electrum-decrypt
c("mnemonic electrum-decrypt --decrypt-password",[m,"electrum-decrypt",A,"--ciphertext","ABEiM0RVZneImaq7zN3u/zY0181f7qAY/NWiVQFLdHE=","--decrypt-password","{S}"],"value","test-password","wrong-password")
# xpub-search
for mode,extra in [("path-of-xpub",["--target-xpub",XPUB84]),("account-of-descriptor",["--descriptor",f"wpkh([73c5da0a/84'/0'/0']{XPUB84}/0/*)"])]:
    c(f"mnemonic xpub-search {mode} --phrase",[m,"xpub-search",mode,A]+extra+["--phrase","{S}"],"value",P,P2)
    c(f"mnemonic xpub-search {mode} --ms1",[m,"xpub-search",mode,A]+extra+["--ms1","{S}"],"value",MS1,"ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7g")
    c(f"mnemonic xpub-search {mode} --passphrase",[m,"xpub-search",mode,A]+extra+["--phrase",P,"--passphrase","{S}"],"value","hunter2-passphrase",PW2,multi=True)
c("mnemonic xpub-search passphrase-of-xpub --passphrase",[m,"xpub-search","passphrase-of-xpub",A,"--target-xpub",XPUB84_PW,"--phrase",P,"--passphrase","{S}"],"value","hunter2-passphrase",PW2,multi=True)
c("mnemonic xpub-search passphrase-of-xpub --phrase",[m,"xpub-search","passphrase-of-xpub",A,"--target-xpub",XPUB84_PW,"--passphrase","hunter2-passphrase","--phrase","{S}"],"value",P,P2,multi=True)
# slip39
def split_check_slip39(o):
    shares=[l for l in o.splitlines() if l.strip() and not l.startswith("#")][:2]
    if len(shares)<2: return o
    return _sp.run([m,"slip39","combine",A]+sum([["--share",x] for x in shares],[]),capture_output=True,text=True).stdout
c("mnemonic slip39 split --from phrase=",[m,"slip39","split",A,"--from","phrase={S}","--group-threshold","1","--group","3,2"],"node",P,P2); C[-1]["check"]=split_check_slip39
c("mnemonic slip39 split --passphrase",[m,"slip39","split",A,"--from","phrase="+P,"--group-threshold","1","--group","3,2","--passphrase","{S}"],"value",PW,PW2,multi=True)
C[-1]["check"]=lambda o: _sp.run([m,"slip39","combine",A,"--passphrase","hunter2-passphrase"]+sum([["--share",x] for x in [l for l in o.splitlines() if l.strip() and not l.startswith("#")][:2]],[]),capture_output=True,text=True).stdout
c("mnemonic slip39 combine --share",[m,"slip39","combine",A,"--share","{S}","--share",S39[1]],"value",S39[0],S39[2],multi=True)
c("mnemonic slip39 combine --passphrase",[m,"slip39","combine",A,"--share",S39[0],"--share",S39[1],"--passphrase","{S}"],"value",PW,PW2,multi=True)
# ms-shares
def check_msshares(o):
    sh=[l.strip() for l in o.splitlines() if l.strip().startswith("ms1")][:2]
    if len(sh)<2: return o
    return _sp.run([m,"ms-shares","combine",A,"--share",sh[0],"--share",sh[1],"--to","phrase"],capture_output=True,text=True).stdout
c("mnemonic ms-shares split --from phrase=",[m,"ms-shares","split",A,"--from","phrase={S}","--threshold","2","--shares","2"],"node",P,P2); C[-1]["check"]=check_msshares
MSSH=["ms127776qw8m0z2uqfrdpxhc2q7v88ys6l25qum46xfaug68t2","ms127776p47az8mrqts9hf0dmqyj77w32nmlqxsl6m0f3zgdar"]
c("mnemonic ms-shares combine --share",[m,"ms-shares","combine",A,"--share","{S}","--share",MSSH[1],"--to","phrase"],"value",MSSH[0],"ms127776qw8m0z2uqfrdpxhc2q7v88ys6l25qum46xfaug68t3",multi=True)
# ms split / combine / hashlock
def check_mssplit(o):
    sh=[l.replace(" ","") for l in o.splitlines() if l.startswith("ms1")][:2]
    if len(sh)<2: return o
    return _sp.run([s,"combine",A,"--"]+sh,capture_output=True,text=True).stdout
c("ms split --phrase",[s,"split",A,"--threshold","2","--shares","2","--phrase","{S}"],"value",P,P2); C[-1]["check"]=check_mssplit
c("ms split --hex",[s,"split",A,"--threshold","2","--shares","2","--hex","{S}"],"value",ENT,ENT2); C[-1]["check"]=check_mssplit
MSP=["ms12ec7pqsf34k729gl6yrsfk9q244kzsx62s9krxwywxv4jav","ms12ec7pp3txgcymeun2ws3tceqmggc83f2m3l2cj299w9l9vr"]
c("ms combine <shares> (first)",[s,"combine",A,"--","{S}",MSP[1]],"pos",MSP[0],"ms12ec7pqsf34k729gl6yrsfk9q244kzsx62s9krxwywxv4jaw",multi=True)
c("ms hashlock --hashlock-phrase",[s,"hashlock",A,"--kind","sha256","--hashlock-phrase","{S}"],"value","correct horse battery staple","another phrase entirely")
c("ms hashlock --hex",[s,"hashlock",A,"--kind","sha256","--hex","{S}"],"value","11"*32,"22"*32)
