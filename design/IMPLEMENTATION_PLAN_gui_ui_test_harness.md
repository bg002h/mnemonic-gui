# IMPLEMENTATION PLAN — mnemonic-gui automated UI-functionality harness

**Spec:** `design/SPEC_gui_automated_ui_test_harness.md` (R0-GREEN round-2). **Status:** draft → plan-R0.
**Carried Minor (spec R0 r2):** scope the I2 renderer-faithfulness AccessKit check per-effect —
Hidden/Disabled are enable/visibility tree nodes; **PinValue** = assert read-only + value coercion (not a
visibility node); **DisableOptions** = assert per-option greyout (only meaningful with the popup driven
open), NOT a per-flag node. Folded into P2 below.
**Ship vehicle:** GUI = **PR + CI** (no tag/crates.io). All work on a feature branch; merge after CI green.

## Phase ordering (each phase: tests-first → impl → per-phase R0 to 0C/0I before next)

### P0 — FEASIBILITY SPIKE (gates I1's reach; do FIRST, smallest)
Prove `egui_kittest` 0.31 can drive each non-Text identity `FlagKind` through REAL interaction:
- `Dropdown` — **already PROVEN in-tree** (`tests/tree_form.rs:669-675` opens a ComboBox by role +
  clicks the option). Confirmatory — reuse that pattern, not exploratory (m1).
- `Boolean` — trivial `Action::Click` toggle (proven primitive); `Path` — `type_text` + button (proven).
- `Number`/`DragValue` — **the ONE genuine unknown** (m2): kittest exposes no `drag()`/`set_value()`.
  Try in order: (a) focus → keyboard-edit-mode → `type_text` digits → Enter (public helpers only;
  frame-fragile → rely on run-to-stable); (b) AccessKit `SetValue` if enqueuable; (c) Increment/Decrement
  actions. **Fallback if all fail:** hand-cell via `state_mut()` (à la `archetype_form.rs:238-247`) — this
  narrows enumerated I1 by ONE kind, does NOT collapse the plan.
**Deliverable:** a tiny `tests/spike_widget_drivers.rs` proving (or disproving) each, using a minimal
one-flag form per kind. **Outcome contract:** for each kind → DRIVABLE (enumerated I1 in P1) or
NOT-DRIVABLE (P1 hand-cells that kind + a logged §6 gap). **Per-phase R0 reviews the spike findings +
ratifies the I1 reach** before P1 builds on it. If a kind is undrivable with no sound hand-cell, its
enumerated coverage is descoped (logged, not silent).
**Gate:** `cargo test --test spike_widget_drivers`; the findings table persisted to the per-phase report.

### P1 — Enumerator + I1 wiring round-trip (spike-approved kinds)
**Files:** new `tests/ui_harness/mod.rs` (the enumerator + a `subcommand → minimal-valid-base-state`
seed table; **`#![allow(dead_code)]`** — a shared test module trips `clippy --all-targets -D warnings`
`dead_code` per consuming binary, m6) + `tests/ui_harness_i1_roundtrip.rs`.
**Tests-first:** for a vertical slice (≥1 subcommand per CLI, mixing the drivable kinds), assert the
identity round-trip: render via kittest, **widget-inject** a distinguishable value into the under-test
flag (base state seeds ONLY context flags), run-to-stable, `assemble_argv`, assert value↔flag binding.
**Impl:** the enumerator iterating `(tab, sub, identity-flag)`; the per-subcommand seed table (hand-seed
minimal valid base; vary only leaf values). Transform kinds (Range/Timestamp/Composite/TaggedOrIndexed)
+ Slot/Tree surfaces = hand-authored cells with explicit expected argv (NOT enumerated identity).
**Anti-tautology:** do NOT re-assert flag names (owned by `schema_mirror`). **Run-to-stable**, not a
fixed frame count (the real flake vector).
**Gate:** the slice green; `cargo test`; clippy; fmt; NO `cargo fmt` churn beyond new files.

### P2 — I2 conditional/state metamorphics (the 17 conditional subcommands)
**Files:** `tests/ui_harness_i2_conditional.rs`.
**Tests-first / Impl** — per the §5 per-effect table:
- **Renderer-applies-the-rule (per-effect, carried-Minor scoping):** drive the **FORM-LEVEL render path**
  (`conditional()` is applied at the form loop, NOT inside the single-flag `render_with_dispatch` — m3;
  the harness must render the whole subcommand form so the rule is actually exercised). Then: Hidden/Disabled
  → assert the AccessKit node's hidden/disabled state == `conditional(state)`; **PinValue** → render assert
  is **best-effort** (AccessKit read-only is unproven) but the **argv value-coercion assert is FIRM** (m4);
  **DisableOptions** → per-option greyout best-effort (drive the popup open), the no-argv-effect assert firm;
  NOT a per-flag node. Catches renderer↔rule desync; does NOT catch a wrong rule (stated).
