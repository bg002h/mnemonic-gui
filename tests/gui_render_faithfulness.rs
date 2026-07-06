//! P3 — the egui_kittest FAITHFULNESS gate (SPEC §3 anchor; plan Leg-1/P3).
//!
//! The anti-tautology proof that `gui-render`'s emit DEPICTS what the GUI
//! actually renders. For **all 61 forms**, over the SAME canonical state
//! `render_fixture(tab, sub)`, it compares two INDEPENDENT projections on the
//! tree-observable axes (per flag: presence / disabled / control-class +
//! secret-masking; per positional: presence + secret-masking; the action bar):
//!
//!   (1) the **REAL render** — the form rendered through the production egui
//!       widget path inside an `egui_kittest::Harness`, then read off the
//!       AccessKit widget tree (roles + label nodes). This is the GROUND TRUTH
//!       of what egui draws.
//!   (2) the **emit projection** — `render_emit::project_form`, the SAME
//!       `form_elements` core that produces the documented ASCII render.
//!
//! **Why this is NOT a tautology.** Side (1) never consults the schema/emit to
//! decide a widget's role — it constructs the real egui widgets
//! (`render_with_dispatch`, byte-identical to `src/main.rs`'s form loop; the
//! production positional + Run renders) and reads `accesskit::Role`/label nodes
//! out of the rendered tree. Side (2) reasons purely from `(schema,
//! conditional, render_fixture)`. The two share NO projection code path — one
//! goes schema→egui-widget→AccessKit, the other schema→prediction. Their
//! agreement is therefore evidence the emit faithfully mirrors the GUI, not a
//! definitional identity.
//!
//! **Out of THIS gate (per SPEC §2 / plan m4 — not AccessKit-recoverable;
//! covered by P5 regen-determinism + `schema_mirror`):** path-vs-text, the
//! `(required)` marker, default/placeholder TEXT, and the bespoke sub-surface
//! INTERNALS (SlotEditor / tree / archetype param — only their placeholder
//! LINE is part of the emit, asserted present, never field-level here).
//!
//! **Secret hygiene (first-class).** The fixture is `FormState::default()` —
//! no secret values are ever set or injected. A divergence is reported with
//! form COORDINATES only (tab / sub / flag-or-positional NAME + the coarse
//! observed-vs-emit class) — never the AccessKit tree or the form state, which
//! can carry undo-ring / secret material.
//!
//! ## How the render path was EXTENDED (plan P3 / spec m-R3-1)
//! `render_whole_form` (PR #24) renders only the flag grid — no positionals,
//! no action bar. The missing render surface lives in the SHARED harness
//! module `tests/ui_harness/mod.rs` (promoted there from this file in the
//! visual-track P1 — behavior-identical; the form-snapshot suite renders the
//! same extended path):
//!   - `ui_harness::render_one_positional` — the production positional widget
//!     (a `text_edit_singleline` for non-secret, a masked `SecretLineEdit` for
//!     secret), a byte-mirror of `src/main.rs:832`'s positional loop body.
//!   - `ui_harness::render_action_bar` — the `[ Run ]` button, a mirror of
//!     `src/main.rs:1023` under the canonical (non-tree) fixture where
//!     `run_enabled == true`.
//!
//! The extended whole-form harness (`ui_harness::render_extended_form_harness`)
//! renders flags + positionals + the action bar, so the action bar sits in a
//! faithful full-form context. Per-flag control-class and per-positional
//! masking are read from ISOLATED production renders (the whole form surfaces
//! many same-Role widgets with no per-flag AccessKit handle — the documented
//! PR #24 limitation — so role-class is targetable only one widget at a time).

use egui::accesskit::Role;
use egui_kittest::kittest::Queryable;
use egui_kittest::Harness;

use mnemonic_gui::app::CliTab;
use mnemonic_gui::form::fixtures::render_fixture;
use mnemonic_gui::form::render_emit::{self, ControlClass, Presence};
use mnemonic_gui::form::secret_widget::REVEAL_EYE_GLYPH;
use mnemonic_gui::schema::{FormState, PositionalArgSchema};

mod ui_harness;

