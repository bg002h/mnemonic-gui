# R0 review — P2 (I2 conditional & state integrity), UI-test harness — round 1

**Reviewer:** opus architect (adversarial, per-phase mandatory R0 gate; 0C/0I required).
**Subject:** branch `feat/ui-harness-p0-spike` @ `7325ae4` (P2 commit); `master` @ `da47994` (untouched).
**Diff scope:** TESTS-ONLY — `git diff --name-only master..feat/ui-harness-p0-spike` = `tests/spike_widget_drivers.rs`, `tests/ui_harness/mod.rs`, `tests/ui_harness_i1_roundtrip.rs`, `tests/ui_harness_i2_conditional.rs` + `design/` docs. **NO `src/` change** (`git diff --stat master..HEAD -- src/` empty). Worktree clean post-review.

---

## VERDICT: GREEN — 0 Critical / 0 Important.

P2 is faithful, non-tautological, and all gates pass. Every narrowing is legitimate and verified against current source. Findings below are 3 Minor + 3 Nit, none gating, none misleading P3–P6. I am not rubber-stamping — the four narrowings were the real risk and I chased each to source; they hold.

---

## Gates (all re-run live, `--jobs 2`)

| Gate | Result |
|---|---|
| `cargo test --test ui_harness_i2_conditional` | **31 passed / 0 failed** |
| Determinism re-run (run 2) | **31 passed / 0 failed** — identical → deterministic |
| `cargo test --test ui_harness_i1_roundtrip` (P1) | **10 passed / 0 failed** |
| `cargo test --test spike_widget_drivers` (P0) | **6 passed / 0 failed** |
| `cargo clippy --all-targets -- -D warnings` | **clean** (Finished, 0 warnings) |
| broad `cargo test --jobs 2` | **60 `test result: ok` lines, 0 FAILED / 0 error / 0 panicked** |
| `cargo test --test schema_mirror` | **21 passed / 0 failed** |
| `cargo test --test gui_schema_conditional_drift` | **5 passed / 0 failed** |
| `src/` change | **none** |
| broad fmt churn | **none** (diff = new test files + design only; no existing file reformatted) |

---

## Adversarial verification of the load-bearing claims

### 1. `render_whole_form` faithfulness (primary risk) — VERIFIED byte-faithful
Compared `tests/ui_harness/mod.rs:414-449` (`render_whole_form`) + `:376-398` (`is_render_suppressed`) line-by-line against the real loop `src/main.rs:601-686`:
- `vis = conditional(state)` once per frame (`mod.rs:420` vs `main.rs:582-585`) ✓
- `--slot && allows_slots` continue (`mod.rs:381-383` vs `main.rs:625-627`) ✓
- tree-mode continue (`mod.rs:384-388` vs `main.rs:634-640`) ✓
- archetype-mode continue (`mod.rs:389-396` vs `main.rs:641-647`) ✓
- `Hidden → continue` (`mod.rs:432-435` vs `main.rs:648-651`) ✓
- `DisableOptions` extraction (`mod.rs:436-444` vs `main.rs:658-668`) ✓
- `add_enabled_ui(!Disabled, render_with_dispatch(…, disabled_options))` (`mod.rs:445-447` vs `main.rs:674-686`) ✓

**Does omitting the sub-surfaces hide a conditional bug?** No. I grepped every `conditional()` fn (`src/form/conditional.rs`) for gate inputs: the 17 conditionals read `state.values` (flag values), `state.tree` (`build_descriptor`, `:657`), and `state.positionals` (`md_encode :782`, `md_address :833`). **No `conditional` reads `state.slots`** — the only `slot_count` reads are `template_slot_count_warning` (`:346`), a *non-conditional* warning helper, NOT a gate. So omitting `SlotEditor` (and `bundle`/`verify-bundle`/`export-wallet` are `allows_slots: true`) changes **no** flag's visibility/enable. The tree/positional/archetype-value gate inputs are seeded directly by the cells (`st.tree = Some(...)`, `st.positionals.push`, `seed(--archetype,…)`), which is equivalent to rendering the sub-surface for a *pure* conditional. Faithful.

