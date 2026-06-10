# SPEC — GUI v0.30.0: toolkit pin → v0.52.0 + the 12 build-descriptor flags + generic repeating-row widget

**Status:** R0 GREEN (round 3 confirm, 0C/0I) — implementation may begin
**Source grounding verified at:** mnemonic-gui `origin/master` = `020f765`; toolkit tag `mnemonic-toolkit-v0.52.0` @ `2cad4b7` (local binary `/scratch/code/shibboleth/mnemonic-toolkit/target/debug/mnemonic` = 0.52.0)
**Resolves:** `FOLLOWUPS.md::gui-build-descriptor-presets-pending-pin-bump` (this repo, root registry) + the toolkit companion (`mnemonic-toolkit/design/FOLLOWUPS.md`, same slug).
**Parent:** toolkit presets SPEC (`design/SPEC_descriptor_builder_presets.md`) §8 + allow SPEC §6; architect direction-consult (stream A1) incl. its Finding 1 (the repeating-row render gap).

## 0. Scope

One GUI MINOR cycle (v0.30.0), three legs + housekeeping:

1. **Pin bump v0.50.0 → v0.52.0** across the 6 lockstep sites.
2. **`BUILD_DESCRIPTOR_FLAGS` += 12** (the measured drift, exactly: `--after --allow --archetype --emit-spec --final-key --hash --key --older --recovery-key --recovery-older --recovery-threshold --threshold` — probed against the v0.52.0 binary; zero drift on every other subcommand).
3. **Generic repeating-row widget for NON-secret repeating flags** — without it, 3 of 5 archetypes (min 2 `--key`) cannot be driven from the form (consult Finding 1: `widget.rs` renders at most one row per flag name; the argv assembler already handles N rows).
4. Housekeeping: a cheap conditional rule (disable `--spec` when `--archetype` is set), record the remaining un-projected clap rules as ACCEPTED, file the **live repeating-secret bug** as its own FOLLOWUP (out of scope), GUI CHANGELOG + tag, toolkit follow-ups at ship.

**Non-goals:** the archetype-forms wizard (A2, v0.31.0 — consumes the `archetypes` schema section; NOT this cycle); any `--json` wire consumption change; fixing the repeating-secret emission bug (filed, not fixed); `docs/manual-gui` anchor debt (existing FOLLOWUP absorbs it).

## 1. Pin bump (6 sites; measured-clean discipline)

Sites (the v0.29.0 pattern): `Cargo.toml` toolkit dep tag, `Cargo.lock` (rev = the TAG COMMIT `2cad4b7`, not a recon SHA), `pinned-upstream.toml` `[mnemonic].tag` (the CI install source), `README.md` pin marker, the `pinned_version` banner in `src/schema/mnemonic.rs` (`:3764` at `020f765`), the module-doc `src/schema/mnemonic.rs:1` ("Pinned schema … from mnemonic-toolkit-v0.50.0" → v0.52.0). 4 gated by `pin_coherence`/`readme_pin_coherence`, 2 ungated (banner + module-doc) — grep-sweep `v0.50.0` over `src/` + `README.md` + `pinned-upstream.toml` at edit time to catch strays — EXPECT 3 historical version-stamped attribution comments (`mnemonic.rs:32/:3407/:3513`); per the v0.29.0 discipline those are left as-is (R0-r1 M3).

**Measured drift (recon, 2026-06-09):** `schema_mirror` vs the local v0.52.0 binary fails ONLY `mnemonic build-descriptor`, missing exactly the 12 flags above; `only in schema: []`; all other subcommands clean. The §2 adds therefore close the gate exactly.

## 2. Schema adds — `BUILD_DESCRIPTOR_FLAGS` += 12

