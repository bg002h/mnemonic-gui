"""Every multi-secret shape the GUI's forms can produce for a measured input, plus the refusal
shapes. Each shape = the fixed (public) argv around the sources, the sources in argv order, and
an effect extractor. run_plans.py plans each one with plan.py and runs it."""
import json, os

HERE = os.path.dirname(os.path.abspath(__file__))
FX = os.path.join(HERE, "fixtures")

P = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
P2 = "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo wrong"
PW = "hunter2-passphrase"
MS1 = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f"
T24_MS1 = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqcwugpdxtfme2w"
ENT = "00000000000000000000000000000000"
WIF = "KyZpNDKnfs94vbrwhJneDi77V6jF64PWPF8x5cdJb8ifgg2DUc9d"
BIP38_A = "6PYP8fdoVaE3ThLmEnYcGo3nJeBqd8PvB7CRvTz3TX5L9ojoPHKCg7QXG6"
XPUB84_PW = "xpub6CPUCVp94gpNs3bS1eGiwSWwZMLNmfr2Uo1t5v8YtY4XVoxhUraBH7sRyVfgwSNCxRVpX1bDREtc5KriC7smtjYco8w57agUrg1nLawsNFG"
S39 = [l for l in open(os.path.join(FX, "s39.all")).read().splitlines() if l.strip()]
MSSH = ["ms127776qw8m0z2uqfrdpxhc2q7v88ys6l25qum46xfaug68t2", "ms127776p47az8mrqts9hf0dmqyj77w32nmlqxsl6m0f3zgdar"]
MSPLIT = ["ms12ec7pqsf34k729gl6yrsfk9q244kzsx62s9krxwywxv4jav", "ms12ec7pp3txgcymeun2ws3tceqmggc83f2m3l2cj299w9l9vr"]
BPJ = json.load(open(os.path.join(FX, "bundle_pw.json")))


def node(key, flag, prefix, v):
    return {"key": key, "form": "node", "flag": flag, "prefix": prefix, "value": v}


def val(key, flag, v):
    return {"key": key, "form": "value", "flag": flag, "value": v}


def pos(key, v):
    return {"key": key, "form": "pos", "flag": None, "value": v}


def group(key, vs):
    return {"key": key, "form": "group", "flag": None, "value": vs}


m, ms = "mnemonic", "ms"
SHAPES = []


# F-694: `sources` are listed in the FORM's order (the order `channels::assemble` finds them),
# because the planner is order-dependent by design (DESIGN §A4.3). T6's
# t6_every_shape_through_the_real_form_plans_as_the_pure_planner asserts it.
def shape(name, cli, sub, pre, sources, post=(), symmetric=(), expect="run"):
    """symmetric: the source-index pairs whose values the CLI combines order-independently
    (share sets). A swap between such a pair is output-invisible, by the math, not by accident."""
    SHAPES.append({"name": name, "cli": cli, "sub": sub, "pre": list(pre), "sources": sources,
                   "post": list(post), "symmetric": [tuple(x) for x in symmetric], "expect": expect})


XS = "mnemonic xpub-search "
shape("addresses phrase+passphrase", m, ["addresses"], ["--address-type", "p2wpkh", "--count", "1"],
      [node("mnemonic addresses --from phrase=", "--from", "phrase=", P), val("mnemonic addresses --passphrase", "--passphrase", PW)])
shape("restore phrase+passphrase", m, ["restore"], ["--template", "bip84"],
      [node("mnemonic restore --from phrase=", "--from", "phrase=", P), val("mnemonic restore --passphrase", "--passphrase", PW)])
shape("restore ms1+passphrase", m, ["restore"], ["--template", "bip84"],
      [node("mnemonic restore --from ms1=", "--from", "ms1=", MS1), val("mnemonic restore --passphrase", "--passphrase", PW)])
shape("derive-child phrase+passphrase", m, ["derive-child"], ["--application", "bip39", "--length", "12", "--index", "0"],
      [node("mnemonic derive-child --from phrase=", "--from", "phrase=", P), val("mnemonic derive-child --passphrase", "--passphrase", PW)])
shape("bundle slot+passphrase", m, ["bundle"], ["--network", "mainnet", "--template", "bip84"],
      [val("mnemonic bundle --passphrase", "--passphrase", PW), node("mnemonic bundle --slot @N.phrase=", "--slot", "@0.phrase=", P)])
shape("bundle wsh-multi 2 slots", m, ["bundle"], ["--network", "mainnet", "--template", "wsh-multi", "--threshold", "2"],
      [node("mnemonic bundle --slot @N.phrase=", "--slot", "@0.phrase=", P), node("mnemonic bundle --slot @N.phrase=", "--slot", "@1.phrase=", P2)])
shape("bundle wsh-multi 2 slots+passphrase", m, ["bundle"], ["--network", "mainnet", "--template", "wsh-multi", "--threshold", "2"],
      [val("mnemonic bundle --passphrase", "--passphrase", PW),
       node("mnemonic bundle --slot @N.phrase=", "--slot", "@0.phrase=", P), node("mnemonic bundle --slot @N.ms1=", "--slot", "@1.ms1=", T24_MS1)])
shape("convert phrase+passphrase", m, ["convert"], ["--to", "xpub", "--template", "bip84"],
      [node("mnemonic convert --from phrase=", "--from", "phrase=", P), val("mnemonic convert --passphrase", "--passphrase", PW)])
