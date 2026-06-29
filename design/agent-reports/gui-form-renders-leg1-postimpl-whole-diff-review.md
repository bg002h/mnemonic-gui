# Leg 1 (GUI-form-renders) — post-implementation whole-diff review

**Scope:** mandatory cross-cutting whole-system review of the COMPLETE Leg-1 diff
`master (01520a5) .. feat/gui-render-form-emit (c98143c)` in `/scratch/code/shibboleth/mnemonic-gui`
(24 files, +2779/-423; 16 src + 3 tests + 8 design reports). Independent + adversarial; over and
above the per-phase R0s (P1, P2, P3×4 — all GREEN). Verification run with `cargo test --jobs 2` +
both clippy gates + the headless build + `cargo check --locked` + `cargo tree`.

---

## VERDICT: GREEN — PR/tag-ready, 0 Critical / 0 Important.

The leg coheres as one correct system, is behavior-preserving for the GUI app, is secret-safe across
the whole diff, and every CI job passes locally. Findings below are Minor / release-ritual reminders
only; none block the PR or the tag. One Minor (versioning/CHANGELOG) is a non-gated pre-tag step that
is easy to miss and must be remembered before cutting `mnemonic-gui-v0.53.0`.

---

## Critical

None.

## Important

None.

## Minor / Nit