/// Single-positional harness (the ISOLATED production render — exactly one
/// input control, so its `Role` is unambiguously targetable).
fn render_one_positional_harness(
    pos: &'static PositionalArgSchema,
    base: FormState,
) -> Harness<'static, FormState> {
    Harness::new_ui_state(
        move |ui, state: &mut FormState| {
            ui_harness::render_one_positional(ui, pos, 0, state);
        },
        base,
    )
}

// ─── tree-observable extraction off the REAL AccessKit tree ──────────────────

/// True iff ≥1 node of `role` exists (`query_all` never panics on multiples,
/// unlike the singular `query_by_role`).
fn has_role(h: &Harness<'static, FormState>, role: Role) -> bool {
    h.query_all_by_role(role).next().is_some()
}

/// True iff ≥1 node carries the exact accessible label `label`.
fn has_label(h: &Harness<'static, FormState>, label: &str) -> bool {
    h.query_all_by_label(label).next().is_some()
}

/// The flag's name-label node is present in the rendered form.
fn flag_label_present(h: &Harness<'static, FormState>, name: &str) -> bool {
    has_label(h, name)
}

/// Classify the input control of an ISOLATED single-flag render off its real
/// AccessKit tree — the GROUND-TRUTH counterpart to `render_emit`'s predicted
/// [`ControlClass`]. Returns `None` if no recognized control rendered (a
/// finding to report, never silently passed). The probe order mirrors the
/// widget dispatch:
///   - any repeating flag emits a `"+ add"` button (header always renders);
///   - a `NodeValueComposite` is a ComboBox + an adjacent text/password field;
///   - a Dropdown is a bare ComboBox; a secret scalar Text a `PasswordInput`;
///     a Boolean a CheckBox; a Text/Path a `TextInput`;
///   - the four `Unset`-default kinds render a `"Set"` button (no input yet).
fn observe_control(h: &Harness<'static, FormState>) -> Option<ControlClass> {
    if has_label(h, "+ add") {
        Some(ControlClass::Repeating)
    } else if has_role(h, Role::ComboBox)
        && (has_role(h, Role::TextInput) || has_role(h, Role::PasswordInput))
    {
        Some(ControlClass::Composite)
    } else if has_role(h, Role::ComboBox) {
        Some(ControlClass::ComboBox)
    } else if has_role(h, Role::PasswordInput) {
        Some(ControlClass::Secret)
    } else if has_role(h, Role::CheckBox) {
        Some(ControlClass::CheckBox)
    } else if has_role(h, Role::TextInput) {
        Some(ControlClass::TextInput)
    } else if has_label(h, "Set") {
        Some(ControlClass::SetButton)
    } else {
        None
    }
}

/// v0.57.0: does the isolated render expose the reveal (👁) eye button? The eye
/// is queried by its EXACT label glyph (not `Role::Button`, which a `?` help
/// icon / `Set` button would also match), so it specifically detects the eye.
fn observe_reveal_eye(h: &Harness<'static, FormState>) -> bool {
    has_label(h, REVEAL_EYE_GLYPH)
}

// ─── the faithfulness gate ───────────────────────────────────────────────────

