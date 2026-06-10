# R0 review — BRAINSTORM_node_tree_builder — round 1
**Verdict: YELLOW**

Grounding largely solid — all five path productions, the stdin path, --emit-spec, the round-trip, and the xprv heuristic verified live. Two locked-decision claims factually wrong (mode-mutex leak-proofness; exit-2⇒envelope); two §0 corrections (payload count = 9; envelope-bounds overstatement).

## Critical
**C1.** "Disabled suppresses argv → stale values can't leak" is FALSE for the 9 archetype param flags: disabling --spec/--archetype suppresses only those two; declared params emit whenever not Hidden, and all carry requires=archetype toolkit-side → every tree-mode run with stale archetype params fails with a confusing clap error. The tree-mode arm must Hide all 9 params AND the render dispatch must be mode-aware (active_archetype reads only the dropdown — the param form would still render in tree mode).
**C2.** Exit 2 does NOT imply a parseable envelope: bad-arity specs (= every partially-built tree) and schema_version:2 exit 2 with EMPTY stdout + stderr text (BuildDescriptorSpec is exit-class 2, error.rs:503). Parse contract: "stdout parses as {diagnostics:[…]} → node view; else surface stderr in the global strip."

## Important
**I1.** "only 8 payload shapes" — the list (and the live schema) has NINE.
**I2.** The cap bounds keys+hashes (≤12 @ 1 timelock-state, exact) but NOT node count/depth (depth-16 chain passes) — the §3 depth cap is load-bearing; add a UI posture for pathological-but-valid trees.
**I3.** Pin sources thinner than claimed: no thresh.subs[i] literal exists in the toolkit (GUI authors its own); keys[i] lives in cli tests not gate tests; the LIVE parity gate is the real tether — probe validated all five productions deep-nested (root.thresh.subs[2].andor[2].multi.keys[0] etc.).
**I4.** Validate parsed-view lifetime dangling: storage location, last_run interplay, and INVALIDATION — sibling removal shifts [i] indices so stale tints mis-attach (a correctness item in a funds tool); cheapest = clear-diagnostics-on-any-tree-mutation.
**I5.** Pin BOTH spec_schema_version AND supported_doc_schema_version (the mirror copies the grammar = spec_schema_version axis).
**I6.** stdin discipline: write_all → DROP ChildStdin → wait_with_output (undropped handle deadlocks — the toolkit reads to EOF); tolerate BrokenPipe (degrade to collect-output); state the ~2KB ≪ pipe-buffer size assumption that licenses skipping a threaded writer.

## Minor
**M1.** xprv heuristic verified for GUI reuse (Yprv/[origin]tprv fire; all-caps not a real encoding). **M2.** digests correctly excluded from redaction (state considered-and-rejected); FormState has no Clone — the redaction struct literal is the compile-time forcing fn, name it. **M3.** fixture immutability is CYCLE-scoped — gate (iii)'s live exit-0 run is the staleness tether; note it. **M4.** tree-mode Copy is misleading until P3 (--spec - hangs pasted into a shell) — annotate/suppress. **M5.** cite tightening (envelope cmd/build_descriptor.rs:367, exit-2 :326-327 + error.rs:503).

## Ground-truth audit
0.1 OK; 0.2 OK; 0.3 CORRECTED (C2/M5); 0.4 OK; 0.5 OK (complete, probed); 0.6 CORRECTED (9 shapes); 0.7 OK; 0.8 OK (+ no-Clone note); 0.9 PARTIALLY CORRECTED (I2).

## Empirical probes run
spec-schema dump (17 kinds, 9 shapes, both versions 1); emit-spec round-trip exit 0; planted-error tree → 4 diagnostics covering all 5 productions deep-nested; bad-arity + schema-v2 → exit 2 EMPTY stdout; preset params sentinel + flag field; xprv heuristic probes; depth-16 chain exit 0.
