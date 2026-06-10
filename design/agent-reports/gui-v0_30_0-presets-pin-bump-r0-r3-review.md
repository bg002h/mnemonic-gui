# R0 review — SPEC_gui_v0_30_0_presets_pin_bump — round 3
**Verdict: GREEN** (0C/0I)

## Round-2 fold verification (I-1, M-A..M-D: RESOLVED | FOLD-DRIFT)

| Fold | Status | Verification |
|---|---|---|
| I-1 display mapping | **RESOLVED** | §3 gains the "Empty-option display" bullet: `""` → `"(none)"` for BOTH the popup row and `selected_text`, explicitly DISPLAY-ONLY. §6's conditional cell gains the exact round-trip. **Render-arm soundness confirmed:** the single Dropdown arm (`src/form/widget.rs:267-287`) owns both display sites — `.selected_text(sel.as_str())` at `:277` and `ui.selectable_value(sel, (*opt).to_string(), *opt)` at `:283` (third arg = display text, second = stored value). A two-site label map covers selected_text + every popup row without touching `sel`, the write-back, or emission. Repeating rows render through the same arm → inherit the mapping. `grep -c '""' src/schema/mnemonic.rs` = 0 (generalization safety). Round-trip reachability: §4 disables `--archetype` only when `--spec` non-empty; `main.rs:459` gates only `Visibility::Disabled` — the combo stays clickable. |
| M-A seed value | **RESOLVED** | Required-row seed = `default_flag_value_for_flag(flag)`; convert `--to` (`default_value: None`, mnemonic.rs:762) falls through to `NODE_TYPES[0]` = `"phrase"` (mnemonic.rs:87) — identical to today. |
| M-B banner cite | **RESOLVED** | §8 reads `:3764`; source confirms; no `:3690` remains. |
| M-C respawn intent | **RESOLVED** | declared INTENDED; ambiguity closed. |
| M-D hedge collapse | **RESOLVED** | §4 points at the §6 resolution; no contradiction. |

No fold-drift. The C1 sentinel chain (seed → suppress → emit → mutex) is untouched by the I-1 display layer (all four legs key off the stored value).

## Critical
None.
## Important
None.
## Minor
**M-i (optional wording polish).** §3 "the first render seeds ONE row iff `flag.required`" + M-C's "lazy seed re-fires" together are unambiguous, but a literal reading of "first render" could suggest a one-shot. If touched again: "any render observing zero rows for a required flag seeds one." No re-review needed.

## Empirical probes run
1. HEAD = `020f765`, tree clean except SPEC + reports → round-2 citations valid by commit identity.
2. Folded SPEC + round-2 report read in full.
3. widget.rs full read — Dropdown arm `:267-287` (display sites `:277`/`:283`), `default_flag_value_for_flag` `:166-189`, dispatch seed `:101-110`, mismatch-recovery `:378-384`.
4. invocation.rs:303-320 + `:84` — emit guards re-confirmed.
5. main.rs:440-470 — `add_enabled_ui` gate; `:76-77` persistence.
6. mnemonic.rs — `:3764` banner; `:756-762` `--to`; `:86-89` NODE_TYPES; `grep -c '""'` = 0.
7. conditional.rs:586-600 — the export_wallet mutex precedent.
