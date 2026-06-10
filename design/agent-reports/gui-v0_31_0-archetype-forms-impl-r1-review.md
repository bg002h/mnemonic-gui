# Impl review — GUI v0.31.0 archetype forms — round 1
**Verdict: GREEN** (0C/0I; 3 minors, none ship-blocking)

Reviewed commit `77a54a4` (13 files). Working tree clean and identical to the commit.

## Critical
None.
## Important
None.
## Minor
**m1 — Drift gate's PATH-fallback panics (not skips) against a stale pre-`build-descriptor` `mnemonic` on PATH.** NOT a SPEC violation — the implementation matches the mandated `schema_mirror.rs:608-620` pattern line-for-line; CI exports MNEMONIC_BIN. Optional hardening: clearer panic message in a future cycle.
**m2 — Deviation 2 (per-frame empty-row seed via synthesized `required`) is conformant but under-documented.** Behaviorally safe (empty rows skip both emission gates); tests account for it; one module-doc sentence would close the gap. Not blocking.
**m3 — (speculative, pre-existing class)** hand-crafted both-sides-of-the-mutex persisted state could render the param form under a Disabled dropdown; unreachable via the live UI (cell_13 mutex). Note-only.

## SPEC-conformance checklist
1. §1 transcription VERIFIED against the live 0.52.0 binary (all 5 archetypes, param order, summaries verbatim; decaying's non-obvious 8-param order exact). 4 self-consistency units via the public SCHEMA route.
2. §2 gate byte-pattern-identical skip-if-absent; field-by-field + ORDER + summary; GREEN with MNEMONIC_BIN.
3. §3 lib seam REAL — main.rs holds only guard + lib-helper name-set continue + dispatch (the helper IS the name-set: no parallel list to drift); partition unit pins 10 suppressed / 8 mode-independent; summary placement correct.
4. §4 all arms conformant: C1 >1-row arm (suppress_add + "(exactly 1)" + removable); FromRowCount bespoke-only (.max(1), ALL-rows; statics still Static(20)); m3 honored (min from static clone); RepeatAnnotation default None — repeating_rows 10/10 (v0.30.0 callers unchanged); hex hint non-blocking; FlagKind Copy / FlagSchema Clone side-effect-free.
5. §5 conditional discriminating: undeclared→Hidden appended; declared no entry; --spec Disabled unchanged (cell_13 green); the 5 per-archetype argv cells catch both Hidden-on-declared and emit-undeclared bugs.
6. Both reported deviations acceptable (argv cells sans populated --spec = mutex-vacuous state; the required-seed activation = the documented R0-r2 M-C rule, no argv pollution — empty rows skip at invocation.rs:61/:307-308).
7. Full suite 397/0 (1 pre-existing ignored); clippy -D warnings clean; verbose spot-runs 13/13, 1/1, 10/10.
8. CHANGELOG/version/README self-tag verified (readme_pin_coherence green).
9. Newly-wrong sweep: nothing (helper-driven name-set; O(5)/O(18) scans negligible; kittest harness omission immaterial).

## Empirical probes run
Live --spec-schema diff (exact match); clippy -D warnings clean; full suite 397/0; verbose spot-runs; negative no-MNEMONIC_BIN probe (m1); git diff/log clean.