#[test]
fn gui_render_emit_is_faithful_to_real_form_for_all_61_forms() {
    // Census (mirror `sweep_census`): the gate must cover exactly 61 forms.
    let all = render_emit::all_forms();
    assert_eq!(
        all.len(),
        61,
        "expected exactly 61 GUI subcommand forms (mnemonic 32 + md 10 + ms 10 + mk 9)"
    );

    // Coordinate-only divergence accumulator (NEVER the tree / form state).
    let mut divergences: Vec<String> = Vec::new();
    let mut covered = 0usize;

    for (tab, sub) in all {
        covered += 1;
        let bin = tab.bin_name();
        let proj = render_emit::project_form(tab, sub);

        // ── (1a) presence + disabled + action bar: the REAL extended whole
        //         form ──
        //
        // `egui_kittest::Harness` SETTLES in its constructor (`Harness::from_builder`
        // → `run_ok()` loops `step()` until no immediate repaint is requested),
        // so `wf` is the GUI's post-auto-seed STEADY state — the screen the user
        // actually sees on load, NOT a single construction frame. The emit's
        // `project_form` now mirrors that exactly: it evaluates `conditional`
        // over the `seeded_fixture` render-gated MONOTONE FIXED POINT (P3 R0
        // ruling A), which seeds every rendered non-secret flag's default the
        // way the GUI's per-frame auto-seed does (`widget.rs:220-229`) and
        // re-evaluates `conditional` to convergence. Both sides therefore reflect
        // the same on-load fixed point, so presence AND disabled compare
        // like-for-like — e.g. on bundle/verify-bundle/export-wallet the seeded
        // single-sig `--template -> bip44` greys the multisig-only
        // `--threshold`/`--multisig-path-family` (+ export-wallet `--descriptor`)
        // on BOTH sides.
        let wf = ui_harness::render_extended_form_harness(tab, sub, render_fixture(tab, sub.name));

        // Action bar.
        let real_run = has_label(&wf, "Run");
        if real_run != proj.has_run {
            divergences.push(format!(
                "{bin}/{}: action bar — real Run present={real_run}, emit={}",
                sub.name, proj.has_run
            ));
        }

        // Per-flag PRESENCE + DISABLED (both gated). A flag is `Present` in the
        // rendered grid iff its name-label node exists; `Absent` covers BOTH a
        // mode-suppressed flag (behind a sub-surface) and a conditional-`Hidden`
        // one. When present, its settled `is_disabled()` (off the real label
        // node, `ui_harness::label_disabled`) must equal the emit's
        // `seeded_fixture` disabled prediction.
        //
        // The disabled re-gate is the ANTI-TAUTOLOGY GUARD on the seed simulator
        // itself (R0 ruling A): if `seeded_fixture` ever drifts from the GUI's
        // real per-frame auto-seed, the seeded `conditional` outcome diverges
        // from the settled render and this axis REDs. (Secret `*-stdin` toggles
        // render always-disabled by the renderer, not the conditional — the emit
        // models that via `is_secret_bool`, and the settled label is likewise
        // disabled, so they agree.)
        for fp in &proj.flags {
            let real_present = flag_label_present(&wf, fp.name);
            let emit_present = matches!(fp.presence, Presence::Present);
            if real_present != emit_present {
                divergences.push(format!(
                    "{bin}/{}: flag {} — real present={real_present}, emit present={emit_present}",
                    sub.name, fp.name
                ));
                continue;
            }
            if emit_present {
                if let Some(real_disabled) = ui_harness::label_disabled(&wf, fp.name) {
                    if real_disabled != fp.disabled {
                        divergences.push(format!(
                            "{bin}/{}: flag {} — real disabled={real_disabled}, emit disabled={}",
                            sub.name, fp.name, fp.disabled
                        ));
                    }
                }
            }
        }

        // ── (1b) per-flag control-class: the REAL isolated render ──
        for fp in &proj.flags {
            if !matches!(fp.presence, Presence::Present) {
                continue;
            }
            let flag = sub
                .flags
                .iter()
                .find(|f| f.name == fp.name)
                .expect("projected flag must exist in schema");
            // Construction frame again (the one-shot render of the canonical
            // fixture); a flag's control-class is frame-invariant, so no
            // settle is needed and none is taken (kept consistent with 1a).
            let h = ui_harness::render_flag_harness(
                tab,
                sub,
                flag,
                render_fixture(tab, sub.name),
            );
            match observe_control(&h) {
                Some(real) if real == fp.control => {}
                Some(real) => divergences.push(format!(
                    "{bin}/{}: flag {} — real control={real:?}, emit control={:?}",
                    sub.name, fp.name, fp.control
                )),
                None => divergences.push(format!(
                    "{bin}/{}: flag {} — real control UNCLASSIFIABLE, emit control={:?}",
                    sub.name, fp.name, fp.control
                )),
            }

            // ── v0.57.0: the reveal (👁) eye — modelled on BOTH sides for the
            //    ALWAYS-eye case (scalar secret Text → ControlClass::Secret,
            //    site #1). The emit predicts `flag_has_reveal_eye`; the real
            //    isolated render must expose the adjacent `Role::Button` labelled
            //    `👁` (a scalar secret Text has no `?` help icon, so the eye is
            //    the only Button — `has_label(👁)` is unambiguous). The
            //    value-CONDITIONAL eyes (the composite site #3, whose default
            //    node may or may not be secret) are NOT modelled here — that
            //    would force fixture-value coupling into the gate (reveal-R0
            //    ruling 4); they are covered by the dedicated kittest cell
            //    `secret_reveal_toggle::cell8b`. The non-vacuity negative for the
            //    modelled case lives in a dedicated test below.
            if matches!(fp.control, ControlClass::Secret) {
                let real_eye = observe_reveal_eye(&h);
                let emit_eye = render_emit::flag_has_reveal_eye(flag);
                if real_eye != emit_eye {
                    divergences.push(format!(
                        "{bin}/{}: flag {} — real reveal-eye={real_eye}, emit reveal-eye={emit_eye}",
                        sub.name, fp.name
                    ));
                }
            }
        }

        // ── (1c) positional presence + secret-masking: the REAL isolated
        //         render ──
        for (i, pp) in proj.positionals.iter().enumerate() {
            let pos = &sub.positional_args[i];
            let h = render_one_positional_harness(pos, render_fixture(tab, sub.name));
            let masked = has_role(&h, Role::PasswordInput);
            let plain = has_role(&h, Role::TextInput);
            if !(masked || plain) {
                divergences.push(format!(
                    "{bin}/{}: positional {} — no input control rendered",
                    sub.name, pp.name
                ));
            } else if pp.secret != masked {
                divergences.push(format!(
                    "{bin}/{}: positional {} — real masked={masked}, emit secret={}",
                    sub.name, pp.name, pp.secret
                ));
            }
        }
    }

    assert_eq!(covered, 61, "census: every one of the 61 forms must be checked");
    assert!(
        divergences.is_empty(),
        "GUI-render faithfulness divergences ({} — emit depiction != real render):\n{}",
        divergences.len(),
        divergences.join("\n")
    );
}