**`query_by_label` exact-match:** kittest 0.1.0 names the substring variant separately as `by_label_contains` / `query_by_label_contains` (`kittest-0.1.0/src/query.rs:194-200`, doc "the node label contains the given substring"); the un-suffixed `by_label` is full-string match. So `query_by_label(flag.name)` is EXACT — `probe_label_match_is_exact_not_substring` (`i2:222-232`) would RED on a `_contains` swap (`--descriptor` substring-collides `--descriptor-file`). Verified handle is unique.

**`is_disabled()` ⇔ `add_enabled_ui(false)`:** `probe_disabled_state_reflects_add_enabled_ui_gate` (`i2:234-242`) empirically proves the wrap propagates to the inner label's AccessKit `is_disabled` (compare-cost: `--miniscript` set ⇒ `--descriptor` disabled, populated sibling enabled). Confirmed by run, not assumed.

### 2. DisableOptions narrowing — LEGITIMATE
**"No live producer in the 17":** grep-verified. `grep "Visibility::DisableOptions {"` over `src/form/conditional.rs` = **zero constructions**; the only token is the v0.7.2-revert *comment* (`conditional.rs:257`, "v0.7.1 introduced row 10/11 DisableOptions pushes here; v0.7.2 reverted them"). Claim is solid via source.
The synthetic cell `i1_disable_options_render_best_effort_and_argv_firm` (`i2:693-731`) fairly tests both legs: render-best-effort (`--pick` label present+ENABLED; the greyed `beta` *option* still renders in the popup — `i2:716-722`), and FIRM argv (stale `beta` STILL emits — `i2:726-730`, matching `invocation.rs:189` where `suppresses` excludes DisableOptions). Fair.

### 3. PinValue narrowing — LEGITIMATE
**Not render-consumed:** the render layer is visibility-agnostic — `render_with_dispatch` receives only `disabled_options`, and `grep "Visibility::\|PinValue\|conditional" src/form/widget.rs` finds **no rendering decision** keyed on visibility (the 3 hits are comments). The form loop (`main.rs:648-686`) applies *only* Hidden/Disabled/DisableOptions; PinValue falls through as a normal enabled widget. So `assert_enabled(--account)` (`i2:272`) is correct, not a missed assert.
**FIRM argv bites:** `i1_bundle_pinvalue_firm_argv` (`i2:559-587`) seeds `--account 5` + canonical `--descriptor wpkh(@0)`, settles, and asserts argv emits `0` (the pin) AND `5` is absent. The pin fires because `wpkh(@0)` classifies **Canonical** (`classify_descriptor_canonicity` wpkh regex `^wpkh\(…@\d+…\)$`, `conditional.rs:108`) ⇒ `is_descriptor_non_canonical` false ⇒ `conditional.rs:243-247` pushes `PinValue(0)`; `assemble_argv` replaces (`invocation.rs:208-214`). Bites.

