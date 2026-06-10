# R0 review — SPEC_gui_v0_30_0_presets_pin_bump — round 1
**Verdict: RED**

Scope, pin mechanics, the measured-drift claim, the flag table, the bug filing, and the test/release ritual are all sound and were empirically reproduced. RED is driven by two Critical design gaps in §2/§3/§4 that, implemented as written, ship a deadlocked build-descriptor form and regress the existing convert form. Both have small, local fixes; one revision round should reach GREEN.

## Critical

**C1. `--archetype` has no unset representation — the default form emits a guaranteed-refusal argv AND the §4 mutex deadlocks `--spec` permanently.**
- `--archetype` is a non-repeating `Dropdown(ARCHETYPES)` with no toolkit default (`gui-schema` probe: `default: None`). On first render the scalar path pushes a seeded value: `src/form/widget.rs:101-110` → `default_flag_value_for_flag` → `default_flag_value_for` seeds `Dropdown(opts.first())` = `"decaying-multisig"` (widget.rs:133-135).
- Emission: `emit_one`'s Dropdown arm emits any non-empty value with no declared default (`src/form/invocation.rs:316-318`, `is_at_default` returns false at `default_value: None`, invocation.rs:46-48). So the default v0.30.0 build-descriptor form emits `--archetype decaying-multisig` with no params → toolkit refusal every time (probed). This breaks the v0.29.0-shipped `--spec` workflow.
- Deadlock: `has_value("--archetype")` is true for any non-empty Dropdown (`src/schema/mod.rs:405-407`), i.e. true from frame 1. The §4 rule "archetype set → disable `--spec`" therefore disables `--spec` permanently (and `Disabled` also suppresses `--spec` from argv, invocation.rs:160-177). The SPEC's phrase "non-default value" is undefined for this flag; every reading is broken.
- Required fix (pick one, specify it): (a) empty-string sentinel — seed `Dropdown("")`; `emit_one` already skips empty Dropdown (invocation.rs:316-317) and `has_value` is false for empty; either prepend `""` to `ARCHETYPES` or set `default_value: Some("")` (the value-enum is NOT gate-checked, see I1); or (b) extend `FlagValue::Unset` to Dropdown (bigger: the mismatch-recovery branch at widget.rs:378-384 re-seeds Unset→default for Dropdown). The §6 conditional cell must then assert the unset state leaves BOTH `--spec` enabled and `--archetype` un-emitted.

**C2. §3 "the first render of a repeating flag seeds NO row" regresses `convert --to` (required, repeating Dropdown).**
- `--to` on convert: `required: true, repeating: true, Dropdown(NODE_TYPES)` (`src/schema/mnemonic.rs:755-764`). Today the scalar path seeds one row `Dropdown("phrase")` and the repeating assembler loop emits `--to phrase` — the default convert form is runnable. Under the blanket zero-seed rule, the default convert form emits no `--to` → clap "required argument missing" on every run until the user discovers "+ add". No test pins this (all repeating-flag tests synthesize `state.values` directly), so it would ship silently.
- Other required repeating flags are unaffected (`--group` mnemonic.rs:1271 and `--target-address` :2610 are Text — empty Text never emitted today either; full census via probe: only `--to` regresses).
- Fix: seed ONE row when `flag.required` (preserves `--to`; `--allow`/`--key`/`--recovery-key` are all optional so the zero-seed motivation — never auto-emit an allowance — is preserved), and pin the rule with a cell on convert's default form.

## Important

