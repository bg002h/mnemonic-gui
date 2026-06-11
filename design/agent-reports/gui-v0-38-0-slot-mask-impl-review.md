# Implementation review — GUI v0.38.0 slot secret mask (2026-06-10)

Reviewer: Fable 5 implementation-review agent (post-impl, pre-commit). Spec @ design/SPEC_gui_v0_38_0_slot_secret_mask.md (R0 GREEN r2). Verdict: GREEN (0 Critical / 0 Important / 2 Minor — m1 folded: T1b renamed to reflect its removal-half assertion; m2 line cite fixed). Review verbatim below.

---

## Critical / Important
None / None.

## Minor
- **m1 — T1b name overclaimed** (asserted removal, not scrubbing; the zeroize is discriminatingly pinned by T1 on the `zeroize_if_secret` seam it calls). FOLDED: renamed `t1b_remove_row_removes_the_row` + comment noting T1 pins the zeroize half.
- **m2 — `value: String` cite off-by-one** (:101→:102). Cosmetic; fixed in spec.

## Verdict
**GREEN — 0 Critical / 0 Important.**
1. **Render mask (slot_editor.rs:243-262):** `is_secret_bearing()` gated FIRST → `.password(true)`; else the exact `(Path,Some(hint))|_` match. Path∉secret holds (SECRET_SLOT_SUBKEYS lacks Path) → `.password` never combines with `hint_text`; Path∉secret comment present. `zeroize_if_secret` (:123) empties only secret rows; free `remove_row` (:132) zeroizes-then-removes, called at :269; `use zeroize::Zeroize` (:12); heap-residue caveat present. No state-shape change.
2. **Tests 5/5:** T2 RED-revert VERIFIED (gate→`if false` → t2 REDs, t2b stays green; restored sha256-identical). T3 non-vacuity VERIFIED (dropping Ms1 from is_secret_bearing → T3 REDs "split-brain at Ms1"; restored identical) — closes the real gap (secrets.rs:370 covers 8/10, omits Seedqr+Ms1). ALL.len()==10 tripwire present.
3. **Ritual:** CHANGELOG [0.38.0] claims verified (persistence-already-safe at persistence.rs:105-111); version 0.38.0 at Cargo.toml:3/Cargo.lock/README:42; FOLLOWUPS resolved; no flag surface → schema_mirror/drift unaffected.
4. **Full suite 55 ok / 0 failed** (pinned BINs); clippy `-D warnings` clean. Tree left as found.
