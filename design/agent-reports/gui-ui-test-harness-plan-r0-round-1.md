# R0 review — IMPLEMENTATION_PLAN_gui_ui_test_harness.md (round 1)

**Reviewer:** opus architect (adversarial). **Gate:** 0 Critical / 0 Important — NO code before GREEN.
**Target:** `design/IMPLEMENTATION_PLAN_gui_ui_test_harness.md` (draft → plan-R0).
**Spec status:** R0-GREEN round-2 (1 carried Minor m1, folded into the plan's P2).
**Verified against:** live `mnemonic-gui master` source + the pinned `egui_kittest 0.31.1` /
`kittest 0.1.0` / `egui 0.31.1` crate sources in the cargo registry (not the plan's claims).

---

## Verdict

**GREEN — 0 Critical / 0 Important / 7 Minor.**

The plan executes the R0-GREEN spec correctly, completely, and in a buildable order. The phase DAG
is sound (P0 spike gates I1 reach → P1 builds the enumerator/seed-table on a slice → P2/P3 layer the
metamorphics and secret net → P4 the real-CLI cells → P5 sweeps → P6 freezes the deterministic gate),
every spec invariant class (I1→P1, I2→P2, I3→P3, I4→P4) is assigned, the carried Minor m1 is folded
into P2 per-effect, the post-impl whole-diff review is correctly placed BEFORE merge, and the ship
vehicle (PR+CI, no tag, no `cargo fmt`) matches the GUI repo's gates.

**On the load-bearing question (is P0 likely to fail and collapse the plan?) — NO.** I independently
verified the egui_kittest API against source. Three of the four spike kinds are already
de-risked: **Dropdown driving is PROVEN in-tree** (`tests/tree_form.rs:669-675` opens a ComboBox by
role, clicks the option label, and asserts the *stored state* mutated), Boolean is a trivial
`Action::Click` toggle, and Path is `type_text`+button (both proven primitives). The **only genuine
unknown is Number/DragValue** — and even there a working path exists in egui 0.31, AND the plan's
descope fallback (hand-cell via state-mutation, exactly as `tests/archetype_form.rs:238-247` already
does) means a Number-spike failure narrows enumerated I1 by one kind rather than collapsing the plan.
The spike-gate architecture is exactly right. Details in the next section.

The 7 Minors are clarifications and one wording-contradiction; none forces mid-build rework, and the
mandatory per-phase R0 loop catches the two most consequential at P1/P2 design time. Cleared to
implement, starting with P0.

---

## P0 feasibility adjudication (the load-bearing risk gate — prompt item 2)

I traced the actual kittest/egui driving primitives, since the whole plan scales on P0's outcome.

**kittest 0.1.0 `Node` action API** (`node.rs`): `focus()` (:71, `Action::Focus`), `click()` (:80,
`Action::Click`), `hover()` (:89, cursor-move to center), `simulate_click()` (:98, positional
press/release), `type_text()` (:109, focus + IME text), `key_press`/`key_down`/`key_up`/`press_keys`/
`key_combination` (:143-179). **There is NO `drag()` and NO `set_value()` helper** — only click,
positional click, text/IME, and key events.

| Spike kind | Feasible? | Evidence |
|---|---|---|
| **Dropdown** (ComboBox open + select) | **PROVEN — already done in-tree** | `tests/tree_form.rs:669-671` `get_by_role(Role::ComboBox).click()` → `run()` → `get_by_label("andor").click()` → asserts `tree.root.kind=="andor"` (:674-675). `tests/repeating_rows.rs:386-390` opens the `--archetype` popup the same way. |
| **Boolean** (checkbox toggle) | **Very low risk** | `node.click()` issues `Action::Click`; egui checkboxes toggle on click. Buttons clicked this way across ~14 test files. |
| **Path** (text + sentinel) | **Low risk** | `type_text` into TextEdit is the one proven primitive; sentinel is a button `.click()` (cf. `kittest_import_wallet_form.rs:111`). |
| **Number** (DragValue set) | **GENUINE UNKNOWN — but a path exists; non-collapsing** | No `drag`/`set_value` helper. egui 0.31 `DragValue` *does* support driving (see below); none is proven in-tree. |

**Number/DragValue — the one real unknown, resolved to "feasible, spike-must-confirm-the-primitive":**
egui 0.31's `DragValue` (`egui-0.31.1/src/widgets/drag_value.rs`) is drivable three ways, but kittest
exposes a helper for only the first:
- **(a) Focus → kb-edit → type → commit** (uses only public kittest helpers): `is_kb_editing =
  mem.has_focus(id)` (:454-457) — a focused DragValue renders AS a TextEdit; `node.type_text("42")`
  focuses then sends IME text; defocus/Enter parses it (:537+). This is the path the spike should try
  FIRST. **Risk:** it is multi-frame and frame-order-fragile (the IME `Event::Text` must survive until
  the DragValue re-renders as a TextEdit on the post-focus frame) — exactly the "run-to-stable" hazard
  the plan flags. This is the precise thing the spike exists to validate.
- **(b) AccessKit `Action::SetValue` with `ActionData::NumericValue`** (:505-509) — egui consumes it
  directly; the cleanest path, BUT **kittest 0.1.0 has no helper to enqueue it** (`node.click()` hard-
  codes `Action::Click`; the event queue is private). The spike would have to reach the raw
  `egui::RawInput`/AccessKit action channel — possible but a custom primitive.
- **(c) `Action::Increment`/`Decrement`** (:494-495, registered :684-687) — same no-helper limitation.

**Bottom line:** P0 is **not** likely to fail wholesale. The ComboBox case (which *looks* like the
biggest risk) is already shipped working code; the spike's real job is to settle the DragValue
primitive (a) vs (b)/(c). If all three prove too fragile, the explicit, sound fallback already lives
in-tree: hand-cell Number via `state_mut()`-injection + a logged §6 gap, mirroring
`archetype_form.rs:238-247`. The plan's outcome contract covers this. **No collapse path.** See m2 for
the one improvement: name these candidate primitives in P0 so the spike author doesn't start cold.

---

## Critical

None.

## Important

None.

---

## Minor / Nit

**m1 — P0 over-states Dropdown as unproven; it's already proven in-tree (de-risk + reuse the
pattern).** The plan lists `Dropdown` as a thing to "prove" with the same weight as the others, and
the risk section calls non-Text driving "unproven." Dropdown driving is **shipped working code**
(`tests/tree_form.rs:655-687`: open-by-role + click-option + assert stored mutation). Recommend: P0's
Dropdown leg is *confirmatory* (cite `tree_form.rs:669-675` as the reference primitive), and P1's
enumerated Dropdown driver should **reuse that exact pattern** rather than re-derive it. One honest
nuance worth keeping in the spike: the proven select-and-mutate is on the *tree kind-picker*; the
`FlagKind::Dropdown` path via `render_with_dispatch` is only proven as far as *popup-open + row-render*
(`repeating_rows.rs:386-397` asserts the option row, never clicks it to mutate the stored value). So
the Dropdown spike still has marginal value (confirm the option-click mutates a `FlagKind::Dropdown`
store), but the *driving primitive* is settled.

**m2 — P0 under-specifies the Number/DragValue driving primitive.** "set a DragValue via the harness"
gives the spike author no starting point, and the obvious mental model (drag) has no kittest helper.
Recommend P0 name the candidate paths explicitly: (a) `type_text`→kb-edit→Enter [try first, only public
helpers], (b) AccessKit `SetValue` [needs a custom enqueue], (c) `Increment`/`Decrement`; and name the
fallback (state-mutation hand-cell à la `archetype_form.rs:238-247`). Without this the spike risks
burning a cycle rediscovering the API surface I traced above. (Cites: `drag_value.rs:454-457,494-495,
505-509,682-687`; `kittest .../node.rs:80,109` — no `drag`/`set_value`.)

**m3 — I2's render-faithfulness needs the FORM-level render path, not the single-flag
`render_with_dispatch`; the plan should say so.** This is the closest thing to a mid-build pivot risk.
`conditional()` lives at `src/form/conditional.rs` and returns `Vec<(flag, Visibility)>`; the
Hidden-skip / Disabled-greyout / PinValue-override / DisableOptions application happens at the
**form-level render loop**, NOT inside `render_with_dispatch` (`widget.rs:81` takes only
`disabled_options`, and applies no `Visibility`). So P1's single-flag injection harness (the proven
`render_with_dispatch(ui, …, flag, state, &[])` pattern, e.g. `repeating_rows.rs:367`) is correct for
I1/I3, but **P2 must drive the full-subcommand form render** (the path `conditional_visibility.rs`
already exercises) so the effects are actually applied. The plan implies this (DisableOptions "drive
the popup open", PinValue read-only only make sense form-level), but never states the enumerator needs
two render modes. Recommend P2 explicitly target the form-level harness; the per-phase R0 on P2 will
otherwise catch it, but naming it now avoids a P2 design churn.

**m4 — P2's PinValue/DisableOptions render asserts should inherit spec m1's "best-effort" hedge,
not be firmed to hard asserts.** Spec R0 r2 m1 flagged that PinValue read-only is "an AccessKit
read-only-property query, unproven in-tree" and DisableOptions per-option greyout has "no per-*flag*
tree node," recommending a best-effort framing with the **argv half as the load-bearing one**. The
plan's P2 hardens both to firm asserts ("assert widget read-only", "assert the named options greyed").
The argv-coercion assert (PinValue emits-replaced via `assemble_argv`) and the row-10/11 stale-emit
checks ARE solid and load-bearing — keep those firm. But the **render-side** read-only / per-option-
disabled AccessKit queries are unproven and could false-RED P2 if egui doesn't surface those
properties. Recommend: gate the two render-side asserts on a tiny P2 feasibility check (or mark
best-effort), exactly as spec m1 advised — the funds/false-CI-load-bearing half stays covered by the
argv asserts regardless.

**m5 — I4's "`#[ignore]`" wording contradicts its own cited reference and would silently zero out CI
coverage.** P4 says "Env-gated/`#[ignore]` … match the existing runner-integration pattern." The
existing pattern is **early-return-skip**, NOT `#[ignore]`
(`tests/runner_integration.rs:260-263`: `if !path.exists() { eprintln!(…); return; }`). This matters:
`cargo test --workspace` (the CI step at `schema-mirror.yml:133`) does **not** run `#[ignore]`d tests,
so an `#[ignore]` I4 would never execute even with the pins installed — defeating I4 entirely. The
schema-mirror job DOES install all four pins (`schema-mirror.yml:49-86`) and runs the workspace tests
with `MNEMONIC_BIN/MD_BIN/MS_BIN/MK_BIN` set (:128-133), so early-return-skip cells run with teeth in
CI and skip cleanly locally. Recommend P4 commit to the early-return-skip pattern and drop the
`#[ignore]` alternative.

**m6 — the shared `tests/ui_harness/mod.rs` will trip `clippy --all-targets -D warnings` on
`dead_code`.** `build.yml:30` runs `cargo clippy --all-targets -D warnings` (covers test targets). A
shared helper module is **compiled into every consuming test binary**, and any binary that uses only
part of the enumerator/seed-table will emit `dead_code`/`unused` → a hard CI failure. The repo has no
existing `tests/<dir>/mod.rs` shared-helper precedent (the only `mod` uses are inline submodules), so
this is a new pattern. Recommend the shared module carry `#![allow(dead_code)]` (and possibly
`unused_imports`). The layout itself is correct — files under `tests/ui_harness/` are not separate test
targets, so P2/P3/P5/P6 reuse it via `mod ui_harness;` (this answers "does the plan silently rebuild
the enumerator?" — no, the layout supports genuine reuse; just gate the warnings).

**m7 — P5's "expects to find bugs" fix-loop is unbounded; add an explicit in-cycle triage bar.** P5
says "file a FOLLOWUP per real bug → fix (each fix its own per-phase-gated change)." With ~47
never-full-flow-tested subcommands, an open-ended in-cycle fix-loop can balloon the cycle without a
defined exit. The mechanism for deferral already exists (FOLLOWUP per bug); the plan just needs the
*bound*: recommend in-cycle fix only **Critical (funds/secret-hygiene) findings**, and **defer all
others to their own FOLLOWUP/cycle** so P5 has a deterministic exit and P6 freezes the harness +
absorbs only the regression cells for what was actually fixed. (Honest framing already present; this
just makes the stop-condition explicit.)

---

## Cross-checks that PASSED (no finding)

- **Phase DAG / buildability:** P0→P1→P2→P3→P4→P5→P6→R is correctly ordered; P1 explicitly consumes
  P0's drivability findings ("spike-approved kinds"); enumerator is reusable by P5/P6 via the
  shared-module layout (m6). ✓
- **Carried Minor m1 folded:** P2 scopes per-effect (Hidden/Disabled = AccessKit node state; PinValue
  = read-only+coercion; DisableOptions = per-option greyout via driven-open popup), NOT a blanket
  per-flag node check. (m4 is a residual hedge refinement, not a fold miss.) ✓
- **Seed-table / I1 blindness:** "widget-inject the under-test value; base state seeds ONLY context
  flags" matches spec §5-I1 (IMP-3). The single-flag `render_with_dispatch` harness keeps widget
  identification unambiguous (relevant because closed ComboBoxes have empty AccessKit labels —
  `repeating_rows.rs:358-360`). ✓
- **I3 surfaces complete:** persisted state (post-`redact_for_persistence`, `persistence.rs:77`),
  masked-argv confirm modal (`assemble_argv_with_secret_mask`, `invocation.rs:152`), and `--spec -`
  stdin are all named; FAKE fixtures + coordinates-only failure output respect secret hygiene; the
  deliberate-leak negative test (assertion-bites proof, no real secret) is specified. The `secret:
  bool` flag attribute exists (`schema/mod.rs:76,106`) so the `secret==true` enumeration is sound. ✓
- **I4 env-gate + CI:** the workspace-test job already runs all kittest tests with the four pins
  installed; "fold into the existing job, no new CI infra, no `wgpu`" is correct
  (`schema-mirror.yml:127-133`). (Pattern correction in m5.) ✓
- **GUI hygiene:** no `cargo fmt --check` gate (`build.yml` has clippy+build only) — correct; PR+CI no
  tag — correct; `schema_mirror` gates flag-NAMES and the new tests add none, so they won't trip it;
  src-grepping gates (`dropdown_id_salt`) are untouched by tests/-only additions. (clippy caveat in
  m6.) ✓
- **Completeness:** every phase is test-authoring (tests-first inherent); post-impl whole-diff review
  is BEFORE merge; all 4 invariant classes assigned; counts (17 conditional, 61 subs) re-verified
  against source. ✓

---

## Bottom line

**GREEN (0C/0I), 7 Minor.** The plan is well-architected and faithful to the R0-GREEN spec. The single
most important thing the prompt asked me to surface — *will P0 fail and collapse the plan?* — is
answered **no**: I verified against the actual egui_kittest/egui sources that Dropdown driving is
already proven in-tree, Boolean/Path are low-risk, and only Number/DragValue is a genuine unknown that
(i) has a plausible public-API path, (ii) has a sound in-tree state-mutation fallback, and (iii) is
exactly what the spike is designed to settle. Fold the 7 Minors (m2/m3/m5/m6 are the high-value ones:
name the DragValue primitives, flag the form-level render path for I2, fix the `#[ignore]`→early-return
wording, and `#![allow(dead_code)]` the shared module) and re-dispatch a convergence check per the
reviewer-loop discipline. The plan is cleared to implement starting with the P0 spike.
