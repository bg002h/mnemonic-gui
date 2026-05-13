# Phase A.3 egui_kittest Scaffold — R2

**Date:** 2026-05-12
**Reviewer:** feature-dev:code-reviewer (Sonnet 4.6)
**Scope:** Fold commit f4f61da on branch v0_2 verifying R1 folds + plan amendments
**Plan ref:** /home/bcg/.claude/plans/v0_2-mnemonic-gui.md §C Phase A.3 (R5 amended)

## Verdict

**0C / 0I / 1N — converge**

All five R1 findings (3I + 2N) are correctly folded. Plan canonical and stash are in sync at all amended lines. Exit gate satisfied. Phase A.3 ready to close.

---

## Fold verification

### R1 I-1 — Exit gate wording (plan line 760)

**Status: CORRECTLY APPLIED**

Both canonical (`/home/bcg/.claude/plans/v0_2-mnemonic-gui.md:760`) and stash (`/scratch/code/shibboleth/mnemonic-toolkit/.v0_2-plan-stash.md:760`) read identically. The rewrite:

- States "both cells pass against v0.1 behavior at landing" (GREEN-at-landing acknowledged)
- Identifies "pre-implementation RED state is the compile-fail of the missing test binary" (compile-fail-as-RED acknowledged)
- Includes "`accesskit` feature added to `eframe`" requirement
- Cites R5 fold and R1 C-1 fold for context

All three I-1 required elements present.

---

### R1 I-2 — Cell 1 fidelity (plan line 751)

**Status: CORRECTLY APPLIED**

Both canonical and stash at line 751 rewrite cell 1 as "add row → remove → add again; assert `to_slot_argv()` emits no pairs (SPEC §6.7 empty-value omission rule)." The byte-exact argv deferral is explicitly stated: "byte-exact argv assertion with subkey/value-typed flow deferred to a Phase B.1 addendum, since `SecretLineEdit` exercises kittest's TextEdit interaction directly." Deferral vehicle (Phase B.1) is explicitly named.

The actual test at `tests/widget_interaction.rs:35-88` matches the amended spec: add/remove/add lifecycle with `to_slot_argv()` empty assertion.

---

### R1 I-3 — SPEC §6 cell 2 aspirations (plan line 530)

**Status: CORRECTLY APPLIED**

Both canonical and stash at line 530 include the R5 fold notice: "R5 fold: dropped aspirational `--threshold` / `--wallet-name` visibility assertions — no v0.2 phase extends `conditional::export_wallet()` with those rules." Cell 2's Assertion column now covers only the SPEC §5 mutual-exclusion + runtime-pre-check Required logic, matching the actual implementation.

---

### R1 N-1 — Comment fix at `tests/widget_interaction.rs:60`

**Status: CORRECTLY APPLIED**

Lines 60-63 read:

```
// The new row has default values: index = 0, subkey = SlotSubkey::Xpub
// (hard-coded in SlotRow::default() at slot_editor.rs:83-90 — note
// that this is NOT SlotSubkey::ALL[0] which is Phrase; the default
// impl picks Xpub explicitly), value = "".
```

Correctly names `SlotSubkey::Xpub` and explicitly distinguishes from `SlotSubkey::ALL[0]` to prevent re-confusion.

---

### R1 N-2 — FOLLOWUPS entry `gui-accesskit-production-side-effect`

**Status: CORRECTLY APPLIED**

Entry exists at `FOLLOWUPS.md` under the Active section. Checklist:

- Root cause: "Cargo feature unification then activates `egui/accesskit` globally because `egui_kittest 0.31.1 → kittest 0.1.0 → accesskit 0.17.1` requires it" — present and detailed.
- Disposition: "Disposition: accepted. No cargo mechanism scopes a feature activation to dev/test builds only" — present and explicit.
- Revisit triggers: two stated — (1) egui_kittest 0.32+ decoupling the dep, (2) accessibility-tree exposure of mnemonic input fields as a threat-model concern.
- Trace: links to R5 fold log and the R1 report.

---

### Section C iterative-review log R5 entry

**Status: CORRECTLY APPLIED**

Both canonical and stash carry the R5 entry. It:

- Attributes the findings to "Phase A.3 landing commit" (bb50ac9)
- Explicitly uses "lightweight folds"
- Cites all 3 I-findings (R5 I-1, R5 I-2, R5 I-3)
- Cites both N-folds (R5 N-1 comment fix, R5 N-2 FOLLOWUPS entry)

---

### Plan stash sync

**Status: IN SYNC**

Spot-checked at lines 530, 751, 760, and the R5 log range. Canonical and stash are character-identical at all amended locations.

---

## Sub-threshold notes

### N-1 (new) — SPEC §6 table cell 1 row not aligned with I-2 fold

**Confidence:** 80
**File/ref:** Plan line 529 (both canonical and stash)

The I-2 fold correctly amended the normative phase description at line 751 but did not update the SPEC §6 table at line 529. The cell 1 table row still reads "add row → set subkey + value → remove → add again; then `assemble_argv()` | Emitted `--slot @0.xpub=<value>` matches expected argv byte-for-byte" — the pre-fold aspirational assertion. This creates an internal plan inconsistency between the table (§6) and the normative text (Phase A.3 description).

The table entry is documentation-only; no functional consequence. Stash and canonical agree (not a sync issue). Fix at next plan touch of this section — natural candidate is the Phase B.1 addendum that picks up cell 1's deferred byte-exact argv assertion.

---

## Exit gate checklist (amended per R5)

| Gate item | Status |
|-----------|--------|
| `cargo test --test widget_interaction` compiles clean | PASS — confirmed by R1; fold touches only FOLLOWUPS.md + test comment + R1 report (no test-logic changes) |
| Both cells pass against v0.1 behavior at landing | PASS |
| `egui_kittest = "0.31"` in `[dev-dependencies]` | PASS — `Cargo.toml:35` |
| `accesskit` feature added to `eframe` | PASS — `Cargo.toml:13` lists `"accesskit"` in features |
| No snapshot `.png` files | PASS — only `screenshots/*.png` (release UI screenshots, not kittest artifacts) |
| 0C / 0I | PASS |

---

Phase A.3 converges in 2 reviewer rounds (R1 → fold → R2). Next gate: the remainder of v0.2 begins at Phase B.1 (SecretLineEdit widget). Phase B.1 should fold in the Phase A.3 R2 N-1 (align SPEC §6 table cell 1 row with the I-2 amendment) when it touches the plan.