**I1. SPEC §2:39 "the dropdown value enums for `--archetype`/`--allow` ARE gate-checked (value-enum comparison)" is FALSE.**
`schema_mirror` consumes only the flag `name`; `src/schema_check.rs:97-104`: "Other fields (required, kind, choices) intentionally not deserialized". No other test compares GUI Dropdown values against gui-schema `choices` or `--help` possible-values (repo-wide grep). Consequence: a typo in `ARCHETYPES`/`ALLOW_RULES` is silent until runtime clap rejection. The SPEC must (a) correct the claim and (b) add a pin — cheapest: extend `tests/build_descriptor_schema.rs` to assert both consts byte-equal to the probed values (archetypes `decaying-multisig, hashlock-gated, kofn-recovery, simple-timelocked-inheritance, tiered-recovery`; allow `malleable, mixed-timelock, repeated-keys, resource-limit, sigless-branch`); better: a real choices-vs-gui-schema comparison for all Dropdowns (closes the class).

**I2. `tests/build_descriptor_schema.rs` breaks and the SPEC's §6 test list omits it.**
`build_descriptor_flag_set_matches_v0_50_0_surface` pins the exact 6-flag set (`tests/build_descriptor_schema.rs:28-44`) and asserts `sub.conditional.is_none()` (:47). Both assertions go RED this cycle. The SPEC must own updating this characterization (18-flag set, `conditional.is_some()`, rename off "v0_50_0").

**I3. An added-but-untouched `--allow` row silently emits `--allow malleable` — on the one flag where accidental emission is a funds-safety opt-out.**
"+ add" per §3 seeds `default_flag_value_for_flag` → Dropdown rows seed `opts[0]`, never empty; `emit_one` emits any non-empty Dropdown. The §6 "empty-row emits nothing" cell only holds for Text (`emit_one` Text guard, invocation.rs:307-308 — that §3 deferred verify RESOLVES YES). The SPEC must define add-row semantics for repeating Dropdowns: seed empty (`Dropdown("")`, skipped by emit_one) and pin a cell that an added-untouched `--allow` row emits nothing.

**I4. Per-row egui ID collision in the repeating widget.**
`render()` salts ComboBoxes with `("flag_dropdown", flag.name)` (`src/form/widget.rs:276`); N rows of the same repeating Dropdown (`--allow`) share an ID — exactly the v0.1.1 popup/hover-state-leak bug class documented in `tests/dropdown_id_salt.rs:1-17` (that test pins the textual convention only and will NOT catch the per-row collision). The §3 design must thread a row index into the salt (and into `("flag_tagged", …)`/`("flag_nodevalue", …)` if those kinds ever repeat).

**I5. Zero-row render state loses the flag's label, `?` help icon, and required marker.**
Label/help-icon/asterisk render inside `render()`'s per-row horizontal (widget.rs:233-235, 386-388), and `needs_help_icon` returns true for ALL repeating flags (widget.rs:27-32). With zero rows nothing renders — the flag becomes invisible and the manual-help affordance vanishes. The widget needs a header row (label + `?` + "+ add") rendered regardless of row count.

## Minor

- **M1.** §8 census: `--share` has **3** repeating+secret Text sites (mnemonic.rs:1319, 1457, 1562), not 2. `--ms1` "2 sites" is correct for repeating+secret (:638, :2060; 5 further non-repeating secret sites exist).
- **M2.** §6 lists `lint_argv_secret_flags` among GUI gates — it is toolkit-side. The GUI analogue is `tests/schema_mirror_secret_drift.rs` (probed GREEN at v0.52.0 — the 12 flags are all `secret` absent/false in gui-schema).
- **M3.** The §1 grep-sweep over `src/` will hit 3 historical `v0.50.0` attribution comments (mnemonic.rs:32, :3407, :3513) besides the 2 ungated sites — "historical version-stamped notes — leave". (Banner is actually at mnemonic.rs:3764, not ~3690.)
- **M4.** Add `build-descriptor` to the must-have list in `tests/conditional_visibility.rs::coverage_all_constrained_subcommands_have_conditional_fn` (:324-333) once the conditional lands.
- **M5.** §7: the checklist is `mnemonic-toolkit/design/RELEASE_CHECKLIST.md:66`. The pin at install.sh:44 is currently `mnemonic-gui-v0.21.1` — 8 GUI releases stale; the ship-time bump jumps to v0.30.0 and the lapse is worth a line in the toolkit commit.
- **M6.** §3's affected-flag list omits the sibling-CLI repeating flags that also gain multi-row UI: `md --key`/`--fingerprint`, `mk --policy-id-stub`/`--from-md1`. The "widget generalizes" cell could use one.
- **M7.** Per-row remove: collect the remove-intent and apply after the row loop (the existing `transition` pattern, widget.rs:229-232/390-392) — mutating `state.values` mid-iteration is a borrow error.
- **M8.** Number bounds (probed): `--after` clap bound is `0..=4294967295` — GUI `min: 1` is tighter than clap (defensible: `after(0)` is invalid miniscript; state the intent). `--older`/`--threshold` have NO clap bounds — the Static maxes are pure UI affordances exactly as the SPEC frames them.