1. **[release-ritual, NOT gate-enforced — remember before the tag] Version + CHANGELOG not bumped.**
   `Cargo.toml:3` is still `version = "0.52.0"` (the Word-Card cycle's shipped version); the branch
   makes no `version` change and adds no `CHANGELOG.md` entry. This is a GUI **MINOR** (additive: new
   non-gated `gui-render` `[[bin]]` + default-on `gui` feature; GUI-app behavior-preserving) → the tag
   should be `mnemonic-gui-v0.53.0` with `Cargo.toml` bumped to `0.53.0` and a CHANGELOG entry. **No
   CI gate catches a mismatch:** `build.yml`'s `compute-version` derives the artifact `VERSION` from
   the *tag ref name* (`mnemonic-gui-${REF}`), not from `Cargo.toml`, and there is no `changelog-check`
   job (GUI has no changelog gate, consistent with project memory). So tagging at a stale/mismatched
   version would silently publish artifacts named `…-0.53.0…` whose binary `--version` still reports
   `0.52.0`. Per the GUI ritual (PR+CI-then-tag) the bump conventionally rides with the release/tag
   commit, so its absence from the feature branch is expected — flagged because nothing fails if it is
   forgotten.

2. **[by-design, noted] `gui-render` is built but not packaged in the release matrix.** With default
   features on, `cargo build --release` (the `build` matrix, `build.yml:165-172`) now compiles BOTH
   `mnemonic-gui` and the new non-gated `gui-render` bin. The `package-unix`/`package-windows` steps
   (`build.yml:180-196`) `tar`/`7z` only the `mnemonic-gui` binary **by explicit name**, so the extra
   binary does NOT perturb release assets or the SHA256SUMS. Net effect: a small release-build-time
   increase, no functional change. No action required.

3. **[by-design] `fixtures::render_fixture(tab, sub)` ignores its args (`let _ = (tab, sub);`).** A
   documented API-stability reservation so a future per-form fixture can land without touching call
   sites (`src/form/fixtures.rs:26-29`). Clippy `--all-targets -D warnings` is green, so no
   `unused_variables`. By design.

4. **[robustness note, not a defect] The seeded fixed-point cross-check leans on kittest settling.**
   `project_form` predicts the on-load disabled/presence state via the `seeded_fixture` monotone
   fixed-point simulation (`render_emit.rs:102-158`); the faithfulness gate validates it against the
   REAL egui render, which reaches its fixed point only because `egui_kittest::Harness` settles enough
   frames for the production auto-seed (`widget.rs` None-arm push) to converge. This is empirically
   GREEN across all 61 forms and was the explicit subject of P3 R0 ruling A (4 rounds). Sound as
   built; noted so a future form with deep conditional cascades re-confirms convergence rather than
   assuming it.

---

## Whole-leg coherence — PASS

- **Single structural source confirmed.** `form_elements(sub, state)` (`render_emit.rs:227-297`) is
  the ONE pass; both the documented ASCII (`render_form_from_state`, `:301-345`) and the P3
  tree-observable projection (`project_form`, `:465-518`) are reductions of that single `Vec<FormElement>`.
  There is no divergent second render path — the plan-P3 "ASCII and the projection must come from the
  one core" invariant holds in code.
- **Fixture wiring is coherent, not tautological.** `render_fixture` (`fixtures.rs`) is the ONE shared
  canonical base (`FormState::default()`). The emit seeds defaults on top of it via `seeded_fixture`
  (the GUI's per-frame auto-seed fixed point). The faithfulness harness keeps the base BLANK
  (`render_fixture`, `gui_render_faithfulness.rs:244`) and lets the **production** widget path
  auto-seed inside a settling `Harness` — `render_whole_form` (`tests/ui_harness/mod.rs:418-451`)
  calls the same `widget::render_with_dispatch` that `src/main.rs:677` calls. So side (1)
  schema→production-egui→AccessKit and side (2) schema→simulation→prediction share NO projection code:
  their agreement is genuine evidence, not an identity. (Verified the harness invokes the production
  dispatch, not a re-implementation.)
- **Suppression/visibility gate is mirrored, not forked.** `render_emit::is_render_suppressed`
  (`:523-544`), `render_whole_form`, and `tests/ui_harness::is_render_suppressed` all route through
  the single egui-free `mode_predicates` source of truth (the ui_harness diff repoints from
  `tree_form::`/`archetype_form::` to `mode_predicates::` for exactly this reason).

## Secret-hygiene ruling — PASS (first-class; no leak anywhere in the leg)

- **`SecretLineEdit` extraction preserves every property.** `secret_model.rs`: `buf:
  Zeroizing<Vec<u8>>` (zeroed on drop), redacting `Debug` that prints only `len` (`:34-40`), **no**
  `Serialize`/`Deserialize`, **no** `Clone`. `FormState.secret_widgets` stays `#[serde(skip)]`
  (`schema/mod.rs:319-321`, unchanged). The gated `secret_widget.rs` keeps only the egui `show`
  surface and re-exports the type — pure inherent-impl split.
- **Emit masks at the gate, before the value.** `flag_value_str` (`render_emit.rs:592-632`) returns
  the fixed `MASKED` sentinel for any secret value-bearing flag (`is_secret && !is_secret_bool`)
  BEFORE consulting any default/value. `every_secret_flag_renders_masked_never_cleartext`
  (`gui_render_emit.rs:168-213`) proves this over ALL 61 forms — green.
- **The secret-Composite seed (r2 fold, `1d82155`) is safe.** Verified `seed-xor-combine --share` is
  `secret:true, repeating:true, NodeValueComposite` (`schema/mnemonic.rs:2073-2082`). `seeded_fixture`
  seeds it ONE row (mirroring the widget dispatch's `render_repeating`), but with an EMPTY default
  value (`default_value: None`), and the render masks it regardless. The value never originates from
  real secret state: the emit base is `FormState::default()` with an empty `secret_widgets`, and the
  emit path never persists. No disk/preview/log seam.
- **Faithfulness test leaks nothing.** Base is the blank `render_fixture`; no secret is ever injected;
  divergences accumulate COORDINATES only (tab/sub/flag-or-positional NAME + coarse class —
  `gui_render_faithfulness.rs:249-340`), never the AccessKit tree or form state.
- **Whole-diff print scan clean.** The only added value-bearing `format!`s are non-secret
  `Range(a,b)` (reached only after the `is_secret` gate) and the byte-identical extracted
  `to_slot_argv_masked` `@N.subkey=value` line, which already carries its per-token secret-mask bit.
  No unmasked secret print introduced.
- Regression gates green: `persist_redaction_v0_34_0` (9), `repeating_secret_rows` (8),
  `ui_harness_i3_secret_nopersist` (7), `secret_taxonomy_pin` (9), `schema_mirror` (21).

## CI-readiness verdict — PASS (all jobs verified locally)

- **`clippy` (`--all-targets -- -D warnings`):** green (exit 0).
- **`headless` (new): `cargo build -p mnemonic-gui --no-default-features`** green (exit 0); **`cargo
  clippy --no-default-features -- -D warnings`** green (exit 0). `--all-targets` is correctly OMITTED
  (the egui_kittest test targets reference `egui::Ui` and only compile with `gui` on). Build closure
  is genuinely egui-free: `cargo tree --no-default-features --edges normal` shows NO
  egui/eframe/wgpu/winit (the egui hit under default `cargo tree` is the `egui_kittest` **dev**-dep,
  not in the build closure).
- **`msrv` (`cargo check --locked`):** green (exit 0). `Cargo.lock` is unchanged on the branch and
  in-sync — making `eframe`/`egui` optional + adding `[[bin]]`s changes no resolved versions, so
  `--locked` is satisfied.
- **`build` matrix (`cargo build --release`, default features):** unaffected — packages only
  `mnemonic-gui` by name (see Minor 2).
- **`release` (tag-gated):** structurally unaffected by this leg.
- **`schema_mirror`:** green (21) — this leg adds no toolkit-CLI flag/subcommand and does not touch
  `mnemonic-gui/src/schema/mnemonic.rs`, so the flag-name mirror is undisturbed.
- The `required-features = ["gui"]` on the `mnemonic-gui` bin correctly causes `--no-default-features`
  to *skip* (not fail) the egui bin, while the non-gated `gui-render` bin still builds — verified by
  the clean headless build.

## Behavior-preservation re-verification — PASS

- **GUI app untouched.** `src/main.rs`, `src/app.rs`, `src/runner.rs` have ZERO diff on the branch —
  the form render loop + runner are byte-unchanged. The leg moves code and adds an emit path; it does
  not alter GUI logic.
- **Extractions are pure moves + re-exports.** Per-module deletions ≈ additions (`slot_editor` −229 /
  `slot_model` +229; `widget` default-fns −87 / `flag_defaults` +86; `secret_widget` buffer −86 /
  `secret_model` +82; mode predicates out of `tree_form`/`archetype_form` into `mode_predicates`).
  Each gated module re-exports the moved symbols, so existing paths resolve unchanged. `clippy
  --all-targets -D warnings` green → no unused imports / dead code introduced by the split.
- **Test counts match the claim.** HEAD = **622 passed / 0 failed / 4 ignored** (full suite, exit 0)
  == master 607/0/4 **+15** (14 in `gui_render_emit.rs` + 1 in `gui_render_faithfulness.rs`). No
  pre-existing test regressed; spot-confirmed `schema_mirror`, `secret_taxonomy_pin`,
  `persist_redaction_v0_34_0`, `repeating_secret_rows`, `ui_harness_i1_roundtrip`,
  `ui_harness_i3_secret_nopersist` all green.

## PR-readiness / conventions — PASS

- Commit messages use conventional prefixes (`refactor`/`feat`/`test`/`fix`/`docs`) with correct
  trailers on every commit (`Co-Authored-By: Claude Opus 4.8 (1M context)` + `Claude-Session`).
- Design trail tracked: P1, P2, P3 R0 rounds 1–4 persisted under `design/agent-reports/` (+1192).
- No broad `cargo fmt` churn — the diff is a targeted extraction (GUI has no fmt gate; none was run).
- Working tree clean; branch left clean.

---

### One-line bottom line
GREEN — Leg 1 is PR-ready and tag-ready at 0C/0I. The only pre-tag action is the non-gated
release ritual (bump `Cargo.toml` → `0.53.0` + add a CHANGELOG entry before cutting
`mnemonic-gui-v0.53.0`); everything in the diff itself is correct, coherent, secret-safe,
behavior-preserving, and CI-green.
