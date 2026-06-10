# R0 review — SPEC_gui_v0_32_0_node_tree_builder — round 2
**Verdict: GREEN** (0C / 0I / 3M — all 12 round-1 folds verified applied without contradiction; the 3 new Minors are one-line pins)

## Round-1 fold verification
I1 ✓ (laws coherent w/ §4.3 + the §5 projection cell; no naive-law residue). I2 ✓ (all six classes re-probed byte-exact; type-class→root re-confirmed). I3 ✓. I4 ✓ (storage model matches v0.31.0 code exactly; migration leg holds). I5 ✓ (both strip cells). I6 ✓. M1-M6 ✓ (argv[0] runner contract confirmed; display_or; structural-emptiness w/ the k=0 gate backstop; Copy gated; import checks; both cells). §0/§6 anchors re-verified (gate.rs:281 heuristic; the 10-edge comment; install.sh at v0.31.1).

## Critical
None.
## Important
None.
## Minor
**M1.** The andor plant lacks a recipe and the naive guess VALIDATES (`andor(pk,older,pk)` exit 0!); the probed working recipe is `andor(pk, after, older)` → sigless_branch @ root.andor[1]. Pin it.
**M2.** Selector gestures half-specified: any non-Tree click clears `enabled` FIRST (focusing a non-rendered dropdown is a no-op otherwise); Generic-click sets the dropdown to "(none)" (scope the never-destroys sentence to values/params/tree-nodes; dropdown selection exempted).
**M3.** Fixed Validate argv diverges for `--allow` (it changes the VERDICT, probed: sigless + --allow → exit 0) — a deliberately-sigless tree shows permanently-red Validate while Run succeeds. Decide knowingly: pass --allow through, or annotate.

## Empirical probes run
17/9/both-1 schema; all six plants re-run byte-exact (+ the naive-andor validates trap); type-class→root; tpub/mixed network-agnostic Validate; --allow verdict flip; --emit-spec both legs. Stale release-build note: target/release is v0.48.0 — build fresh.