## Citation audit

| SPEC claim | Verdict | Evidence |
|---|---|---|
| §1 exactly 6 pin sites | TRUE | grep `v0.50.0`: Cargo.toml:42, Cargo.lock:2296-2297, pinned-upstream.toml:22, README.md:50, mnemonic.rs:1, mnemonic.rs:3764 (+3 historical comments, M3) |
| §1 Cargo.lock rev = tag commit 2cad4b7 | TRUE | `git rev-parse mnemonic-toolkit-v0.52.0^{commit}` = `2cad4b71a6…` |
| §1/§2 measured drift = exactly the 12 flags | TRUE | reproduced verbatim |
| §2 flag names + both value lists + order | TRUE | `--help` + `gui-schema` agree byte-for-byte |
| §2 value-enums gate-checked | **FALSE** | schema_check.rs:97-104 — I1 |
| §2 `--to` repeating-Dropdown precedent | TRUE | mnemonic.rs:755-764 |
| §3 single-row gap / N-row assembler / emit_one empty-Text skip | TRUE / TRUE / RESOLVED-TRUE | widget.rs:101-110; invocation.rs:255-258; :307-308 |
| §3 no kittest breaks | TRUE | repeating-flag cells synthesize state.values |
| §4 Disabled-mutex expressible | TRUE (but C1) | Visibility::Disabled mod.rs:209; conditional.rs:593-598 precedent |
| §4 rule-count test needed | NOT required | toolkit emits `conditional_rules: []`; drift gate skips empty; no SUBCOMMAND_FLOORS entry |
| §5 repeating-secret bug real | TRUE | widget.rs:76-85 vs invocation.rs:238-242; nothing copies secret_widgets → state.values |
| §6 no pinned toolkit diagnostic strings | TRUE | grep "rerun with": zero hits |
| §7 CHANGELOG/ritual | TRUE | head `[0.29.0]`; install.sh:44 (M5) |
| §8 `--share` 2 sites | **FALSE** | 3 sites (M1) |

## Empirical probes run

1. `mnemonic --version` → 0.52.0; `git tag --points-at 2cad4b7` → mnemonic-toolkit-v0.52.0.
2. `build-descriptor --help` → the 12 flags; value lists match §2 in content and order.
3. `gui-schema` build-descriptor → `conditional_rules: []`; no `secret`/`default_value` on all 18 flags.
4. schema_mirror vs v0.52.0 → FAILED with exactly the 12 flags, `only in schema: []`.
5. canonicity_drift (1 passed), gui_schema_conditional_drift (5 passed), schema_mirror_secret_drift (1 passed), xpub_search_schema_mirror (9 passed) — all GREEN at v0.52.0.
6. clap bounds: `--after` 0..=4294967295; `--older`/`--threshold` unbounded at clap (gate validates).
7. `--archetype kofn-recovery` alone → refusal, 3 param diagnostics (C1 evidence).
8. grep sweeps: v0.50.0 census; "rerun with" (none); secret_widgets write paths (none); repeating/required/secret census over all four schema files.