shape("convert wif+bip38-passphrase", m, ["convert"], ["--to", "bip38"],
      [node("mnemonic convert --from wif=", "--from", "wif=", WIF), val("mnemonic convert --bip38-passphrase", "--bip38-passphrase", PW)])
shape("convert bip38+bip38-passphrase", m, ["convert"], ["--to", "wif"],
      [node("mnemonic convert --from bip38=", "--from", "bip38=", BIP38_A), val("mnemonic convert --bip38-passphrase", "--bip38-passphrase", PW)])
for mode, extra in [("path-of-xpub", ["--target-xpub", XPUB84_PW]), ("passphrase-of-xpub", ["--target-xpub", XPUB84_PW]),
                    ("account-of-descriptor", ["--descriptor", f"wpkh({XPUB84_PW}/0/*)"])]:
    shape(f"xpub-search {mode} phrase+passphrase", m, ["xpub-search", mode], extra,
          [val(XS + mode + " --phrase", "--phrase", P), val(XS + mode + " --passphrase", "--passphrase", PW)])
    shape(f"xpub-search {mode} ms1+passphrase", m, ["xpub-search", mode], extra,
          [val(XS + mode + " --ms1", "--ms1", MS1), val(XS + mode + " --passphrase", "--passphrase", PW)])
shape("silent-payment secret+passphrase", m, ["silent-payment"], [],
      [val("mnemonic silent-payment --passphrase", "--passphrase", PW), val("mnemonic silent-payment --secret", "--secret", P)])
shape("slip39 split phrase+passphrase", m, ["slip39", "split"], ["--group-threshold", "1", "--group", "3,2"],
      [node("mnemonic slip39 split --from phrase=", "--from", "phrase=", P), val("mnemonic slip39 split --passphrase", "--passphrase", PW)])
shape("slip39 combine 2 shares+passphrase", m, ["slip39", "combine"], [],
      [val("mnemonic slip39 combine --share", "--share", S39[0]), val("mnemonic slip39 combine --share", "--share", S39[1]),
       val("mnemonic slip39 combine --passphrase", "--passphrase", PW)], symmetric=[(0, 1)])
shape("slip39 combine 2 shares", m, ["slip39", "combine"], [],
      [val("mnemonic slip39 combine --share", "--share", S39[0]), val("mnemonic slip39 combine --share", "--share", S39[1])], symmetric=[(0, 1)])
shape("seed-xor combine 2 shares", m, ["seed-xor", "combine"], ["--shares", "2"],
      [node("mnemonic seed-xor combine --share phrase=", "--share", "phrase=", P), node("mnemonic seed-xor combine --share phrase=", "--share", "phrase=", P2)],
      symmetric=[(0, 1)])
shape("ms-shares combine 2 shares", m, ["ms-shares", "combine"], ["--to", "phrase"],
      [val("mnemonic ms-shares combine --share", "--share", MSSH[0]), val("mnemonic ms-shares combine --share", "--share", MSSH[1])], symmetric=[(0, 1)])
shape("import-wallet 2 cosigner ms1", m, ["import-wallet"], ["--blob", os.path.join(FX, "blob-2of2.bsms"), "--format", "bsms", "--json"],
      [val("mnemonic import-wallet --ms1", "--ms1", T24_MS1), val("mnemonic import-wallet --ms1", "--ms1", MS1)])
shape("verify-bundle ms1+slot+passphrase", m, ["verify-bundle"],
      ["--network", "mainnet", "--template", "bip84", "--mk1"] + BPJ["mk1"] + ["--md1"] + BPJ["md1"],
      [val("mnemonic verify-bundle --passphrase", "--passphrase", PW),
       val("mnemonic verify-bundle --ms1", "--ms1", BPJ["ms1"][0]), node("mnemonic verify-bundle --slot @N.phrase=", "--slot", "@0.phrase=", P)])
shape("ms combine share group", ms, ["combine"], [], [group("ms combine <shares>", MSPLIT)])
shape("ms verify phrase+ms1", ms, ["verify"], [], [val("ms verify --phrase", "--phrase", P), pos("ms verify <ms1>", MS1)])
shape("ms derive ms1+passphrase", ms, ["derive"], [], [val("ms derive --passphrase", "--passphrase", PW), pos("ms derive <ms1>", MS1)])
shape("ms derive phrase+passphrase", ms, ["derive"], [], [val("ms derive --phrase", "--phrase", P), val("ms derive --passphrase", "--passphrase", PW)],
      expect="refuse")
shape("ms derive hex+passphrase", ms, ["derive"], [], [val("ms derive --hex", "--hex", ENT), val("ms derive --passphrase", "--passphrase", PW)],
      expect="refuse")


def effect(stdout):
    """The funds-relevant line(s) a human would compare: fingerprints, keys, addresses, phrases,
    digests, verdicts. Reported beside the whole-stdout equality."""
    keys = ('"descriptor":', "result:", "match:", "master fingerprint", "master_fingerprint:", "fingerprint", "wif:", "bip38:", "xpub:", "sp1", "bc1", "hash:",
            "entropy:", "phrase:", "descriptor", "OK:", "mk1q")
    lines = [l.strip() for l in stdout.splitlines() if l.strip()]
    for k in keys:
        for l in lines:
            if k in l:
                return l[:90]
    return stdout.strip().splitlines()[0][:90] if stdout.strip() else ""
