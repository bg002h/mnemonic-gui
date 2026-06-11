# R0 round-2 architect review — SPEC_gui_v0_38_0_slot_secret_mask (2026-06-10)

Reviewer: Fable 5 architect agent (R0 round 2, post-fold). master @ 54c13c3. Verdict: GREEN (0 Critical / 0 Important / 4 non-blocking Minor). Review verbatim below (abridged).

---

## Critical / Important
None / None. Both round-1 Importants folded correctly; all dependency claims verified against the live pinned source.

## Minor
- **m1 — T3 deref:** `for sk in SlotSubkey::ALL` yields `&SlotSubkey`; `slot_subkey_is_secret(sk: SlotSubkey)` takes by value → needs `slot_subkey_is_secret(*sk)` (or `for &sk in ALL`). Mechanical; flagged so the TDD-red step isn't a spurious type error.
- **m2 — `SlotSubkey::ALL` EXISTS** (slot_editor.rs:54-65, 10 variants — drop the "add if absent" hedge). But ALL has NO exhaustiveness gate (unlike the toolkit enum's macro at slot_input.rs:395). A future 11th variant forgotten in ALL → T3 silently skips it → the split-brain reopens one rung up. Add `assert_eq!(SlotSubkey::ALL.len(), 10)` tripwire to T3, or file `gui-slotsubkey-all-exhaustiveness-ungated`.
- **m3 — T2 needs a NEW harness:** `full_form_harness` skips `--slot` (repeating_rows.rs:145). Stand up `Harness::new_ui_state(|ui, st: &mut SlotState| slot_editor::render(ui, st, None))` — `render` is pub + directly callable.
- **m4 — Problem-section off-by-one:** `value: String` is slot_editor.rs:101 not :100. Cosmetic; every other citation verified exact.

## Fold-verification
- **I1 (all-variants T3) FOLDED, correct:** the split-brain is real-and-unpinned (secrets.rs:372 omits Seedqr+Ms1; argv_assembler_slot.rs:203 is a tautology). Today both predicates agree on all 10 (local 6-set == imported SECRET_SLOT_SUBKEYS, verified vs pinned toolkit 87c33c5) → T3 GREEN immediately, correctly framed as a pin.
- **I2 (hard PasswordInput T2) FOLDED, correct:** `query_by_role(Role::PasswordInput).is_some()/.is_none()` is the exact repo API (repeating_secret_rows.rs:84/:373); .password→PasswordInput, plain→TextInput established. Needs the m3 harness.
- **M1/M2/M3 FOLDED:** Path∉secret comment; zeroize heap-residue caveat (→ FOLLOWUP gui-secret-buffer-allocator-residue, exists); SlotRow::zeroize_if_secret + free remove_row seam borrow-sound (the inline remove runs after the iter_mut loop closes).
- **Ritual/SemVer/persistence-safe/no-cross-repo all re-verified.**

## Verdict
**GREEN — 0 Critical / 0 Important.** 4 non-blocking minors (m1 *sk, m2 ALL tripwire, m3 new harness, m4 line). Implementation may proceed.
