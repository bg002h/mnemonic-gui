# Implementation Review — GUI v0.40.0 (wire the paste-warn modal) — before commit

Reviewed the uncommitted working-tree implementation against the R0-GREEN spec.

**Verdict: 0 Critical / 0 Important / 1 Minor.**

## Critical / Important
None.

## Minor
**M-1 — T-B4 grep blind to multi-line `show` formatting (tripwire fragility, not a defect).** The count `.show(ui, ` − `.show(ui, |` requires `ui,` on the `.show(` line; a future line-wrapped 4th direct site would evade the count (read 2, pass falsely). All 3 current sites are single-line. **APPLIED:** collapse all whitespace before counting (`src.split_whitespace().collect()` → match `.show(ui,` − `.show(ui,|`) so multi-line calls still match. Re-ran: 4/4 pass.

## Confirmations
- **Detection correct:** `Event::Paste` stays in `i.events` after the TextEdit (egui 0.31 `filtered_events` clones); `response.changed()` attributes to the focused recipient. `pasted_len` computed every frame but acted on only inside `if response.changed()`. The theoretical false-positive (typing into the secret field while a paste event from another widget coexists in the same frame) is the SPEC-accepted structural assumption — physically near-impossible (typing vs pasting are distinct gestures). Buffer zeroize/reassign ordering unchanged + correct.
- **Read-once placement:** `remove_temp::<bool>` after the CentralPanel closure (where all `show` calls ran), before the modal; reads AND clears (T-B3) → no leak / no forever-retrigger. No `show` calls outside the CentralPanel closure.
- **Modal:** gates on `pending_paste_warn`, renders `PASTE_WARN_MODAL_TEXT`, non-blocking `egui::Window`, clears on Dismiss; coexistence with the run-confirm modal is SPEC-accepted.
- **Honesty:** the `:1989-1991` over-claim corrected (now "validates the PREDICATE … wiring was DEAD until v0.40.0"); the `[0.40.0]` entry scopes the warn to SecretLineEdit + names both deferred FOLLOWUPs. `paste_warn_id()` is a stable `egui::Id`.
- **Tests:** T-B2 live (inject ≥/< threshold into a focused PasswordInput), RED-proven via scratch-revert; T-B3 pure-logic read+clear; T-B4 count math verified (widget.rs 2, main.rs 4−3=1, total 3).
- **No regressions:** `pending_paste_warn` init `false` in `new()` (no Default derive, not serde); `show` change is additive; no schema_mirror/manual/toolkit-pin impact. Both resolved slugs + both new deferred FOLLOWUPs correctly filed.

Faithful to the R0-GREEN spec.
