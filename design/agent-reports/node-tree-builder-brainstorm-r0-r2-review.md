# R0 review — BRAINSTORM_node_tree_builder — round 2
**Verdict: YELLOW** (0C / 1I / 3M — one residual leak path the C1 fold under-counts; everything else folded clean)

## Round-1 fold verification
All 13 folds verified applied + mutually consistent. C1↔§1.2/§1.6 consistent (Hidden/Disabled both suppress argv invocation.rs:160; ARCHETYPE_PARAM_FLAGS exactly 9; active_archetype reads only the dropdown — mode-aware dispatch is real). C2 stated twice, matching; all three legs live-confirmed (bad-arity → exit 2 stdout 0 BYTES; schema-v2 → stderr text; gate error → envelope). I1-I6, M1-M5 all verified (incl. the exact deep literal `root.thresh.subs[2].andor[2].multi.keys[0]` re-probed; note: node-level key errors attach to the NODE path — the gate-(ii) plant must use a keys-CLASS error like secret_key).

## Critical
None.

## Important
**I-1. `--emit-spec` is the TENTH `requires = "archetype"` edge and leaks in tree mode.** GUI declares it a plain Boolean; `Boolean(true)` emits the bare flag (invocation.rs:347-349). Checked in archetype mode then switching to tree → stale flag rides argv with --archetype suppressed → clap error on every run + it conflicts with the --json that Validate appends. The GUI's own comment counts "10 requires-archetype edges" (conditional.rs:635). Fix: the tree-mode arm hides every requires=archetype flag — the 9 params PLUS --emit-spec (10 total).

## Minor
**M-1.** §1.1 still says "the 8-shape fact" — stale vs the folded 9. **M-2.** §2.3 say "visibility-suppression (Disabled/Hidden)" not Disabled-only. **M-3.** normalize the production counting (4 child_paths arms + root + keys[i]; gate (ii) coverage list unambiguous).

## Empirical probes run
spec-schema (17/9/both-versions-1); bad-arity + schema-v2 + planted-tprv legs; node-vs-keys error-class split; depth-16 chain; --emit-spec refusal; "params" sentinel live; all line cites re-verified.

**Foundation assessment:** sound for the SPEC after the one-clause I-1 fold (+ ripples).
