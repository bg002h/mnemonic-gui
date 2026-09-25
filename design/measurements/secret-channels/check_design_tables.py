"""Assert DESIGN_secret_channels_and_new_forms.md carries every generated block verbatim:
table.md (§A2), missing_sources.md (§A2b), c1_evidence.out (§A3a), and both halves of
plans.md (§A5 plans, §A9 T3' evidence). Exit 1 names the stale block. With --fill TEMPLATE,
write the document from a template holding {{A2_TABLE}} etc. instead."""
import os, sys
HERE = os.path.dirname(os.path.abspath(__file__))
DOC = os.path.join(HERE, "..", "..", "DESIGN_secret_channels_and_new_forms.md")
def rd(f): return open(os.path.join(HERE, f)).read().rstrip("\n")
plans = rd("plans.md")
a5, t3 = plans.split("\n\n", 1)
BLOCKS = {"A2_TABLE": rd("table.md"), "NO_ENTRY": rd("missing_sources.md"),
          "C1_EVIDENCE": rd("c1_evidence.out"), "A5_PLANS": a5, "T3_EVIDENCE": t3}
if len(sys.argv) == 3 and sys.argv[1] == "--fill":
    doc = open(sys.argv[2]).read()
    for k, v in BLOCKS.items():
        doc = doc.replace("{{" + k + "}}", v)
    open(DOC, "w").write(doc)
doc = open(DOC).read()
stale = [k for k, v in BLOCKS.items() if v not in doc]
print("stale blocks:", stale if stale else "none")
sys.exit(1 if stale else 0)
