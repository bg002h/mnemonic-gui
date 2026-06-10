# R0 review — SPEC_gui_v0_31_1_repeating_secrets — round 1

**Verdict: RED**

The core inversion is **adjudicated correct**, the per-row `secret_widgets` design is sound, and the persistence story holds. But the SPEC's §2 assembler code as written breaks the currently-working seed-xor-combine `--share`, and the §4 call-site census misses the one site that fails **silently** (compiles clean with wrong semantics) and regresses run-confirm + conditional gating for every scalar secret.

## Critical

**C1 — §2 assembler code breaks NodeValueComposite repeating secrets (seed-xor-combine `--share` stops emitting; required flag → form un-runnable).** seed-xor's `--share` is secret+repeating+**NodeValueComposite** (`mnemonic.rs:1590-1596`); it emits today THROUGH the secret branch's repeating values-read (the widget routes it via render_repeating → state.values; the widget secret dispatch requires Text, the assembler check is kind-BLIND). §2's code would `secret_widgets.get("--share")` → None → continue → emits NOTHING. `cell_v0_3_seed_xor_combine_argv_assembles` (`tests/widget_interaction.rs:378-434`) would go RED. **Fix: the assembler secret branch mirrors the widget dispatch — `flag_is_secret && matches!(kind, Text)` → the vec; non-Text secrets fall through to the generic values paths**; the Boolean `*-stdin` secrets currently emit NOTHING (the kind-blind branch ate them) — falling through would make them START emitting; decide explicitly (recommend: preserve today's suppression in this PATCH + file a FOLLOWUP).

**C2 — §4 census misses `FormState::has_value` (`src/schema/mod.rs:339-347`) — the one migration site that compiles silently with wrong semantics.** After Vec, `w.is_empty()` is `Vec::is_empty` (compiles!) — meaning flips from "buffer non-empty" to "vec non-empty". Consequences: run-confirm modal fires on every passphrase-bearing run (`should_confirm_run`, wired `main.rs:668`); conditional XOR gating misfires (`has_passphrase` sites; `has_ms1` → `--bundle-json` wrongly disabled); the required-seed rule keeps `--share` ≥1 blank row → misfire on a blank form. **Fix: list has_value as a site with per-row semantics `rows.iter().any(|w| !w.is_empty())`; §5 pins the faithful migration of the empty-widget negative cell (`tests/secrets.rs:167-172` → `vec![SecretLineEdit::new()]`).**

## Important

**I1 —** §4 also misses `secrets::zeroize_form_state` (`secrets.rs:292-294`) — compile-caught (inherent method, not the trait), but the census is wrong twice; per-row sweep `values_mut().flatten()`.
**I2 —** the live-path typing mechanism is unproven in-repo: no test types into any TextEdit. kittest 0.1.0 has `Node::focus()` + `type_text()` — name them + define the fallback (seed rows via `SecretLineEdit::from_text` + drive the real `render_with_dispatch` for add/remove/seed coverage — still pins the render→assemble seam).
**I3 —** §3's union sweeps in 5 **Boolean** `secret: true` flags (`--passphrase-stdin` ×12 sites, `--secret-stdin` ×2, `--decrypt-password-stdin` ×2, `--bip38-passphrase-stdin` ×1) whose persisted toggles would newly reset across restarts. Defensible (the `--passphrase-stdin` precedent; no secret material) — state it deliberately. Also: these Booleans currently emit NOTHING via the kind-blind secret branch — preserve that in this PATCH; file `boolean-stdin-secret-toggles-never-emit`.
**I4 —** enumerate the §5 lists. MIGRATE to the vec source: `cell_import_wallet_repeating_ms1_argv` (kittest_import_wallet_form.rs:157-213), `cell_import_wallet_env_sentinel_literal_emission` (:322-350), `cell_v0_3_slip39_combine_argv_assembles` (widget_interaction.rs:296-336). KEEP UNCHANGED (counter-example pin): `cell_v0_3_seed_xor_combine_argv_assembles` (:378-434). Mechanical `insert(name, vec![widget])`: argv_assembler.rs:41/:282/:322, argv_assembler_visibility.rs:198/:218, secrets.rs:164/:171, persistence.rs:392. Unaffected: r7_no_auto_repair_removal.rs ambient `--ms1` values.

## Minor

**M1 —** the paste-warn modal is NOT wired live (zero `should_warn_on_paste` callers in src/; deferral documented at `tests/widget_secret.rs:19-25`) — don't claim "preserved per row" and don't repeat the fold comment's false "modal still fires" line in the §0 supersession note.
**M2 —** row removal orphans egui `TextEditState` (undo-ring plaintext snapshots) at the vacated trailing positional ID — same class as the existing `gui-secret-buffer-allocator-residue` FOLLOWUP; record, no new work. Zeroize-on-removal itself confirmed (Drop on `Zeroizing<Vec<u8>>`).
**M3 —** PATCH call correct; the pub-field type change breaks the lib API — CHANGELOG note.
**M4 —** drift test: add `schema_secret_flag_names() ⊇ {--ms1, --share} ∪ SECRET_FLAG_NAMES` so an emptied union fails loud.

## Citation audit
Inversion rationale CONFIRMED (redact_for_persistence drops only the 3 classes; save() has no other layer; values-routed secrets WOULD persist; a crafted stale state.json with --ms1 values rows would emit AND re-persist today — §2+§3 close both). All §0/§7 cites verified. Census re-verified (+5 Boolean secrets the SPEC omitted). §4 "4 sites" wrong: 6 src + 9 test. §3 feasible (all 4 schemas pub const SCHEMA; md/mk have zero secret flags). Runner has no secret env-bag (`runner.rs:91` MNEMONIC_FORCE_TTY only).

## Empirical probes run
Read-only audit (full reads + greps as enumerated in the review body); kittest API check (focus/type_text exist at kittest-0.1.0); git state local == origin dabbdfe.
