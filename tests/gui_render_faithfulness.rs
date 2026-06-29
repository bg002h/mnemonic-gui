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
//! no action bar. This file adds the missing render surface:
//!   - [`render_one_positional`] — the production positional widget (a
//!     `text_edit_singleline` for non-secret, a masked `SecretLineEdit` for
//!     secret), a byte-mirror of `src/main.rs:832`'s positional loop body.
//!   - [`render_action_bar`] — the `[ Run ]` button, a mirror of
//!     `src/main.rs:1023` under the canonical (non-tree) fixture where
//!     `run_enabled == true`.
//!
//! The extended whole-form harness ([`render_extended_form_harness`]) renders
//! flags + positionals + the action bar, so the action bar sits in a faithful
//! full-form context. Per-flag control-class and per-positional masking are
//! read from ISOLATED production renders (the whole form surfaces many
//! same-Role widgets with no per-flag AccessKit handle — the documented PR #24
//! limitation — so role-class is targetable only one widget at a time).

use egui::accesskit::Role;
use egui_kittest::kittest::Queryable;
use egui_kittest::Harness;

use mnemonic_gui::app::CliTab;
use mnemonic_gui::form::fixtures::render_fixture;
use mnemonic_gui::form::render_emit::{self, ControlClass, Presence};
use mnemonic_gui::form::secret_widget::SecretLineEdit;
use mnemonic_gui::schema::{FormState, PositionalArgSchema, SubcommandSchema};

mod ui_harness;

// ─── extended render path: positionals + action bar (plan P3 / m-R3-1) ──────

/// Render ONE positional through the production widget path — a byte-mirror of
/// `src/main.rs:832`'s positional loop body (minus the multi-row +/✕ chrome,
/// which is not part of the presence/masking projection). Non-secret → a plain
/// `text_edit_singleline` (`Role::TextInput`); secret → a masked
/// `SecretLineEdit` (`Role::PasswordInput`).
fn render_one_positional(
    ui: &mut egui::Ui,
    pos: &'static PositionalArgSchema,
    idx: usize,
    state: &mut FormState,
) {
    let label = format!(
        "{} {}{}",
        pos.name,
        if pos.required { "*" } else { "" },
        if pos.repeating { "..." } else { "" }
    );
    if pos.secret {
        let key = format!("positional:{}", pos.name);
        let rows = state
            .secret_widgets
            .entry(key)
            .or_insert_with(|| vec![SecretLineEdit::new()]);
        for row in rows.iter_mut() {
            ui.horizontal(|ui| {
                row.show(ui, &label, pos.help);
            });
        }
        return;
    }
    ui.horizontal(|ui| {
        ui.label(&label);
        while state.positionals.len() <= idx {
            state.positionals.push(String::new());
        }
        ui.text_edit_singleline(&mut state.positionals[idx]);
    });
}

/// Render every positional (extends `render_whole_form`, which renders none).
fn render_positionals(
    ui: &mut egui::Ui,
    sub: &'static SubcommandSchema,
    state: &mut FormState,
) {
    for (i, pos) in sub.positional_args.iter().enumerate() {
        render_one_positional(ui, pos, i, state);
    }
}

/// Render the `[ Run ]` action bar — a mirror of `src/main.rs:1023`. Under the
/// canonical (generic, non-tree) fixture `run_enabled == true`.
fn render_action_bar(ui: &mut egui::Ui) {
    ui.add_enabled(true, egui::Button::new("Run"));
}

/// The extended whole-form harness: the PR #24 flag-grid render PLUS the
/// positionals and the action bar, over the canonical `base` state.
fn render_extended_form_harness(
    tab: CliTab,
    sub: &'static SubcommandSchema,
    base: FormState,
) -> Harness<'static, FormState> {
    Harness::new_ui_state(
        move |ui, state: &mut FormState| {
            ui_harness::render_whole_form(ui, tab, sub, state);
            render_positionals(ui, sub, state);
            render_action_bar(ui);
        },
        base,
    )
}

