# R0 review — SPEC_gui_automated_ui_test_harness.md (round 1)

**Reviewer:** opus architect (adversarial). **Gate:** 0 Critical / 0 Important.
**Target:** `design/SPEC_gui_automated_ui_test_harness.md`
**Verified against:** `mnemonic-gui master @ da47994` source (not the spec's claims).

---

## Verdict

**RED — 0 Critical / 6 Important / 5 Minor-Nit.**

The altitude, tool (schema enumeration over `egui_kittest`), and the central thesis
(render-via-kittest is MANDATORY; a hand-built-`FormState` test is structurally blind)
are **correct and well-argued — I verified them against the real v0.31.1 history and
the existing anti-patterns in-tree.** All the load-bearing *factual* premises the spec
rests on are TRUE (kittest 0.31 dep, headless `cargo test --workspace` CI, `secret_widgets`
serde-skip, 61 subcommands, the dead-bug anchor correctly killed). This is **not** a
redesign. But six precision/scope/feasibility defects must close before a plan-doc:
the co-headline secret invariant (I3) rests on a false coverage claim, the widget-kind
taxonomy is wrong/incomplete vs the actual `FlagKind`, the generation model is ambiguous
in a way that reintroduces exactly the blindness §4 forbids, the non-Text injection
feasibility is unproven, and two of the I2 invariants are ill-fenced against the
documented `Visibility` semantics.

---

## Critical

**None.** No single false premise invalidates the approach. (See I1 below — it borders
Critical and *would* become Critical if the team intends I3 to **replace** the existing
classification-drift gate rather than complement it.)

---

## Important

### IMP-1 — I3's co-headline claim "catches the unclassified-secret class **by construction**" is FALSE
Spec §1:16-17 and §I3:74-81 ("Catches the 'unclassified secret leaks to disk' class
**by construction** (the class behind the two v0.31.1 incidental leaks)").

The sweep, as defined, iterates **only flags already classified `secret == true`** ("For
every secret-classified flag × every subcommand", :74). An *unclassified* secret — the
actual v0.31.1 class (`xpub-search-inline-phrase-not-secret-classified`,
`ms-repair-ms1-not-secret-classified`) — is by definition **not in that iteration set**:
it renders as an ordinary `Text` widget, its value lands in `state.values`, and that
value is *supposed* to persist (autosave). So a "type fixture → serialize → assert-absent"
sweep **cannot** observe it. The class it actually catches is the **inverse**: a flag that
*is* classified secret nonetheless leaking because routing/redaction broke.

The unclassified class is **already owned** by existing gates, verified in-tree:
- `tests/schema_mirror_secret_drift.rs:84-124` — set-equality of GUI hand-coded
  `FlagSchema.secret==true` vs the toolkit's ground-truth `flag_is_secret` projection
  (`mnemonic gui-schema` v5 `secret` field). A GUI under-classification fires RED here.
  Its own header (:15-17) names this as "the v0.3.0..v0.3.2 BIP-39 persistence leak class."
- `tests/secret_taxonomy_pin.rs` (companion — NodeType/SlotSubkey constants).

**Action:** reframe I3 honestly as a **routing/redaction regression net over
already-classified secrets across ALL persistence surfaces** (flags + secret positionals
+ tree keys + slot subkeys) — that *is* genuinely valuable and new-as-universal — and
strike the "by construction / the v0.31.1 class" justification, OR re-attribute that class
to `schema_mirror_secret_drift` + `secret_taxonomy_pin`. As written, the spec gives a
self-custody tool false confidence that its highest-value secret gate covers a class it is
structurally blind to. (Borders Critical: if a plan reads this as license to retire the
classification-drift gate, the actual hole opens.)

### IMP-2 — Widget-kind taxonomy is wrong and incomplete vs the real `FlagKind`
Spec §1:20 ("10 widget kinds") and §3:44-45 list transform kinds as
"NodeValueComposite, TaggedOrIndexed, Range, Timestamp, **SlotEditor, Tree**."

Actual `FlagKind` (`src/schema/mod.rs:142-167`) has **9** variants:
`Text, Number, Dropdown, Boolean, Range, Timestamp, NodeValueComposite, TaggedOrIndexed,
Path`. Therefore:
- **`SlotEditor` and `Tree` are NOT `FlagKind` variants.** They are separate `FormState`
  surfaces — `slots: SlotState` (`mod.rs:293`) and `tree: Option<TreeState>`
  (`mod.rs:333`) — driven by their own sub-forms (`form/slot_editor.rs`,
  `form/tree_form.rs`/`tree_model.rs`). They are **not** enumerable from
  `(cli_tab, subcommand, flag)` the way §3:31 / §5 assume; the schema exposes them via
  `SubcommandSchema.allows_slots` and a separate tree builder, not as flags.
- **`FlagKind::Path` is omitted entirely** from the identity/transform partition — yet
  the spec itself references the Path-with-stdio-sentinel surface (`--spec -`) as an I3
  target (:81 region / point in §I3). Path with `stdio_sentinel` emits its value verbatim
  (`mod.rs:164-166`) and is arguably identity, but the spec never places it.

**Action:** correct the taxonomy to the 9 real `FlagKind`s; classify `Path`; and state
explicitly how `slots`/`tree` are enumerated and driven (they need bespoke sub-form
drivers, not the flag loop). This is load-bearing for §5's "enumerate from
`(cli_tab, subcommand, flag)`."

### IMP-3 — §5 generation model reintroduces the very I1 blindness §4 forbids
Spec §5:90-95 — "Property/sweep code **varies only leaf values** on top of a valid base."

If "varies leaf values" means **mutating the hand-built `FormState` directly**, the I1
cell collapses into `assemble_argv(hand-built state)` — the exact "structurally BLIND"
pattern §4 I1 (:52-59) declares forbidden. This is not hypothetical: it is precisely how
the existing tree already cheats —
- `tests/widget_interaction.rs:113-129,151,166-168,…` drives **synthetic buttons** whose
  closures mutate `FormState` (`if ui.button("set-template-bip84").clicked() { …set
  --template… }`), never the real Text/Dropdown widget;
- `tests/archetype_form.rs:238-247` `select_archetype()` **writes `state.values`
  directly**, with the comment "what selecting a row in the combobox popup does; the
  popup-interaction path itself is pinned by [a display-label test]."

So the spec must **require** that each I1 cell injects its varied leaf **through the
rendered widget via kittest** (seed the valid base into `Harness::new_ui_state`, then
`focus()`+`type_text()`/`click()`/select on the *one* varied widget, `run()` to settle,
read back `harness.state()`), exactly as `tests/repeating_secret_rows.rs:95-114` does.
As written, §5 leaves the door open to a vacuous sweep.

### IMP-4 — Non-`Text` identity-kind injection (ComboBox / DragValue) is unproven and known-hard
Spec §3:44 buckets **Dropdown** and **Number** as identity-mapped kinds driven by "a real
keystroke/**selection**." Reality in-tree:
- The **only** real-widget value injection that exists is `type_text` into a `TextEdit`,
  and `tests/repeating_secret_rows.rs:13-17` calls it "first in-repo use."
- **Dropdown** renders as `egui::ComboBox` + popup `selectable_value` (`widget.rs:561-593`).
  **No test drives a real option-selection through the popup via kittest** —
  `archetype_form.rs:238-247` deliberately substitutes direct state-mutation; egui
  ComboBox popups (a separate `Area` materialized only after a click, with cross-frame
  AccessKit exposure) are a known kittest pain point.
- **Number / Range / Timestamp** render as `egui::DragValue` (`widget.rs:544,599-601,616`).
  Setting a `DragValue` to a *distinguishable* value via kittest is not a plain `type_text`
  (drag / click-to-edit) and has **no in-repo precedent**.
- Additionally, Number/Range/Timestamp/TaggedOrIndexed start `Unset` behind a **"Set"
  button** (`widget.rs:519-540`) and repeating rows behind **"+ add"** — each adds a click
  + extra frame before the value widget exists.

Many flags are Dropdown/Number. If kittest 0.31 can't drive them, they fall back to
hand-cells = the IMP-3/§4 blind pattern, and the universal-I1 headline + the ~800–1500-line
estimate (§5:93) both break. **Action:** the plan MUST spike ComboBox-popup-selection and
DragValue-set injection in kittest 0.31 **before** committing to the universal I1 sweep.
(Escalates to Critical if the spike proves either infeasible.)

### IMP-5 — I2's "renderer-applies-the-rule" and "disabled-value suppression" don't cover all 5 `Visibility` effects
`conditional()` returns `Vec<(flag, Visibility)>` over **5** variants — `Hidden, Disabled,
Required, PinValue, DisableOptions` (`mod.rs:248-270`), all live (`conditional.rs:205-257,
402-422` push `Disabled`/`PinValue`; v0.7.x `DisableOptions`). The assembler's behavior
(`invocation.rs:181-189`) is: **suppress = `Hidden | Disabled` only**; **`PinValue`
REPLACES** the value and emits; **`DisableOptions` emits the stale value by design**
(CLI backstop, :181-186).

(a) §I2:62-65 "rendered visibility/enable/disable state of each flag equals
`conditional(state)`" is AccessKit-queryable for `Hidden` (node absent) and `Disabled`
(`.is_disabled()`, proven by greyout `T2`, `greyout_stdin_toggles_v0_37_0.rs:108-113`) —
but **not** cleanly for `PinValue` (renders *read-only*, not disabled/hidden) or
`DisableOptions` (per-option greyout inside an unopened combo popup). The gate as sold
("of each flag") can only verify 3/5 effects via the tree.

(b) §I2:70-72 "a value entered then hidden/**disabled** must NOT reach argv …
universalized to ALL subcommands" must be **fenced to `Hidden | Disabled` ONLY**.
`DisableOptions` literally contains "Disable" yet **intentionally emits** the stale value;
`PinValue` **intentionally emits** a replaced value. A naive universalization mirrors the
wrong predicate and **false-reds the CI gate on documented-correct behavior**.

**Action:** scope the renderer-faithfulness gate to `{Hidden, Disabled}` (state
PinValue/DisableOptions are verified via assembler-emission cells / CLI backstop, not the
AccessKit query); fence the suppression property to `Hidden | Disabled` mirroring
`invocation.rs:189`'s `suppresses` predicate. §7 must admit both gaps.

### IMP-6 — I2 "toggle round-trip" leaves the equivalence relation undefined (and self-contradictory)
§I2:67-69 — "toggle on→off returns the form to an **equivalent state**; **no orphaned
values**, no stuck visibility." "Equivalent" is never defined, and it contradicts the
spec's own acknowledgement that some toggles **legitimately destroy data** (gating a field
off may hide+clear a dependent; toggling back on, it's empty). As written the property
false-fails on correct behavior.

The consult scopes this correctly to **visibility-state** equivalence (consult :143-145,
"toggle A on then off returns visibility to baseline") — well-defined and immune to
legitimate value destruction. The spec broadened it to value-level "no orphaned values"
without specifying the relation. **Action:** pin the equivalence relation to *the
`conditional(state)` visibility output before/after the round-trip*, explicitly **NOT**
value-equivalence; and disentangle "no orphaned values" (which is really the IMP-5(b)
suppression property) from "no stuck visibility."

---

## Minor / Nit

- **MIN-1 (determinism vector mislabeled).** §9:122-125 cites "deterministic seeds for the
  gate" but the real flake vector in a kittest gate is the **multi-frame settle**, not RNG
  — `repeating_secret_rows.rs:112` ("settle: buffer write-back happens at frame end"),
  `:141` ("seed lands during frame 1; row renders frame 2"). §9 should mandate a
  run-until-stable / fixed-frame discipline. (Manageable — existing kittest CI is green —
  but unacknowledged where the spec leans hardest on flake-aversion.)
- **MIN-2 (I2 surface overstated).** Only **17 / 61** subcommands have `conditional: Some`
  (47 are `None`); the renderer-desync gate's real surface is those 17. Worth stating so
  the gate isn't sold as "all 61."
- **MIN-3 ("purity" oversold).** §I2:66 "purity" is near-trivial for a plain
  `fn(&FormState) -> FlagVisibility` (no globals/interior-mutability in `conditional.rs`).
  Fine as a cheap guard, but not a "genuinely NEW" headline.
- **MIN-4 (I3 entry point vague).** "drive the persistence/serialization walk" (:76-77)
  should name the real surfaces: secret positionals live under the
  `secret_widgets["positional:<name>"]` reserved key (`mod.rs:71-76`), and `tree` xprv keys
  **persist-then-redact** via `TreeState::redacted_for_persistence` (`mod.rs:328-333`) —
  they are NOT serde-skipped. So the assertion must go through `redact_for_persistence`,
  not raw serde, or it misses the tree/slot surfaces.
- **MIN-5 (motivation stat).** §1:15 "~47 of 61 have no full-flow test" is a 61−14
  hand-wave (kittest files ≠ subcommands 1:1). Harmless; label it approximate.

---

## Verified TRUE (credit — these premises are solid, do not relitigate)

- egui + `egui_kittest` = **0.31** dev-dep (`Cargo.toml:17,77`). ✓
- Headless CI is `cargo test --workspace` with `MNEMONIC_BIN`/`MD_BIN`/`MS_BIN`/`MK_BIN`
  set, `ubuntu-latest`, **no wgpu/xvfb** (`schema-mirror.yml:111-133`). ✓
- `secret_widgets: BTreeMap<String, Vec<SecretLineEdit>>` `#[serde(skip)]`, per-row
  `Zeroizing`/non-`Clone` (`mod.rs:303-322`). ✓ Respect-don't-fix is correct.
- `repeating-secret-flags-never-reach-argv` **RESOLVED v0.31.1**, kittest-pinned — the
  spec **correctly kills** the consult's dead-bug anchor and reframes around enumeration +
  the universal sweep. ✓
- Subcommand count **61 = mnemonic 32 + ms 10 + mk 9 + md 10** (counted `SubcommandSchema {`
  literals per `src/schema/*.rs`). ✓
- **Core I1 thesis is correct.** Render-via-kittest MANDATORY; `assemble_argv(hand-built
  state)` is structurally blind — matches the real v0.31.1 masking-cell history and the
  in-tree `widget_interaction.rs` synthetic-button + `archetype_form.rs::select_archetype`
  direct-mutation anti-patterns. The render→store-seam vs downstream-of-store boundary
  (§4:56-59) is precise and implementable. ✓
- **Layered oracle is genuinely non-tautological** for the wiring class: names → existing
  `schema_mirror` (real clap), wiring → identity round-trip on identity-mapped kinds only,
  functional → real pinned CLI. Correctly excludes transform kinds from the generic
  identity property. ✓
- I3 hygiene rules — fake fixtures only, **no AccessKit/state dump on failure** (undo-ring
  plaintext, `gui-secret-buffer-allocator-residue`), respect `Zeroizing` — correct and
  grounded. ✓
- deterministic-table-for-gate vs proptest-for-sweep split is right for this flake-averse
  project (mlock g4_a history). ✓

---

## Bottom line

Sound design, right altitude, factual premises verified TRUE — but **RED at 0C/0I**.
Close IMP-1 (reframe the co-headline secret claim — it's false as written), IMP-2 (real
`FlagKind` taxonomy incl. Path; slots/tree are not flags), IMP-3 (require widget-driven
leaf injection), IMP-4 (spike ComboBox/DragValue kittest injection — the headline's
feasibility crux), and IMP-5/IMP-6 (fence the I2 invariants to the documented `Visibility`
semantics + define the equivalence relation). Then re-dispatch.