### 4. The two "not render-reachable" narrowings
**(a) export-wallet/convert "neither-set ⇒ Required" unreachable — REAL renderer property, LEGITIMATE.** Verified the auto-seed mechanism: `render_with_dispatch`'s scalar path (`widget.rs:214-223`) computes a default for an ABSENT flag and **pushes it into `state.values`** (the `None` write-back arm `:222`). For a Dropdown with `default_value: None`, the default is `opts[0]` (`default_flag_value_for(kind)`, `widget.rs:382-385`). Both export-wallet `--template` (`mnemonic.rs:1384`) and convert `--template` (`:1235`) are `Dropdown(TEMPLATES)` with `default_value: None`, and `TEMPLATES[0] = "bip44"` (`:69-71`) — NON-empty. `has_value` is false only for an empty Dropdown (`flag_value_is_present`: Dropdown ⇒ `!s.is_empty()`, `schema/mod.rs`). TEMPLATES carries **no empty/`(none)` option** the user could select (the `(none)` text at `widget.rs:581` is only the *display* of an already-empty sentinel, not a selectable opt). So a fresh form auto-seeds `--template = "bip44"` ⇒ `has_template = true` permanently ⇒ the neither-set arm (`export_wallet :605-608`) is genuinely dead at steady state. The cells defeat the auto-seed by seeding `--template ""` explicitly (`s_export_descriptor`, `s_convert_address`) to reach the Disabled/Required arms — the seeded `""` survives render (no coercion to opts[0]; GREEN proves it, else `--descriptor` would cross-disable). The skipped arm's ONLY render-distinguishable effect is `--taproot-internal-key Disabled`, which the reachable single-sig case already covers (`i1_export_wallet`, `i2:350-353`); the Required-on-both markers are render-invisible (`main.rs` wires no conditional-Required asterisk). Zero lost coverage.
**(b) build-descriptor Disabled-on-`--spec` shadowed by the mode continue — LEGITIMATE.** `suppressed_in_archetype_mode` includes `"--spec"` (`archetype_form.rs`), and the archetype-mode `continue` runs BEFORE the `Hidden/Disabled` check in BOTH `main.rs:641-647` and `render_whole_form` (`mod.rs:389-396, 429`). So in archetype mode `--spec` is ABSENT, not greyed; `i1_build_descriptor` asserts `assert_absent(--spec)` (`i2:367`) — faithful. The conditional still *runs* (`vis` computed); it is pre-empted, not silently skipped — byte-identical to the host loop.

### 5. Fenced suppression + toggle round-trip — SOUND
Suppression asserted only for Hidden|Disabled (`i2:803-941`) — matches `invocation.rs:189` (`matches!(v, Hidden | Disabled)`) exactly; PinValue/DisableOptions deliberately excluded. The toggle equivalence is `visibility_projection` = `vis_tag(conditional(state))` over **every** flag (`mod.rs:507-515`), not stored values — sound and non-circular (the projection is the rule; the round-trip asserts state returns to a rule-equivalent point; non-vacuity is explicitly asserted `proj_off != proj_on`, `i2:973-976, 1035-1038`). `i3_toggle_roundtrip_md_compile_widget_driven` (`i2:950-985`) drives REAL widgets (`get_by_role(ComboBox).click()` + `get_by_label("segwitv0").click()`, `i2:994-1003`) — md compile renders exactly 1 TextInput (`--unspendable-key`) + 1 ComboBox (`--context`) + 1 CheckBox (`--json`) (schema/md.rs `COMPILE_FLAGS`), so the drive is uniquely role-targetable. Genuine widget toggle, not state-mutation.

