# Phase 3 SlotEditor Review — R2

**Date:** 2026-05-12
**Reviewer:** feature-dev:code-reviewer (Sonnet 4.6)
**Scope:** Commit `2860aa2 fold Phase 3 R1 (0C/1I) — rows_sorted() doc comment factually corrected`
**R1 report:** `design/agent-reports/v0_1-phase-3-slot-editor-r1.md`

## Verdict

**0C / 0I — converge**

R1 fold verified correct against upstream source. No stale references remain. No new defects.

---

## R1 fold verification — RESOLVED

Post-fold comment at `src/form/slot_editor.rs:98-101`:

> Upstream `cmd::bundle::resolve_slots` keys its `BTreeMap<u8, Vec<&SlotInput>>` by `u8` index alone; duplicate `(index, subkey)` pairs for the same slot are rejected by `validate_slot_set` (`duplicate-subkey` error), NOT by BTreeMap-key collision. R1 I-1 fold (clarifies prior comment).

Stale "(index, subkey) keys" claim absent from all live source.

---

## Upstream verification

- `crates/mnemonic-toolkit/src/cmd/bundle.rs:298` declares `let mut by_index: BTreeMap<u8, Vec<&SlotInput>> = BTreeMap::new();` — keyed by `u8` only.
- `crates/mnemonic-toolkit/src/slot_input.rs:164,195` — `validate_slot_set` emits `kind: "duplicate-subkey"` for duplicate `(index, subkey)` within a slot.

Corrected comment is accurate on both counts.

---

## Broader Phase 3 sweep

- `src/form/invocation.rs:56-60` — `--slot` dispatch routes through `state.slots.to_slot_argv()`, bypassing `values` map. Matches SPEC §6.4.
- `to_slot_argv`, `rows_sorted`, `persistable_rows` unchanged since R1; logically correct.

No new defects above threshold.