- **Value-suppression fenced to Hidden|Disabled** → value entered then Hidden/Disabled must not reach
  argv; universalized across the 17. Do NOT assert suppression for PinValue/DisableOptions.
- **Toggle round-trip (no stuck state):** toggle a gating input on→off→on; assert the **visibility-state
  projection** (`conditional(state)`) returns to baseline (NOT value-state — values may be destroyed).
- `conditional()` purity: a cheap unit check, not a headline.
**Gate:** the 17 green; clippy/fmt.

### P3 — I3 classified-secret persistence regression net
**Files:** `tests/ui_harness_i3_secret_nopersist.rs`.
**Tests-first / Impl:** for every `secret==true` flag × subcommand, widget-inject a FAKE fixture, drive
the persistence walk **including `redact_for_persistence`** (tree `key`/`keys` persist-then-redact —
assert POST-redaction, not just `serde(skip)`), assert the fixture ABSENT from persisted state AND the
masked-argv confirm-modal/preview AND `--spec -` stdin. **Honest scope:** classified-secret regression
net only — does NOT replace `schema_mirror_secret_drift`/`secret_taxonomy_pin` (unclassified detection).
**Harness hygiene:** FAKE fixtures; on failure emit flag/subcommand coordinates ONLY (never dump the
AccessKit tree / state — undo-ring plaintext); respect `Zeroizing`.
**Gate:** all classified secrets green; a deliberate-leak negative test proves the assertion bites.

### P4 — I4 curated real-CLI functional cells
**Files:** `tests/ui_harness_i4_realcli.rs`.
**Impl:** a SMALL happy-path set per CLI: drive form → Run against the pinned binary
(`MNEMONIC_BIN`/`MD_BIN`/`MS_BIN`/`MK_BIN`, the schema-mirror pin) → assert exit-code + `--json` parses +
key fields. **Env-gated EARLY-RETURN-SKIP** if the pinned binary absent (match `tests/runner_integration.rs`
exactly — NOT `#[ignore]`, which would make I4 never run under `cargo test --workspace` even with pins,
zeroing its CI teeth, m5). CI without binaries still passes; the schema-mirror CI job (which installs the
four pins, `schema-mirror.yml:49-86`) actually exercises them.
**Gate:** cells green with pins present; skip cleanly without.

### P5 — One-time sweep (coverage bug-finder, proptest)
Run the enumerator + I1/I2/I3 as **proptest** (broad leaf variation + toggle sequences) over all 61
subcommands (esp. the ~47 with no full-flow cell). Triage findings → file a FOLLOWUP per real bug → fix
(each fix its own per-phase-gated change). **This phase EXPECTS to find bugs; that's its job.**
**Triage bar (m7 — bounds the fix-loop):** in-cycle, fix ONLY funds- or secret-Critical findings; file
everything else as FOLLOWUPs for a follow-on cycle (the harness + its regression cells ship regardless).
Honest: a coverage sweep, not a known-bug fix. Persist the sweep report.

### P6 — Permanent CI gate (deterministic table-driven)
Convert the proven invariants to deterministic table-driven cells (no proptest randomness in CI →
no flake/shrink); absorb P5 regression cells. Wire into the existing headless `cargo test --workspace`
path (no new CI infra; NO `wgpu` feature). Update `schema-mirror.yml` only if a new job is needed
(prefer folding into the existing workspace-test job).

### R — ship
PR on a feature branch → CI green (the schema_mirror + workspace tests, incl. the new harness) → merge.
Flip the `gui-automated-ui-functionality-harness` FOLLOWUP RESOLVED; file the sweep-found bug FOLLOWUPs.
**Post-impl whole-diff review** (mandatory) BEFORE merge — over the whole harness diff + the P5 fixes.

## Risk / sequencing
- **P0 is the load-bearing risk gate** — if non-Text driving is infeasible, I1's "universal" reach
  narrows to hand-cells; report the spike outcome before P1 scales on it.
- **GUI has NO `cargo fmt` CI gate** (per constellation memory) — do not `cargo fmt` the GUI broadly;
  format only new files, minimally.
- **Pin coupling:** I4 uses the schema-mirror-pinned toolkit binary; keep env-gated so the harness
  doesn't hard-depend on a local install.
- **Determinism:** run-to-stable everywhere; deterministic tables for the gate, proptest only for P5.
- **Hidden cost:** the per-subcommand seed table is the O(flags) bulk (~800–1500 lines); P1 builds the
  pattern on a slice, P5 forces the rest.
