# R0 round-1 architect review — SPEC_gui_v0_37_0_greyout_stdin_toggles (2026-06-10)

Reviewer: Fable 5 architect agent (R0, pre-implementation). master @ 55e5cd2. Verdict: GREEN (0 Critical / 0 Important / 5 Minor). Review verbatim below (abridged — full evidence in transcript).

---

## Critical
None. Two candidate-Criticals investigated → not defects:
- **Conditional-mutex trigger loss** (passphrase-XOR-stdin mutex off `has_value("--passphrase-stdin")`, conditional.rs:195/207 etc.): harmless — the mutex only protected against a clap `conflicts_with` rejection that can no longer occur (the toggle no longer emits); no functional regression.
- **Stale-persisted Boolean(true) lockout** (a v0.36.0-checked toggle locks `--passphrase` Disabled with the clearing toggle now itself disabled): NEUTRALIZED by `redact_for_persistence` — it drops every state.values entry under any `schema_secret_flag_names()` name (all 6 stdin toggles are secret:true), so no stale Boolean(true) survives a restart (secrets.rs:323-326). An upgrade IS a restart.

## Important
None.

## Minor
1. **Predicate-mirror EXACT — record the enumeration + add converse-closure to T1.** Full census of `flag_is_secret==true` partitions cleanly: Text (inline secrets, handled by secret_widgets) + one NodeValueComposite (--share) + exactly 6 Booleans (the *-stdin toggles). NO secret Number/Path/Dropdown/Range/Timestamp exists. So Boolean-only grey-out neither over- nor under-greys; `secret && !Text && !Composite` ≡ the 6 Booleans ≡ the assembler's `else→continue` set (invocation.rs:277-278). **Strengthen T1:** also assert NO `secret && !Text && !Composite && !Boolean` flag exists (panics if a future secret-Path is added → trips RED instead of silently rendering a live-but-dead control).
2. **T2 precedent wrong + empty-text checkbox not queryable.** The real `is_disabled()` kittest precedent is `tree_form.rs:543` (works because the button's OWN text is the label). The SPEC's `Checkbox::new(&mut x, "")` + separate `ui.label(flag.name)` means `get_by_label(flag.name)` returns the Label node, not the CheckBox. **Fix:** label the checkbox itself — `Checkbox::new(&mut unchecked, flag.name)`, drop the separate `ui.label`, so `get_by_label(flag.name).is_disabled()` targets the checkbox (mirrors tree_form.rs:543). kittest 0.1.0 exposes is_disabled() via Deref.
3. **Stale doc "5 Boolean" at secrets.rs:323 → actual 6** (--passphrase-stdin is secret:true AND in SECRET_FLAG_NAMES). Pre-existing; optionally fix in-cycle.
4. **egui API correct for 0.31.1:** `Ui::add_enabled(bool, impl Widget)` exists (ui.rs:1666; repo uses it at tree_form.rs:507); on_hover_text on the (enabled) label works regardless of the checkbox's state; `egui` via `use eframe::egui`; render_help_icon module-private in scope. Nests cleanly inside main.rs:653's outer add_enabled_ui wrapper (early return inside the closure; egui disabled-state nests).
5. **SemVer/schema_mirror/ritual sound:** no flag-name/secret-bit/value/subcommand change → schema_mirror + 3 drift gates byte-unaffected; assemble_argv byte-identical; MINOR correct (24 checkboxes become disabled-with-tooltip).

## Verdict
**GREEN (0 Critical / 0 Important).** Predicate-mirror verified exact; both Critical risks dissolve against source. Fold M1 (converse-closure T1) + M2 (label-the-checkbox + cite tree_form.rs:543) during impl; M3 optional.
