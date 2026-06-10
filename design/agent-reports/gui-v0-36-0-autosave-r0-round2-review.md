# R0 round-2 architect review — SPEC_gui_v0_36_0_autosave_atomic (2026-06-10)

Reviewer: Fable 5 architect agent (R0 round 2, post-fold verification). GUI 1244b7c. Verdict: YELLOW (0 Critical / 1 NEW Important I3 — fold-induced T1 drift / 2 Minor). Review verbatim below.

---

## Critical

None.

## Important

**I3 (new, fold-induced) — T1's pre-create-garbage cell contradicts D1's pinned tmp name: as literally specified it is RED against a CORRECT implementation, and the test-preserving "fix" reverses I1. Sync T1 to the PID-suffixed name.**
D1 now pins `state.json.<pid>.tmp`; T1 still says "pre-create `<path>.tmp` with garbage; post-save it must be GONE" and "no `.tmp` sibling remains". Under the literal reading the test pre-creates `state.json.tmp` — a name a correct PID-suffixed `save()` never touches — so after a correct implementation the garbage file (a) is NOT gone and (b) IS a remaining `.tmp` sibling: **both assertions fail, and the cell can never go green without deviating from the spec text.** The implementer's cheapest path to green that keeps the test verbatim is to drop the PID suffix from `save()` — silently reversing I1 while the suite passes. **Fold (one clause):** T1 must pre-create the EXACT D1 name, computed in-test as `format!("state.json.{}.tmp", std::process::id())` (test and `save()` share a process). RED-ability survives: current `save()` is a direct `fs::write` → PID-named garbage remains pre-impl → RED; consumed by rename post-impl → GREEN. The "no `.tmp` sibling remains" clause is then unambiguous as a same-process glob.

## Minor

**M8 (new) — Record that BTreeMap determinism is load-bearing for the content-compare debounce.** The skip is string equality on fresh serializations of freshly-collected maps — sound ONLY because `PersistedState`'s maps are `BTreeMap` (persistence.rs:49, :59). A HashMap refactor → nondeterministic key order → skip almost never fires → debounce silently dead, and T2 would NOT catch it (same-instance iteration order). One D2 parenthetical inoculates it.

**M9 (new, audit trail, no fold needed):** (a) `build_persisted_state(&self)` verbatim-move real (main.rs:999-1018 all &self reads; comments :986-998 move cleanly); (b) the D2 call shape compiles (disjoint field borrows); (c) T2's pre-delete leg implies a user-deleted state.json is not recreated until next change or exit — acceptable; (d) PID-reuse vs orphaned tmp non-issue (truncate before rename); (e) no collision with `state.json.bak`; (f) timer placement + keepalive anchors verified (main.rs:342-350, :170-181).

## Fold-verification

- **I1 — VERIFIED COMPLETE in D1 + Risks; did not propagate to T1 (→ I3).**
- **I2 — VERIFIED COMPLETE** ("NO remove-then-rename fallback, ever"; both-platform contract; sharing-violation degradation; ubuntu-only CI honesty). No residual hedge.
- **M1/M2/M4/M5/M6 — all VERIFIED** (exact construction; cache-after-Ok; 11 cells; power-loss contract; eframe-native non-goal w/ feature-OFF confirmation).
- No other fold-drift.

## Verdict

**YELLOW — 0 Critical / 1 Important (I3).** Fold the one T1 clause (+ optionally M8), re-dispatch for a trivially GREEN round 3.
