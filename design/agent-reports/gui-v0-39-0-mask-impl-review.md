# Implementation Review — GUI v0.39.0 (mask secret values in on-screen command display)

Reviewed the UNCOMMITTED working-tree implementation against the R0-GREEN spec, before commit.

**Verdict: 0 Critical / 0 Important / 2 Minor.**

## Critical
None.

## Important
None.

## Minor

### M1 — Modal loop lacks a `debug_assert_eq` for mask/argv length parity (confidence 82)
`render_copy_command_masked` has a `debug_assert_eq!(argv.len(), mask.len())`; the confirm-modal's inline token loop uses the same `mask.get(i).copied().unwrap_or(false)` fail-open fallback but no corresponding assert. Correct-by-construction (mask travels in the same `PendingConfirm` tuple), so not a practical leak — but a future truncation refactor would fail-open silently in the modal. Fix: add `debug_assert_eq!(argv.len(), mask.len(), ...)` before the modal `for` loop. **APPLIED.**

### M2 — Ritual items not yet applied (expected pre-commit)
Cargo.toml + Cargo.lock 0.38.0→0.39.0; CHANGELOG [0.39.0]; README self-pin :42; FOLLOWUPS resolve `run-confirm-and-preview-show-secrets-cleartext`; file the deferred FOLLOWUPs. Done as the ritual step.

## Positive findings (security property verified)
- **Mask completeness — all four sources covered, correct-by-construction.** Secret Text (`mask.push(true)` on value), secret slot (`to_slot_argv_masked` gates on `is_secret_bearing()`, `--slot` name = false, watch-only = false), secret positional (`mask.push(true)`), composite (`flag_is_secret(flag) || node_type_is_secret(node)` — covers `--share` and `--from phrase=`, while `--from xpub=` = false).
- **`mask.len() == argv.len()` structural invariant:** every `argv.push` paired with one `mask.push`; PinValue/visibility-suppress/Boolean-secret `continue` paths push nothing to argv. `debug_assert_eq` fires in debug/CI.
- **Display wiring:** D1 Preview, D2 confirm modal, D3 last-run all use `render_copy_command_masked`; copy buttons + Run use the REAL render. The `argv_posix = preview.clone()` alias correctly split (POSIX copy reveals the real command). `result.mask` assigned by `spawn_and_capture` at BOTH call sites before `app.last_run` store.
- **T-A3 dangerous direction sound:** `mask.any() ⟹ should_confirm_run` holds for all five vectors; safe asymmetry documented not asserted; no-secret complement tested.
- **`SECRET_MASK` un-quoted** (display sentinel, bypasses shell-quote); T-A2 guards it.
- **Fail-open `unwrap_or(false)` acceptable** given the construction guarantee + debug_assert.
- **Runner stays mask-oblivious**; `to_slot_argv` thin-wrapper single-sources the format string; existing `RunResult` consumers (tree_form helper) updated; no other consumer.
