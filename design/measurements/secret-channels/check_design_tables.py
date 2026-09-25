"""The design's gate (no binaries needed):
  1. §A5 is REGENERATED from plan.py + channel_table.json + channel_policy.json right now
     (gen_plans.py) and must appear verbatim in the document (R1 Nm3);
  2. test_plan.py (the pure refusal / per-OS / permutation legs) must pass;
  3. every MEASURED block — table.md (§A2), missing_sources.md (§A2b), c1_evidence.out (§A3a),
     bytes.md (§A3c), t3.md (§A9), copy_evidence.md (§A7) — must appear verbatim; and
  4. plans_pure.json (T8, bindings AND payload values) must equal a regeneration.
With --fill TEMPLATE it first writes the document from a template holding {{A2_TABLE}} etc."""
import os, re, subprocess, sys
HERE = os.path.dirname(os.path.abspath(__file__))
DOC = os.path.join(HERE, "..", "..", "DESIGN_secret_channels_and_new_forms.md")
sys.path.insert(0, HERE)
import gen_plans
def rd(f): return open(os.path.join(HERE, f)).read().rstrip("\n")
import json
_rows = gen_plans.generate()
a5 = gen_plans.a5_markdown(_rows).rstrip("\n")
# T8's parity file (bindings AND payload values, R2 NI3) must equal a regeneration from plan.py.
_t8_stale = json.load(open(os.path.join(HERE, "plans_pure.json"))) != json.loads(json.dumps(_rows))
BLOCKS = {"A2_TABLE": rd("table.md"), "NO_ENTRY": rd("missing_sources.md"), "C1_EVIDENCE": rd("c1_evidence.out"),
          "BYTES": rd("bytes.md"), "A5_PLANS": a5, "T3_EVIDENCE": rd("t3.md"),
          "COPY_EVIDENCE": rd("copy_evidence.md")}
if len(sys.argv) == 3 and sys.argv[1] == "--fill":
    doc = open(sys.argv[2]).read()
    for k, v in BLOCKS.items():
        doc = doc.replace("{{" + k + "}}", v)
    open(DOC, "w").write(doc)
doc = open(DOC).read()
stale = [k for k, v in BLOCKS.items() if v not in doc]
if _t8_stale:
    stale.append("plans_pure.json (T8 parity file) differs from plan.py")
if re.search(r"\{\{[A-Z0-9_]+\}\}", doc):
    stale.append("unfilled placeholder")
t = subprocess.run([sys.executable, os.path.join(HERE, "test_plan.py")], capture_output=True, text=True, cwd=HERE)
print("stale blocks:", stale if stale else "none", "| test_plan.py:", t.stdout.strip().splitlines()[-1])
sys.exit(1 if stale or t.returncode else 0)