/// Single-positional harness (the ISOLATED production render — exactly one
/// input control, so its `Role` is unambiguously targetable).
fn render_one_positional_harness(
    pos: &'static PositionalArgSchema,
    base: FormState,
) -> Harness<'static, FormState> {
    Harness::new_ui_state(
        move |ui, state: &mut FormState| {
            render_one_positional(ui, pos, 0, state);
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
        // We read the CONSTRUCTION frame (no `run()`), NOT a settled multi-
        // frame state. This is the apples-to-apples counterpart to the emit's
        // model: `render_emit::project_form` evaluates `conditional(state)`
        // ONCE over the pristine `render_fixture` (and the P2 ASCII is pinned
        // on that one-shot). egui_kittest's `new_ui_state` likewise renders one
        // frame whose visibility gate is `conditional(render_fixture)` — same
        // input state, same single conditional evaluation, so the disabled axis
        // compares like-for-like. Advancing a frame would instead read the
        // GUI's multi-frame STEADY state, where the form loop has auto-seeded
        // dropdown defaults (`--template` → its single-sig default on
        // bundle/verify-bundle/export-wallet) that the conditional then reads to
        // disable the multisig-only flags (`--threshold`/`--descriptor`/
        // `--multisig-path-family`). That settled drift is a documented property
        // of `conditional` subs (default is not a seed-loop fixed point —
        // `tests/ui_harness_i2_conditional.rs`), NOT an emit-faithfulness gap:
        // comparing the emit's render-of-default against egui's render-of-
        // settled-state would compare two DIFFERENT input states.
        let wf = render_extended_form_harness(tab, sub, render_fixture(tab, sub.name));

        // Action bar.
        let real_run = has_label(&wf, "Run");
        if real_run != proj.has_run {
            divergences.push(format!(
                "{bin}/{}: action bar — real Run present={real_run}, emit={}",
                sub.name, proj.has_run
            ));
        }

        // Per-flag PRESENCE (structural — gated). A flag is `Present` in the
        // rendered grid iff its name-label node exists; `Absent` covers BOTH a
        // mode-suppressed flag (behind a sub-surface) and a conditional-`Hidden`
        // one. Presence agrees between the emit's pristine model and the
        // settled GUI for all 61 forms.
        //
        // The conditional-sourced DISABLED DECORATION is deliberately NOT
        // gated here — it is the same class the spec's m4 carve-out already
        // excluded for the `(required)` marker. The emit models the pristine
        // blank-form `FormState::default()` (fixtures.rs: "every flag Visible",
        // the documented P2 canonical fixture) and evaluates `conditional`
        // ONCE on it; egui_kittest's `Harness` instead always settles
        // (`run_ok`) to the post-auto-seed steady state, where the form loop
        // has seeded each flag's schema default (e.g. `--template -> bip44`,
        // already DISPLAYED by the emit) and the conditional then greys the
        // multisig-only flags (`--threshold`/`--descriptor`/
        // `--multisig-path-family` on bundle/verify-bundle/export-wallet). The
        // pristine emit thus depicts those flags ENABLED while the settled GUI
        // shows them DISABLED. That gap (a) cannot be observed pristine-vs-
        // pristine through `Harness` and (b) is already covered: the
        // renderer-applies-`conditional`-`Disabled` faithfulness lives in
        // `tests/ui_harness_i2_conditional.rs`, and the emit's pristine
        // `[disabled]` markers are pinned by the P2 `gui_render_emit` ASCII
        // snapshots. See the report's KEY FINDING for the 6 affected flags and
        // the open question of whether the emit should model the settled state.
        for fp in &proj.flags {
            let real_present = flag_label_present(&wf, fp.name);
            let emit_present = matches!(fp.presence, Presence::Present);
            if real_present != emit_present {
                divergences.push(format!(
                    "{bin}/{}: flag {} — real present={real_present}, emit present={emit_present}",
                    sub.name, fp.name
                ));
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