| Flag | FlagKind | repeating | Notes |
|---|---|---|---|
| `--archetype` | `Dropdown(ARCHETYPES)` — new const **`["", "decaying-multisig","hashlock-gated","kofn-recovery","simple-timelocked-inheritance","tiered-recovery"]`** + `default_value: Some("")` (R0-r1 C1: the empty-string UNSET sentinel — seeded `Dropdown("")` is skipped by `emit_one`, `has_value` is false, and the user can re-select "(none)"; without it the default form emits a guaranteed-refusal `--archetype decaying-multisig` AND the §4 mutex deadlocks `--spec` from frame 1. The value-enum is NOT gate-checked (I1) so the sentinel option is safe) | false | toolkit `CliArchetype` order after the sentinel |
| `--key`, `--recovery-key` | `Text` | **true** | xpub strings (NOT Path — the `--spec` lesson cuts the other way); argv order is load-bearing for quorum order (row order = argv order) |
| `--threshold`, `--recovery-threshold` | `Number { min: 1, max: NumberMax::Static(20) }` | false | NOT `FromSlotCount` (build-descriptor has no slot grid); 20 = the `multi` CHECKMULTISIG ceiling — a UI affordance only, the toolkit gate is the validator |
| `--older`, `--recovery-older` | `Number { min: 1, max: NumberMax::Static(2_147_483_647) }` | false | toolkit step-1 bound `1 ≤ N < 2³¹` |
| `--after` | `Number { min: 1, max: NumberMax::Static(4_294_967_295) }` | false | clap accepts 0..=u32::MAX; GUI min 1 is deliberately tighter (`after(0)` is gate-invalid) — R0-r1 M8 |
| `--final-key`, `--hash` | `Text` | false | |
| `--emit-spec` | `Boolean` | false | |
| `--allow` | `Dropdown(ALLOW_RULES)` — new const `["malleable","mixed-timelock","repeated-keys","resource-limit","sigless-branch"]` | **true** | **repeating Dropdown — precedent EXISTS: `--to` (`Dropdown(NODE_TYPES)`, `repeating: true`)**; the FOLLOWUP's "combination not used before" claim was wrong — correct it when resolving |
| `--spec` | (existing `Path`) | — | unchanged |

`schema_mirror` is flag-NAME set-equality ONLY — kinds/repeating AND **dropdown values are NOT gate-checked** (R0-r1 I1: `schema_check.rs:97-104` deserializes names only; a const typo would be silent until runtime clap rejection). Pin: extend `tests/build_descriptor_schema.rs` to assert `ARCHETYPES` (minus the sentinel) and `ALLOW_RULES` byte-equal the v0.52.0 binary's value lists (probed: archetypes `decaying-multisig, hashlock-gated, kofn-recovery, simple-timelocked-inheritance, tiered-recovery`; allow `malleable, mixed-timelock, repeated-keys, resource-limit, sigless-branch`). A general choices-vs-gui-schema gate for all Dropdowns = a candidate FOLLOWUP, out of scope.

## 3. Generic repeating-row widget (non-secret)

**The gap (consult Finding 1, re-verified):** `render_with_dispatch` (`src/form/widget.rs`, non-secret path) does `position(|(k,_)| k == flag.name)` → renders exactly ONE row; no add-row affordance exists anywhere in `src/`. The assembler already emits every matching `state.values` row for `flag.repeating` (`src/form/invocation.rs`). Live today this limits `--md1`/`--cosigner`/`--mk1`/`--to`/`--group`/`--add-path`/`--target-address` to one row (`--slot` has its own SlotEditor branch) — and would cap `--key`/`--recovery-key`/`--allow` at one.