/// v0.57.0 — the reveal-eye faithfulness NON-VACUITY negative (spec §8 test #7).
///
/// Proves the eye cross-check above has teeth: a secret scalar Text field's REAL
/// isolated render carries the 👁 button, so an emit projection that OMITTED the
/// eye (predicted `false`) would DIVERGE from the real render → the gate REDs.
/// Plus a discrimination arm: a non-secret flag carries NO eye on either side,
/// so `observe_reveal_eye` / `flag_has_reveal_eye` are not vacuously always-true.
#[test]
fn reveal_eye_faithfulness_is_non_vacuous() {
    let tab = CliTab::Mnemonic;
    let sub = ui_harness::sub_of(tab, "inspect");

    // A scalar secret Text flag (ControlClass::Secret, site #1).
    let ms1 = ui_harness::flag_of(sub, "--ms1");
    let h = ui_harness::render_flag_harness(tab, sub, ms1, render_fixture(tab, sub.name));
    let real_eye = observe_reveal_eye(&h);
    assert!(
        real_eye,
        "a scalar secret Text field's real render MUST carry the reveal (👁) eye"
    );
    assert!(
        render_emit::flag_has_reveal_eye(ms1),
        "emit MUST predict the eye for a ControlClass::Secret flag"
    );
    // NON-VACUITY: an emit projection omitting the eye (false) would RED against
    // the real render (which has it).
    let projection_omitting_eye = false;
    assert_ne!(
        projection_omitting_eye, real_eye,
        "a projection omitting the eye must diverge from the real render (non-vacuous)"
    );

    // DISCRIMINATION: a non-secret flag has NO eye on either side.
    let json = ui_harness::flag_of(sub, "--json");
    let hj = ui_harness::render_flag_harness(tab, sub, json, render_fixture(tab, sub.name));
    assert!(
        !observe_reveal_eye(&hj),
        "a non-secret flag must NOT render the reveal eye"
    );
    assert!(
        !render_emit::flag_has_reveal_eye(json),
        "emit must NOT predict the eye for a non-secret flag"
    );
}
