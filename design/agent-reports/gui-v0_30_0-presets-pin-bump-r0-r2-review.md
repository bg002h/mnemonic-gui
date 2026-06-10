# R0 review — SPEC_gui_v0_30_0_presets_pin_bump — round 2
**Verdict: YELLOW** (0C / 1I — one residual gap inside the C1 fold's UX surface; everything else folded clean.)

## Round-1 fold verification (C1, C2, I1-I5, M1-M8)

| Finding | Status | Verification |
|---|---|---|
| C1 sentinel | **RESOLVED** (mechanics) — one residual UX gap, see I-1 | Traced end-to-end, all four legs hold. (1) Seed: scalar path → `default_flag_value_for_flag`; with `default_value: Some("")` the Dropdown arm at widget.rs:172 returns `Dropdown("")` directly — NOT `opts.first()` (fallback :133-135 only when `default_value == None`). (2) Suppression doubly guaranteed: `is_at_default` `"" == ""` (invocation.rs:84, gate :303-305) AND `emit_one` skips empty Dropdown (:316-317). (3) Selected archetype emits (`is_at_default` false). (4) Mutex: `flag_value_is_present` is `!s.is_empty()` for Dropdown (mod.rs:405-407) → `has_value` false in the unset state → `--spec` enabled at frame 1; the §4 fn is implementable on the `export_wallet` precedent (conditional.rs:586-598). §6 cell pins the unset direction. |
| C2 seed rule | **RESOLVED** (one wording gap, M-A) | Required-row seed via `default_flag_value_for_flag`: `--to` → `Dropdown("phrase")` (= today). Other required+repeating flags are Text → emit nothing, same as today. No interaction with C1 (`--archetype` non-repeating). |
| I1 | RESOLVED | claim corrected; byte-equal const pin specified. |
| I2 | RESOLVED | all three breakages owned (6-flag set :32-39, `conditional.is_none()` :47, `v0_50_0` name :28). |
| I3 | RESOLVED | add-row Dropdown seeds `Dropdown("")` + cell. |
| I4 | RESOLVED | row_idx in salts. (Text rows safe — egui auto-IDs positional.) |
| I5 | RESOLVED | header row at any row count. |
| M1-M2, M4-M8 | RESOLVED | per round-1 prescriptions. |
| M3 | **FOLD-DRIFT** (minor) | §1 corrected to `:3764` but §8 still says "~`:3690`" — internal inconsistency (M-B). |

## Critical
None.

## Important

**I-1. The `""` sentinel option's DISPLAY rendering is unspecified — as the render arm stands, the unset row is a ~4-pixel sliver, and the SPEC's own claim that "the user can re-select `"(none)"`" is not implementable without a render change the SPEC nowhere owns.**
- The Dropdown arm uses the raw option string as both row label and selected text: `ui.selectable_value(sel, (*opt).to_string(), *opt)` (`src/form/widget.rs:283`) and `.selected_text(sel.as_str())` (:277). For `opt == ""` the popup row's clickable rect ≈ a few px — egui 0.31's `SelectableLabel` sizes to its text (`selected_label.rs:49-51`) and the ComboBox popup does NOT justify items to popup width (`combo_box.rs:417-426`).
- Not merely cosmetic: `FormState` is persisted per `"cli:subcommand"` (`src/main.rs:76-77`) and NO form-reset affordance exists. The first time a user selects any archetype, `--spec` goes Disabled and the only road back is a near-invisible click target — a persisted, softened recurrence of the C1 trap.
- Fix: the Dropdown render arm maps the empty option to a display label (`"(none)"`) for BOTH the popup row and `selected_text` — display-only, stored/emitted value stays `""`. Extend the §6 conditional cell with the round-trip: select an archetype → `--spec` disabled → re-select "(none)" → `--spec` re-enabled and `--archetype` gone from argv. Generalizes safely: no existing Dropdown const contains `""`.

## Minor

**M-A.** §3 states WHEN the required first row seeds but not WITH WHAT VALUE; an implementer reusing the add-row empty seed re-creates the C2 regression. Say: required-row first-render seed = `default_flag_value_for_flag(flag)`.

**M-B.** §8 still cites the banner at "~`:3690`"; §1 and the source agree on `:3764`. Align.

**M-C.** Remove-last-row of a REQUIRED repeating flag unspecified — the natural lazy implementation respawns the sole required row next frame (✕ a no-op). Acceptable/arguably correct; say it is intended so the kittest doesn't enshrine an accident.

**M-D.** §4's "check at edit time" hedge is already resolved definitively by §6 ("no rule-count update needed") — collapse to a pointer.

## Empirical probes run

1. GUI repo at `020f765`; tree clean except SPEC + round-1 report.
2. widget.rs full read — seed paths, Dropdown render arm (:276-286), mismatch-recovery (:378-384).
3. invocation.rs — Disabled suppression (:160-177), repeating loop (:255-258), is_at_default (:84) + emit_one guards (:307-308, :316-317).
4. mod.rs — has_value/flag_value_is_present (:304-312, :405-407); NumberMax::Static(i64); Number(i64).
5. conditional.rs:586-598 — the export_wallet mutex precedent.
6. tests/build_descriptor_schema.rs — the three I2 breakages live.
7. tests/conditional_visibility.rs:315-340 — M4 target.
8. mnemonic.rs:3764 — banner (M-B evidence).
9. schema_check.rs:97-104 — name-only (I1).
10. egui 0.31.1 source — selected_label.rs:49-51, combo_box.rs:408-433 (the sliver-row evidence).
11. greps: no `""` in any Dropdown const; `default_value: Some` precedents exist; no reset affordance; FormState persisted per subcommand.
