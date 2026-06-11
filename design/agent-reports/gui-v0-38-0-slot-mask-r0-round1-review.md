# R0 round-1 architect review — SPEC_gui_v0_38_0_slot_secret_mask (2026-06-10)

Reviewer: Fable 5 architect agent (R0, pre-implementation). master @ 54c13c3. Verdict: RED (0 Critical / 2 Important / 5 Minor). Review verbatim below (abridged; full evidence in transcript).

---

## Critical
None.

## Important

**I1 — T3 must be the all-variants `is_secret_bearing() == slot_subkey_is_secret()` equality; there is a real, currently-unpinned split-brain.** The fix gates render-mask + zeroize on `SlotSubkey::is_secret_bearing()` (slot_editor.rs:82-92, local matches!), while persistence-redaction (persistence.rs:109) + `slot_subkey_is_secret()` (secrets.rs:155) gate on the toolkit-imported `SECRET_SLOT_SUBKEYS`. NO test pins them equal across all variants: `secrets.rs:372` spot-checks 8 of 10, **OMITS Seedqr and Ms1** (the drift-prone added-later variants) and asserts vs hardcoded membership not the predicate; `argv_assembler_slot.rs:203` compares is_secret_bearing to persistable_subkeys (itself `!is_secret_bearing`-derived → tautology). After this cycle a divergence = security split-brain (secret per taxonomy → redacted at persist, but non-secret per local enum → rendered plaintext + not zeroized). **Fix:** T3 = `for sk in SlotSubkey::ALL { assert_eq!(sk.is_secret_bearing(), secrets::slot_subkey_is_secret(sk), "split-brain at {sk:?}") }` — covers Seedqr/Ms1; the single assertion keeping the mask/zeroize set == the redaction set.

**I2 — T2 must be a hard `Role::PasswordInput` assertion, not a "may not be queryable" fallback.** egui registers `.password(true)` TextEdits as `accesskit::Role::PasswordInput`, and the repo already queries exactly this (`repeating_secret_rows.rs:17` doc + :99/:108/:144/:373 `get_*_by_role(Role::PasswordInput)`). So T2: a secret-bearing row (Phrase) → `query_by_role(PasswordInput).is_some()`; a non-secret row (Xpub) → `.is_none()` (the discriminating negative); verified RED without `.password(true)` (a plain edit registers `Role::TextInput`), mirroring v0.37.0's scratch-revert discipline. Drop the pure-logic/smoke fallback (the slot_editor_path_hint precedent is wrong — hint_text isn't queryable; password role demonstrably is).

## Minor
- **M1:** render restructure correct — gate `is_secret_bearing()` FIRST (password edit), else the existing `(Path,Some(hint))|_` match; Path ∉ secret so the arms are mutually exclusive (.password never combines with hint_text). Add a one-line comment noting Path∉secret.
- **M2:** `impl Zeroize for String` exists (zeroize-1.8.2, alloc feature on; SecretLineEdit already calls String::zeroize at secret_widget.rs:83) — `use zeroize::Zeroize; row.value.zeroize()` compiles, no `.as_mut_vec()`. Order (zeroize then remove) borrow-sound (remove runs after the iter_mut loop). Caveat to DOCUMENT (not fix): String::zeroize scrubs the current buffer only, not prior reallocations (same allocator-residue limit as FOLLOWUP `gui-secret-buffer-allocator-residue`).
- **M3:** extract a `SlotRow::zeroize_if_secret(&mut self)` (empties value for secret rows, untouched for non-secret) called from a free `remove_row(state, i)` before `rows.remove(i)` — the cleanest testable seam; T1 asserts `value.is_empty()` post for a secret row + untouched for non-secret (discriminating).
- **M4:** SemVer MINOR correct (render-behavior change, no flag surface → no schema_mirror); persistence-already-safe re-verified (persistence.rs:105-111 drops secret-subkey rows); version sites complete; FOLLOWUPS resolve note should add the zeroize-on-remove half.
- **M5:** no toolkit companion / cross-repo lockstep (GUI-local render/zeroize, no flag/gui-schema delta).

## Verdict
**YELLOW — 0 Critical / 2 Important.** Design correct + deps verified. Fold I1 (all-variants T3 equality — closes the unpinned split-brain) + I2 (hard Role::PasswordInput T2) + the actionable minors (M1/M2 comments, M3 seam), re-dispatch.
