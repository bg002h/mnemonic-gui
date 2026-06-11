# SPEC — GUI v0.38.0: mask secret-bearing slot values + zeroize on remove

**Cycle:** mnemonic-gui v0.38.0 (MINOR) · **Source SHA:** `54c13c3` (= v0.37.0) · **Recon:** `cycle-prep-recon-secret-exposure-cluster.md` §ITEM 2.
**Resolves:** `slot-secret-values-rendered-unmasked` (the self-contained item of the secret-exposure cluster; Items 1+3 are a separate cycle).

## Problem (verified)

`SlotEditor::render` (`src/form/slot_editor.rs:219-228`) branches only on `(SlotSubkey::Path, Some(hint))` vs a `_` fallback `ui.text_edit_singleline(&mut row.value)` — **no branch on `row.subkey.is_secret_bearing()`** (`:82-92`: Phrase/Seedqr/Entropy/Ms1/Wif/Xprv). So a secret-bearing slot value renders in PLAINTEXT on screen. And row removal (`:234-235` `state.rows.remove(i)`) drops a `SlotRow` whose `value: String` (`:102`) is a plain heap allocation — no zeroize. (Persistence is already safe: `redact_for_persistence` drops secret-subkey rows — recon-confirmed; this is render-side + in-memory residue only.)

## Design — keep the `String` storage; mask at render + zeroize at remove

`SecretLineEdit` is NOT reusable here (non-Clone, `#[serde(skip)]`; `SlotRow` is `Clone + Serialize`) — swapping the type would break the slot round-trip and every `value` consumer. So:

1. **Render mask** (slot_editor.rs:219-228): gate `row.subkey.is_secret_bearing()` FIRST → `egui::TextEdit::singleline(&mut row.value).password(true)`; ELSE the existing `(Path,Some(hint))|_` match. Path ∉ secret (R0-r1 M1) so the arms are mutually exclusive (`.password` never combines with `hint_text`) — one-line comment noting Path∉secret so a future reader doesn't merge them.
2. **Zeroize on remove** (slot_editor.rs:234-235): `use zeroize::Zeroize` (`impl Zeroize for String` present — SecretLineEdit already uses it at secret_widget.rs:83; no `.as_mut_vec()`); the `remove_row` seam (T1) calls `zeroize_if_secret` then `rows.remove(i)`. Borrow-sound (remove runs after the iter_mut loop). CAVEAT to comment (R0-r1 M2, do NOT fix): `String::zeroize` scrubs the CURRENT buffer only, not prior reallocations — same best-effort allocator-residue limit as FOLLOWUP `gui-secret-buffer-allocator-residue`.
3. Non-goals here: paste-warn on slot edits (depends on Item 3), the SecretLineEdit migration (state-shape change, unjustified).

## Tests (TDD)

- **T1 (zeroize-on-remove, pure logic; R0-r1 M3 seam):** add `SlotRow::zeroize_if_secret(&mut self)` (empties `value` for `is_secret_bearing()` rows, untouched otherwise) + a free `fn remove_row(state: &mut SlotState, i: usize)` that calls it before `rows.remove(i)`; the egui closure calls `remove_row`. T1: a `Phrase` row with a known value → `zeroize_if_secret` → assert `value.is_empty()`; a non-secret (`Xpub`) row → `zeroize_if_secret` → assert `value` UNCHANGED (discriminating). Directly asserts the post-condition without egui.
- **T2 (render mask, kittest — hard assertion, R0-r1 I2):** egui registers `.password(true)` TextEdits as `accesskit::Role::PasswordInput` and the repo already queries it (`repeating_secret_rows.rs:17` + `:99/:108/:144` via `get_*_by_role(Role::PasswordInput)`). Render via a NEW harness — `Harness::new_ui_state(|ui, st: &mut SlotState| slot_editor::render(ui, st, None))` (R0-r2 m3; `full_form_harness` skips --slot). A secret-bearing row (Phrase) → assert `query_by_role(Role::PasswordInput).is_some()`; and an all-non-secret-row state (Xpub) → assert `.is_none()` (the discriminating negative). Verify RED without the `.password(true)` branch (a plain edit registers `Role::TextInput`), mirroring v0.37.0's scratch-revert discipline. NO pure-logic/smoke fallback.
- **T3 (predicate split-brain pin, R0-r1 I1) — LOAD-BEARING:** the fix gates render/zeroize on `is_secret_bearing()` while persistence-redaction gates on the toolkit-imported `SECRET_SLOT_SUBKEYS` (via `slot_subkey_is_secret`). NO existing test pins them equal across ALL variants (`secrets.rs:372` omits Seedqr+Ms1). Assert the full equality: `for sk in SlotSubkey::ALL { assert_eq!(sk.is_secret_bearing(), secrets::slot_subkey_is_secret(*sk), "split-brain at {sk:?}: render/zeroize gate disagrees with the persistence gate") }`. This single assertion keeps the mask/zeroize set == the redaction set. `SlotSubkey::ALL` EXISTS (slot_editor.rs:54, 10 variants); add `assert_eq!(SlotSubkey::ALL.len(), 10)` as an exhaustiveness tripwire (R0-r2 m2 — ALL itself is not gated, so a forgotten future variant would silently skip T3).
- Existing slot tests + the full suite green; no schema change.

## Ritual

CHANGELOG `[0.38.0]`; version bump (Cargo.toml + Cargo.lock + README self-pin); FOLLOWUPS resolve `slot-secret-values-rendered-unmasked` (note: render-mask + zeroize-on-remove; persistence was already safe). No toolkit pin / schema_mirror / manual impact. SemVer MINOR (user-visible: secret slot values now masked).

## Non-goals

Items 1 (preview/copy/modal masking) + 3 (paste-warn wiring) — the coupled cycle (user chose: mask+reveal-copy / wire paste-warn). The SecretLineEdit-per-row migration.
