# Phase B.1 SecretLineEdit Widget — R1

**Date:** 2026-05-12
**Reviewer:** feature-dev:code-reviewer (Sonnet 4.6)
**Scope:** Commit 62dcdf9 on branch v0_2
**Plan ref:** /home/bcg/.claude/plans/v0_2-mnemonic-gui.md §C Phase B.1; SPEC §3

## Verdict

**0 Critical / 2 Important / 1 Sub-threshold (N) — fold needed**

The core security invariant is correctly implemented: `Zeroizing<Vec<u8>>` primary buffer, `#[serde(skip)]` never-persist enforced by type, `assemble_argv` secret branch bypasses `state.values`, `zeroize_form_state` sweeps `secret_widgets`. One Important (I-1) requires a fold before the exit gate can clear 0C/0I; one Important (I-2) is a recommendation.

---

## Critical findings

None.

---

## Important findings

### I-1 — `should_confirm_run` coverage gap: secret_widgets path untested

**Confidence:** 87
**File:** `tests/secrets.rs`; `src/secrets.rs:85-89`; `src/schema/mod.rs:205-213`

`run_confirm_fires_when_passphrase_populated` is the only positive test for `should_confirm_run` + passphrase. It populates `--passphrase` via `FormState::from_pairs(...)` into `state.values`, not `state.secret_widgets`. In v0.2, the live runtime path puts secret flags exclusively in `secret_widgets`; `state.values` is intentionally bypassed for them. The test therefore exercises `has_value`'s `state.values` branch rather than the new `state.secret_widgets` branch. If the second branch of `has_value` were deleted, the test would still pass and the run-confirm modal would silently fail to fire when a user has typed a passphrase into the `SecretLineEdit` widget.

The `secret_class_flag_emitted_from_secret_widget_not_values_map` cell confirms the `assemble_argv` bypass correctly, but `should_confirm_run` is a distinct function and requires independent coverage.

**Required fold:** add `run_confirm_fires_when_passphrase_in_secret_widgets` to `tests/secrets.rs` that exercises the new branch.

---

### I-2 — `as_string()` wrapping contract is doc-only; type-level enforcement available

**Confidence:** 83
**File:** `src/form/secret_widget.rs:86-94`

`as_string()` is `pub` and returns `String`. Its doc comment mandates the caller wrap the result in `Zeroizing::new(...)`, and `assemble_argv` (`invocation.rs:73`) follows this correctly. However the type system cannot enforce the contract: any current or future caller can call `widget.as_string()` without wrapping and the compiler emits no diagnostic.

Changing the signature to `pub fn as_string(&self) -> Zeroizing<String>` enforces the contract at compile time at zero runtime cost. Required caller adjustments are minimal:

- `invocation.rs:73`: drop the `Zeroizing::new(...)` wrap (it becomes the return type).
- `_doctest_wrap_pattern()` in `secret_widget.rs` becomes redundant — remove.
- Tests calling `widget.as_string()` need `widget.as_string().as_str()` or `&*widget.as_string()` in `assert_eq!`.

This is a recommendation, not a hard blocker, since the sole production call site is correct. The type-level fix is worth making given the `pub` surface and multi-phase future use.

---

## Sub-threshold notes

### N-1 (carry-forward from A.3 R2) — SPEC §6 table cell 1 row not updated

**Confidence:** 95 (confirmed; no functional consequence)
**File:** Plan line 529

The A.3 R2 report named B.1 as the "natural candidate" to fold this documentation inconsistency. The B.1 commit did not fold it. Plan line 529 still reads the pre-I-2-fold aspirational assertion. Recommend fold at next plan touch (latest: Phase B.2).

---

## Deviation rulings

### Deviation (1) — `render_with_dispatch` as new function rather than in-place modification of `render`: ACCEPTED

Architecturally superior. `render` remains a pure `(ui, flag, &mut FlagValue) -> ()` renderer; `render_with_dispatch` owns all form-state-touching logic. `default_value_for_flag` migration to `widget::default_flag_value_for` removes a source-of-truth split. Borrow-checker correctness verified: `state.secret_widgets.entry(...).or_default()` captures a distinct `&mut SecretLineEdit` cleanly under Rust 2021 disjoint capture.

### Deviation (2) — `SecretLineEdit::from_text(&str)` constructor: ACCEPTED

Established precedent in `SecretBuffer::from_text`. Without it, B.1 unit tests would require an `egui_kittest::TestHarness` for every buffer-invariant assertion. Doc clearly notes caller responsibility for `&str` source.

### Deviation (3) — Pre-existing `never_loop` clippy fix folded in: ACCEPTED with note

The B.1 exit gate requires clippy clean, making this fix non-optional. Semantically identical change. A separate preparatory commit would have been preferable for audit clarity; commit message adequately attributes the origin.

---

## Hot-spot resolution

| # | Hot spot | Verdict |
|---|----------|---------|
| 1 | Transient `Vec<u8>` residue in `show()` | OK — error path unreachable via current callers; documented allocator-residue gap tracked in FOLLOWUPS `gui-secret-buffer-allocator-residue`. |
| 2 | `as_string()` Zeroizing wrap at call site | `invocation.rs:73` confirms wrap. See I-2 for type-level recommendation. |
| 3 | `has_value()` extension breakage check | No breakage — `cell_12` uses empty/non-secret states so the `secret_widgets` branch evaluates false. |
| 4 | `FormState::Clone` removal cascade | No callers — zero `.clone()` calls on `FormState` in `src/` or `tests/`. |
| 5 | `PersistedState::Clone` removal cascade | No callers — `redact_persisted_state` uses field-by-field construction; `redact_for_persistence` uses a struct literal. |
| 6 | `render_with_dispatch` borrow-checker | Clean under Rust 2021 disjoint capture. |
| 7 | Modal text byte-exact assertion | All three new substrings ("Zeroizing<Vec<u8>>", "zeroed on drop", "undo ring") present in `PASTE_WARN_MODAL_TEXT`. |
| 8 | `secret_class_flag_emitted_from_secret_widget_not_values_map` | Complete exercise of emit-side bypass. Gap exists on `should_confirm_run` side (I-1). |
| 9 | `secret_widgets_round_trip_never_persists_both_directions` | Adequate. |
| 10 | A.3 R2 N-1 carry-over | Not folded; carries as N-1 here. |

---

## Exit gate checklist

| Gate item | Status |
|-----------|--------|
| `secrets` cells pass (3 new) | PASS |
| `argv_assembler` cell passes (1 new) | PASS |
| `persistence` cell passes (1 new) | PASS |
| `widget_secret::cell_paste_warn_modal_trigger` GREEN | PASS |
| `cargo clippy -- -D warnings` clean | PASS — `never_loop` folded |
| `PersistedState: Clone` removed | PASS |
| `FormState: Clone` removed | PASS |
| `should_confirm_run` → `secret_widgets` path tested | FAIL — I-1 |
| 0C / 0I | NOT MET — pending I-1 fold |

---

Next action: Fold I-1 (add test cell), optionally fold I-2 (type-level enforcement), fold N-1 (plan line 529). Re-review as R2.
