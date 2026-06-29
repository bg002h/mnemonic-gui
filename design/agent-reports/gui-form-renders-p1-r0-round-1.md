# GUI-form-renders — Leg-1 P1 R0 review — round 1

**Scope:** Leg-1 P1 (egui-free extraction refactor + default-on `gui` feature + `render_fixture`)
of the GUI-form-renders cycle. Branch `feat/gui-render-form-emit` @ `4718ac8`
(master untouched @ `01520a5`). Plan:
`mnemonic-toolkit/docs/manual-gui/design/IMPLEMENTATION_PLAN_generated_gui_form_renders.md`
(Leg 1 P1); SPEC §3.

**Reviewer:** opus architect, adversarial, verified against source + live builds/tests.

---

## VERDICT: GREEN — 0 Critical / 0 Important

This is a genuinely behavior-preserving refactor. The load-bearing `--no-default-features`
gate is real and verified independently; the test suite is byte-for-byte count-identical to
master; secret-hygiene is fully preserved with no new persist/serialize/escape path. No
rubber-stamp: I re-ran every gate myself. GREEN, plainly.

---

## Load-bearing-gate re-verification (run independently)

1. `cargo build -p mnemonic-gui --no-default-features` → **`Finished dev profile`** (compiles). ✓
2. `cargo tree -p mnemonic-gui --no-default-features --edges normal | grep -iE 'eframe|egui|wgpu|winit'`
   → **EMPTY** (grep exit 1; 243-line tree, zero graphics-stack edges). ✓
3. `cargo build -p mnemonic-gui` (default, `gui` on) → **`Finished`** (GUI app + `mnemonic-gui` bin build). ✓

The headless build is egui-free **structurally** (no eframe/egui/wgpu/winit in the normal
dependency closure), not merely "happens to compile." Gate is sound and load-bearing.

---

## Behavior-preserving verification

- `cargo test -p mnemonic-gui --jobs 2`: **607 passed / 0 failed / 4 ignored** across 68 test
  binaries. Master (via throwaway worktree @ `01520a5`): **607 / 0 / 4**. **Identical** — the
  plan's claim holds exactly.
- Spot-confirmed green directly: `schema_mirror` (21/0), `persist_redaction_v0_34_0` (9/0),
  `secret_taxonomy_pin` (9/0), `repeating_secret_rows` (8/0), `ui_harness_i3_secret_nopersist`
  (7/0); PR-#24 `ui_harness_i1..i4` + `ui_harness_sweep` all green.
- The refactor is genuinely relocation-only: the moved bodies (`SlotSubkey`/`SlotRow`/
  `SlotState`/`remove_row`/`detect_slot_index_gaps`, `SecretLineEdit` struct+methods, the two
  `default_flag_value_for*`, the four mode-predicates) are **line-for-line identical** to the
  deleted originals (diff is a pure cut→new-file + `pub use` re-export). No logic moved into or
  out of a function body; no control-flow change. Confirmed by reading the full diff.

---

## Re-export pattern soundness

- Non-gated consumers reach the moved types via the **egui-free canonical path**, NOT a gated
  re-export:
  - `schema/mod.rs:293,322,353,363` → `crate::form::slot_model::SlotState` /
    `crate::form::secret_model::SecretLineEdit`.
  - `persistence.rs:30` → `crate::form::slot_model::SlotState`.
  - `secrets.rs:137` → `crate::form::slot_model::SlotSubkey`.
  - `tests/ui_harness/mod.rs:387-397` → `mnemonic_gui::form::mode_predicates::{tree_enabled,
    suppressed_in_tree_mode, active_archetype, suppressed_in_archetype_mode}`.
  The clean `--no-default-features` compile is dispositive: had any non-gated consumer reached
  a type through a gated re-export, that build would fail. It does not.
- **No duplicate definition / no `#[cfg]` type-split:** each moved type is defined ONCE in its
  egui-free module; the gated module carries only `pub use crate::form::<model>::{…}`. The old
  definitions were deleted, not `#[cfg]`-duplicated. `main.rs`'s `slot_editor::SlotState`
  (line 16) and `schema`'s `slot_model::SlotState` are the **same** type via re-export →
  assignment-compatible (proven by the green default build/suite).
- Re-exports only render under `gui`; if `gui` is off the gated modules don't exist, but no
  non-gated code imports from them, so nothing breaks. Correct.

---

## SECRET-HYGIENE RULING — PASS (no regression; first-class bar met)

`SecretLineEdit` extraction to `secret_model.rs` preserves every hygiene property:

- **Buffer:** `buf: Zeroizing<Vec<u8>>` preserved (`secret_model.rs:31`); zeroed on drop by
  `Zeroizing::Drop`. Explicit `zeroize()` preserved (`secret_model.rs:78-81`).
- **No new serialization path:** `SecretLineEdit` derives only `Default` (+ manual `Debug`).
  Grep confirms **no** `Serialize`/`Deserialize`/`Clone` derive anywhere on the type (only
  doc-comments referencing the *deliberate absence* of `Clone`). `FormState.secret_widgets`
  retains `#[serde(skip)]` (`schema/mod.rs:320`) — the field still never serializes. A secret
  cannot now be persisted or wire-encoded.
- **Redacting Debug preserved:** `Debug` prints `len` only, never bytes (`secret_model.rs:34-39`).
- **No `Clone` ⇒ no second in-memory copy:** preserved; `FormState`'s dropped `Clone` derive
  remains dropped (`schema/mod.rs:316`, `persistence.rs:42`).