### 6. Universal sweep non-vacuity — GENUINE, not vacuous
`i1_render_matches_conditional_projection_over_all_17` (`i2:757-793`): expected = `effect_of(sub, state, flag)` (the rule); observed = `label_present`/`label_disabled` reading the **AccessKit tree** (egui's actual render output, `mod.rs:540-547`). These are independent sources — a form-loop gate bug (e.g. Disabled rendered enabled, or a missed Hidden-continue) makes observed diverge from expected → RED. Not self-referential. Non-vacuity guarded by `non_visible > 0` AND `total_checked > 0` (`i2:788-792`), and the `CASES` table is asserted to be exactly 17 with ≥1 non-Visible effect each (`i2:763`).

### 7. Anti-tautology — confirmed
No cell re-proves the rule against itself. `effect_of` preconditions (`eff(...)`) are explicitly documentary where Required is render-indistinguishable from Visible (no asterisk wired) — stated at `i2:36-43, 106-109` and consistent with `main.rs` consuming only Hidden/Disabled. The renderer-vs-rule comparison reads egui output for the observed side. I2 correctly catches DESYNC, not a wrong rule (owned by `conditional_visibility.rs` / `gui_schema_conditional_drift.rs`).

---

## Ruling on each narrowing
| Narrowing | Ruling | Basis |
|---|---|---|
| DisableOptions = synthetic + guard | **LEGITIMATE** | Zero `Visibility::DisableOptions{` in `conditional.rs` (only the v0.7.2-revert comment, `:257`); synthetic cell exercises render-best-effort + firm-stale-argv faithfully. |
| PinValue = best-effort render / firm argv | **LEGITIMATE** | Render layer visibility-agnostic (`widget.rs` no visibility decision); form loop applies only Hidden/Disabled; firm argv coercion `5→0` proven (`i2:559-587`). |
| export-wallet/convert "neither-set ⇒ Required" not render-reachable | **LEGITIMATE** | Auto-seed pushes `opts[0]="bip44"` (non-empty) into `state.values` (`widget.rs:214-223`, `TEMPLATES[0]`); no selectable empty opt ⇒ `has_template` always true at steady state; the arm's only render-distinguishable effect is covered by the single-sig case. |
| build-descriptor Disabled-on-`--spec` shadowed by mode continue | **LEGITIMATE** | `suppressed_in_archetype_mode` ⊇ `--spec`; mode `continue` precedes the Disabled check in both `main.rs:641-647` and `render_whole_form` ⇒ ABSENT, asserted faithfully (`i2:367`). |

---

## Minor (non-gating)
- **M1 — DisableOptions tripwire under-covers multi-flag producers.** `i1_disable_options_no_live_producer_in_the_17` (`i2:645-674`) probes `default` + one-flag-present states only. A FUTURE conditional emitting `DisableOptions` only when ≥2 flags are co-set would evade the runtime tripwire (the grep is the real guarantee today; the tripwire is the future-regression net). Consider probing pairwise, or comment that the tripwire is best-effort. Non-gating: no live producer exists, and the render path already handles DisableOptions generically.
- **M2 — universal projection round-trip is pure-state, not widget-driven.** `i3_visibility_projection_roundtrip_universal` (`i2:1005-1048`) toggles via `FormState` push/retain, so for the 5 non-`md_compile` subs it proves rule-purity + push/retain inverse, not a renderer-introduced stuck state. The widget-driven proof lives only in the `md_compile` cell. Matches the spec (toggle round-trip is *defined* over the visibility projection), so acceptable — flagging the coverage shape for P5/P6 (the sweep could widen widget-driven toggles).
- **M3 — per-sub non-vacuity not individually asserted in the sweep.** `i1_render_matches_conditional_projection_over_all_17` guards `non_visible > 0` globally; a single sub contributing zero eligible flags would pass silently. Mitigated by the 17 targeted `i1_*` cells (each carries ≥1 render-distinguishable assertion). Non-gating.

## Nit
- **N1 — `is_render_suppressed` recomputes mode per-flag vs `main.rs` once-per-frame.** `main.rs:604-621` computes `tree_mode`/`archetype_spec` once; `render_whole_form` recomputes via `is_render_suppressed` per flag (`mod.rs:429`). At run-to-stable the state is quiesced, so the asserted frame is identical — no divergence. Worth a one-line comment for the next reader.
- **N2 — "NOT render-reachable" phrasing (i1_export_wallet, `i2:331-333`) is slightly stronger than true.** The cell itself *can* construct the neither-set state by seeding `--template ""` (as `s_export_descriptor`/`s_convert_address` do); the precise claim is "not reachable by a real user at steady state, and its only render-distinguishable effect is covered elsewhere." Comment-only.
- **N3 — md/ms/mk `--template`-style Dropdowns rely on `opts[0]` non-empty across schema reorders.** Narrowing 4(a)'s "unreachable" rests on `TEMPLATES[0]` being non-empty; if a future schema prepends an empty/`(none)` sentinel to a `*TEMPLATES` array, the neither-set arm becomes reachable and these cells would need a render-leg. Low risk; note for P6's permanent-gate absorption.

---

## P3–P6 forward note
Nothing here misleads later phases. The `render_whole_form` / `is_render_suppressed` machinery (`mod.rs:336-537`) is sound to build P3 (I3 secret persistence) on. M2/N3 are the only items worth carrying into the P5 sweep / P6 gate design (widen widget-driven toggles; guard the opts[0]-non-empty assumption). The exact-label handle + `eligible_for_label_check` exclusions (secret Booleans, mode-suppressed) are correct and reusable.

**Gate: GREEN — 0 Critical / 0 Important. Proceed to P3.**