**Design:** in the non-secret path, branch on `flag.repeating`:
- Render EVERY `(k, v)` row in `state.values` with `k == flag.name`, each with its kind-appropriate widget + a per-row remove button (`✕`).
- One "+ add" button appends a row: `Text` rows seed empty; **`Dropdown` rows seed `Dropdown("")` (R0-r1 I3 — NOT `opts[0]`: an added-but-untouched `--allow` row must emit NOTHING; accidental emission of an allowance is a funds-safety opt-out).** `emit_one` skips empty Dropdown (verified `invocation.rs:316-317`). Cell: an added-untouched `--allow` row emits nothing.
- Zero rows = nothing emitted (assembler semantics unchanged). **Seed rule (R0-r1 C2 / R0-r3 M-i): ANY render observing zero rows for a required flag seeds one (per-frame condition, not a once-latch) — seeded with `default_flag_value_for_flag(flag)` (today's scalar seed: `Dropdown("phrase")` for `--to`, `Text("")` for the Text flags — NOT the add-row empty seed, R0-r2 M-A) — else NO row.** A blanket zero-seed regresses `convert --to` (required, repeating Dropdown — today seeded `"phrase"`, making the default convert form runnable); a blanket one-seed auto-emits optional flags. `--key`/`--recovery-key`/`--allow` are optional → zero-seeded (never auto-emit an allowance). Pin BOTH directions: a convert default-form cell (still emits `--to phrase`) and a build-descriptor cell (no `--key`/`--allow` rows seeded). Removing the LAST row of a required repeating flag respawns it next frame (the lazy seed re-fires) — INTENDED: a required flag always shows ≥1 row (R0-r2 M-C).
- Per-row egui ID salts (R0-r1 I4): thread the row index into the ComboBox/widget salts (`("flag_dropdown", flag.name, row_idx)` — N same-name rows otherwise share an ID, the v0.1.1 popup-state-leak class; `tests/dropdown_id_salt.rs` pins only the textual convention and will not catch it).
- **Empty-option display (R0-r2 I-1):** the Dropdown render arm maps the `""` option to the display label `"(none)"` for BOTH the popup row and `selected_text` — DISPLAY-ONLY (stored/emitted value stays `""`; `emit_one`/`is_at_default`/`has_value` key off the value). Without it the unset row is a ~4px sliver (`selectable_value` sizes to its text; the popup does not justify items) and — with `FormState` persisted per subcommand and no reset affordance — selecting an archetype once would near-permanently trap the user out of `--spec`. Safe to generalize: no existing Dropdown const contains `""`.
- Header row (R0-r1 I5): label + `?` help icon + "+ add" render REGARDLESS of row count (zero rows must not make the flag invisible or drop the help affordance; required marker on the header when `flag.required`).
- Per-row remove: collect remove-intents during the loop, apply after (R0-r1 M7 — the existing `transition` pattern; mutating `state.values` mid-iteration is a borrow error).
- Scalar (non-repeating) path byte-unchanged.
- Secret repeating flags: UNCHANGED this cycle (they take the `secret_widgets` branch before the repeating check — the live emission bug is filed, §5).

**Empty-row semantics (RESOLVED by R0-r1):** `emit_one` skips empty Text (`invocation.rs:307-308`) AND empty Dropdown (`:316-317`) — no assembly-time filter needed; pinned by the empty-row cell.

## 4. Conditional rule (the one cheap projection)

`build-descriptor`'s `SubcommandSchema.conditional` goes `None` → `Some(crate::form::conditional::build_descriptor)`: when `--archetype` has a non-default value, disable `--spec`; when `--spec` is non-empty, disable `--archetype`. (The fn-pointer pattern at `src/form/conditional.rs`; compare-cost mutex precedent.) The OTHER clap edges (10 `requires = "archetype"` params, `--emit-spec` conflicts) stay deliberately UN-projected — **recorded decision: the CLI is the gate; A2's wizard supersedes the generic form as the preset surface, so further projection investment is waste.** No conditional-rules drift-test arm is needed (resolved in §6: the toolkit emits `conditional_rules: []` for build-descriptor and the gate skips empty-rule subcommands).

## 5. FOLLOWUP filings (this cycle, both repos where noted)

- **NEW (GUI): `repeating-secret-flags-never-reach-argv`** — live bug, pre-existing: secret+repeating+Text flags (`import-wallet --ms1`, `seed-xor/slip39 --share`, …) render into the single `secret_widgets` entry (`widget.rs` secret branch) while `assemble_argv` reads repeating secrets from `state.values` (`invocation.rs` — the v0.3-fold comment documents the intended state.values routing) → a live form emits NOTHING for them. Masked by kittest cells that synthesize `state.values` directly. Fix direction: per-row `SecretLineEdit` rendering routed through `state.values` (the fold comment's design). Out of A1 scope (consult ruling).
- **Resolve** `gui-build-descriptor-presets-pending-pin-bump` in BOTH repos (note the corrected `--to` repeating-Dropdown precedent).
- The toolkit `manual-gui` anchor-debt FOLLOWUP grows by the 12 flags (existing entry absorbs; mention at resolve time).

## 6. Tests

- **`schema_mirror` GREEN at the bumped pin** — the 12 adds close the measured drift exactly; value-enums for `--archetype`/`--allow` match the binary.
- **Repeating-row kittest:** drive the build-descriptor form — add 3 `--key` rows + params, assert `assemble_argv` carries 3 `--key` occurrences in ROW ORDER; remove the middle row → 2, order preserved. One cell on an existing repeating flag (`--cosigner` or `--md1`) proving the widget generalizes (the sibling CLIs' repeating flags — `md --key`/`--fingerprint`, `mk --policy-id-stub`/`--from-md1` — gain multi-row UI through the same shared widget; R0-r1 M6).
- **Empty-row cell:** an added-then-left-empty row emits nothing (or is filtered — per the §3 verify).
- **Conditional cell:** UNSET archetype (the `""` sentinel) → `--spec` ENABLED and no `--archetype` in argv (the C1 regression pin); archetype selected → `--spec` disabled; **round-trip: re-select `"(none)"` → `--spec` re-enabled and `--archetype` gone from argv (R0-r2 I-1)**; spec non-empty → `--archetype` disabled.
- **Drift gates re-run at the bumped pin:** full suite with `MNEMONIC_BIN`/`MD_BIN`/`MS_BIN`/`MK_BIN` (the `md`-is-mkdir-alias trap); `pin_coherence` + `readme_pin_coherence`; canonicity per-fixture table; `schema_mirror_secret_drift` (the GUI-side secret gate — R0-r1 M2 corrected from the toolkit-side lint name; probed GREEN at v0.52.0: the 12 new flags are all non-secret — keys are xpubs, `--hash` is a digest).
- **`tests/build_descriptor_schema.rs` UPDATE owned by this cycle (R0-r1 I2):** the existing cell pins the exact 6-flag set + `conditional.is_none()` — both go RED; update to the 18-flag set + `conditional.is_some()` + rename off `v0_50_0` + the I1 value-list pins.
- **No `EXPECTED_ARM_COUNT`** (toolkit concept, absent GUI-side); **no `SUBCOMMAND_FLOORS` entry and no rule-count update needed** (R0-r1 resolved: the toolkit emits `conditional_rules: []` for build-descriptor and the drift gate skips empty-rule subcommands); ADD `build-descriptor` to `tests/conditional_visibility.rs::coverage_all_constrained_subcommands_have_conditional_fn` (R0-r1 M4).

## 7. Release (GUI ritual)

GUI CHANGELOG **is** maintained — add `[0.30.0]`. Bump `Cargo.toml` version 0.29.0 → 0.30.0 (+ lock). Full suite green (with the 4 binaries) → push → GUI CI green (build + schema-mirror push-triggered) → tag `mnemonic-gui-v0.30.0` → tag-build green. **Toolkit follow-ups:** `scripts/install.sh:44` GUI pin bump commit (`chore(install): bump mnemonic-gui pin`, RELEASE_CHECKLIST:66 "GUI release" item — NOTE the pin is currently `mnemonic-gui-v0.21.1`, EIGHT releases stale; say so in the commit — R0-r1 M5) + flip both repos' FOLLOWUP entries.

## 8. Source grounding (verified at GUI `020f765` / toolkit `2cad4b7`)

- Measured drift: probe transcript in this SPEC §1 (schema_mirror panic message, 12 flags, `only in schema: []`).
- `src/form/widget.rs` — secret branch (`flag_is_secret` + `FlagKind::Text` → `secret_widgets`); non-secret single-row `position()` lookup + write-back.
- `src/form/invocation.rs` — `assemble_argv` repeating loops over `state.values` (secret AND non-secret); the v0.3-fold comment on repeating-secret routing.
- `src/schema/mod.rs` — `FlagKind` (incl. `Number{min, max: NumberMax::{Static, FromSlotCount}}`), `FlagSchema.repeating`, `SubcommandSchema.conditional: Option<fn(&FormState) -> FlagVisibility>`.
- `src/schema/mnemonic.rs` — `BUILD_DESCRIPTOR_FLAGS` (v0.29.0, 6 flags); module-doc `:1` "from mnemonic-toolkit-v0.50.0"; `pinned_version` banner `:3764`; `--to` = the live repeating-Dropdown precedent; repeating+secret census (R0-r1 M1): `--ms1` 2 sites / `--share` **3** Text sites secret:true.
- `src/secrets.rs` — `flag_is_secret` (`flag.secret || SECRET_FLAG_NAMES`).
- Toolkit binary 0.52.0 `build-descriptor --help` — the 12-flag surface + value lists for `--archetype`/`--allow`.

---

## Fold log

- **R0 round 1 (RED → folded, 2026-06-09; persisted at `design/agent-reports/gui-v0_30_0-presets-pin-bump-r0-r1-review.md`):** C1 `--archetype` gains the `""` UNSET sentinel (prepended to `ARCHETYPES` + `default_value: Some("")`) — kills both the guaranteed-refusal default emit and the permanent `--spec` deadlock; conditional cell pins the unset state. C2 seed rule = ONE row iff `flag.required` (preserves `convert --to`), zero for optional; both directions pinned. I1 value-enums are NOT gate-checked — claim corrected + byte-equal const pins added to `tests/build_descriptor_schema.rs`. I2 that test's 6-flag/`conditional.is_none()` characterization update owned. I3 add-row Dropdown seeds EMPTY (an untouched `--allow` row must emit nothing). I4 row index threaded into egui ID salts. I5 header row (label/?/add) renders at zero rows. M1 `--share` = 3 sites. M2 GUI secret gate = `schema_mirror_secret_drift`. M3 banner `:3764`; 3 historical comments left. M4 conditional_visibility coverage list grows. M5 toolkit install.sh GUI pin is 8 releases stale — note at ship. M6 sibling-CLI repeating flags noted. M7 collect-then-apply row removal. M8 `--after` min-1 intent stated.
- **R0 round 2 (YELLOW → folded, 2026-06-09; persisted at `design/agent-reports/gui-v0_30_0-presets-pin-bump-r0-r2-review.md`):** all 15 round-1 folds verified (C1 sentinel traced end-to-end through seed/suppress/emit/mutex). I-1 the `""` option gains a display-only `"(none)"` label in the Dropdown render arm (sliver-row + persisted-state trap) + round-trip conditional cell. M-A required-row seed pinned to `default_flag_value_for_flag`. M-B §8 banner cite aligned to `:3764`. M-C required-last-row respawn declared intended. M-D §4 hedge collapsed into the §6 resolution.
- **R0 round 3 (GREEN confirm, 2026-06-09; persisted at `design/agent-reports/gui-v0_30_0-presets-pin-bump-r0-r3-review.md`):** all 5 round-2 folds verified; the I-1 "(none)" mapping lands cleanly in the single Dropdown render arm (both display sites, value untouched). M-i wording polish folded (seed condition = per-frame zero-rows-and-required). **Gate satisfied.**