- **App-exit sweep still reaches it:** `secrets::zeroize_form_state` flattens
  `state.secret_widgets.values_mut().flatten()` and calls `widget.zeroize()`
  (`secrets.rs:323-324`) → `SecretLineEdit::zeroize` in `secret_model.rs`. Path intact.
- **egui-coupled surface stayed gated:** `show(&mut egui::Ui, …)` and `paste_warn_id() ->
  egui::Id` remain in the `#[cfg(feature="gui")]` `secret_widget.rs`. The headless model
  carries no egui surface.
- **`buf` visibility `private → pub(crate)`** (`secret_model.rs:31`): the **minimal** widening
  required for the inherent-impl split (the gated `show` in `secret_widget.rs` mutates `buf`
  across a module boundary that used to be intra-module). It is `pub(crate)`, not `pub` — the
  crate remains the security boundary; no widening of *external* exposure, no new serialize/log
  reachability. Acceptable; see Nit-1.

No secret now persists, serializes, clones, or escapes that did not before. First-class bar met.

---

## Extraction completeness

- Moved set == the spec/plan R0-verified list: `Slot{State,Row,Subkey}` + `remove_row` +
  `detect_slot_index_gaps` → `slot_model`; `SecretLineEdit` (struct + `new`/`from_text`/
  `as_string`/`is_empty`/`zeroize`) → `secret_model`; `default_flag_value_for` +
  `default_flag_value_for_flag` → `flag_defaults`; `tree_enabled`/`suppressed_in_tree_mode`/
  `active_archetype`/`suppressed_in_archetype_mode` → `mode_predicates`. `tree_model` (already
  non-gated on master) merely re-homed into the unconditional block — gating unchanged.
- Boundary is clean, not papered: the `--no-default-features` lib+bin clippy is GREEN with no
  `dead_code`/`unused` suppression, and no non-gated→gated edge exists (grep + compile). No
  egui-free logic was left stranded in a gated module that a non-gated consumer needs.

## `render_fixture`

- `fixtures.rs:26` — egui-free, in the non-gated lib, `pub fn render_fixture(tab, sub) ->
  FormState` returning canonical `FormState::default()` (the documented `sweep_candidate_bases`
  first element for all 61 forms). `tab`/`sub` reserved (`let _ = (tab, sub)`) for a future
  per-form base without call-site churn. Sound single shared source for P2 (emit) + P3
  (faithfulness): both consume the SAME function, so they cannot silently diverge. `pub fn` ⇒
  not dead_code under `-D warnings` despite no consumer yet (P2/P3 add them). ✓

## Hygiene

- `cargo clippy -p mnemonic-gui --all-targets -- -D warnings` → **clean** (exit 0).
- `cargo clippy -p mnemonic-gui --no-default-features -- -D warnings` → **clean** (exit 0)
  (lib + non-gated bins — the headless config's intended invocation, matching the task's gate
  commands).
- No broad `cargo fmt`: branch has **476** rustfmt deviation-blocks vs master's **481** (fewer,
  and untouched `conditional.rs` retains its pre-existing deviation → no repo-wide reformat was
  run). GUI has no fmt gate; deviations are pre-existing verbatim-copied style. See Nit-2.
- Diff is refactor-only: 16 files, all in `src/form/*`, `src/{schema,persistence,secrets}`,
  `Cargo.toml`, `tests/ui_harness/mod.rs`. No `src/bin/gui-render` (correct — that is P2).
- Branch left clean (working tree empty, master untouched, throwaway worktree removed).

---

## Critical

None.

## Important

None.

## Minor / Nit

- **Minor-1 (forward-looking, NOT blocking P1): no CI guard yet for the headless-build gate.**
  `build.yml`'s clippy job runs only `cargo clippy --all-targets -- -D warnings` (default
  features); there is no `--no-default-features` build/clippy job in mnemonic-gui's own CI. The
  load-bearing gate is proven NOW, but nothing in this repo's CI prevents a future PR from
  silently re-pulling egui into the headless closure before P5's `verify-examples-gui`
  (manual-gui repo) first exercises it. **Recommend P2** (which lands the non-gated `gui-render`
  bin) add a `cargo build -p mnemonic-gui --no-default-features` + `cargo clippy
  --no-default-features -- -D warnings` CI step to keep the gate from rotting. The plan did not
  scope a CI job into P1, so this is a recommendation, not a P1 defect.
- **Nit-1:** `cargo clippy --all-targets --no-default-features -- -D warnings` FAILS — the
  egui-coupled test targets (`tests/ui_harness/mod.rs` etc.) reference `egui::Ui` directly while
  `egui` is gated off. This is **by design** (the kittest harness inherently needs egui;
  `cargo test --no-default-features` is never run — the suite runs under default), and it
  matches the task's gate, which specifies the headless clippy WITHOUT `--all-targets`. Worth a
  one-line note in the eventual headless CI step so it uses the non-`--all-targets` form.
- **Nit-2:** the new extracted files carry verbatim-copied non-canonical rustfmt style (e.g. the
  multi-line `#[derive(...)]` in `slot_model.rs:20-31`). No GUI fmt gate; copying preserves the
  exact source. Leave as-is (a fmt pass here would be out-of-policy noise).
- **Nit-3 (already documented):** `buf` widened to `pub(crate)` — minimal and within-crate;
  noted in the secret-hygiene ruling. No action.

---

## Bottom line

P1 meets its mandate: the egui-free extraction is structurally clean, the `--no-default-features`
gate is real and independently re-verified, behavior is preserved to the exact test count
(607/0/4 == master), and secret-hygiene carries through untouched. **GREEN, 0C/0I — proceed to P2.**
The only follow-through worth tracking is wiring the headless build/clippy into CI at P2 so the
gate cannot silently rot (Minor-1).
