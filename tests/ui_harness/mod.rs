#![allow(dead_code)]
//! Shared P1 UI-test-harness machinery (plan
//! `design/IMPLEMENTATION_PLAN_gui_ui_test_harness.md` Phase P1; spec §3–§5).
//!
//! `#![allow(dead_code)]`: this module is `mod ui_harness;`-included into each
//! consuming integration-test binary. A shared test module trips
//! `clippy --all-targets -D warnings`'s `dead_code` lint per consumer that
//! doesn't exercise every item (plan P1 m6), so the allow is mandatory.
//!
//! ## Contents
//! - [`IdentityKind`] + [`identity_kind`] — the §3 identity-mapped FlagKind
//!   classifier (Text / Number / Dropdown / Boolean / Path).
//! - [`identity_flags`] — the **enumerator** over a real `SubcommandSchema`,
//!   yielding the identity-mapped flags that are drivable by P1's enumerated
//!   I1 round-trip (non-secret + non-repeating; the `--slot` repeating-text
//!   surface, every transform kind, and every secret/repeating flag are
//!   excluded — those are P2/P3/hand-cell territory).
//! - [`base_state`] — the per-subcommand **seed table**: a minimal-valid base
//!   `FormState` seeding ONLY *context* flags (never the value being
//!   round-trip-asserted — §5 I1 injection discipline).
//! - [`render_flag_harness`] / [`render_one_flag`] — render the under-test
//!   flag through the **real** form-level per-flag path (`render_with_dispatch`
//!   wrapped in the same visibility / `disabled_options` / `add_enabled_ui`
//!   logic `src/main.rs` uses), inside an `egui_kittest::Harness`.
//! - [`Injected`] + [`drive`] — the **drive dispatch**: given an identity flag
//!   and a value, drive the RENDERED widget via the matching P0 spike
//!   primitive (`tests/spike_widget_drivers.rs`).
//!
//! ## Why a single-flag render (not the whole form)
//! P1's I1 invariant is the **render→store→argv wiring** of ONE flag, not the
//! conditional *interaction* between flags (that's P2/I2, which the plan notes
//! must render the whole form). `render_with_dispatch` does its own
//! get-or-default → widget → write-back to `state.values` internally, so a
//! single call fully exercises the render→store seam under test. Rendering only
//! the under-test flag also makes the widget uniquely targetable by AccessKit
//! Role (`get_by_role` requires exactly one match) — a faithful whole-form
//! render would surface many same-role widgets with no robust per-flag handle
//! (egui does not associate the flag-name label with its input node). The
//! call we make (`render_with_dispatch`) is byte-identical to the one
//! `src/main.rs`'s form loop makes per Visible flag.
//!
//! ## Number / DragValue through-path (NOT a bypass — verified)
//! The P0 spike drove a DragValue bound *directly* to a backing field. Here the
//! DragValue is reached via `render_with_dispatch`'s real indirection:
//!   `render_with_dispatch` clones the `state.values` entry into a local
//!   `value`, `render_row` binds `egui::DragValue::new(n)` to `n: &mut i64`
//!   inside that local, the AccessKit `SetValue` action writes through `n` into
//!   `value`, and `render_with_dispatch` writes `value` back to `state.values`.
//!   `assemble_argv` then reads `state.values`.
//! So the Number cell exercises render→store→argv end-to-end: `assemble_argv`
//! reads ONLY `FormState`, so a GREEN Number round-trip is empirical proof the
//! injected value flowed through the DragValue's own value-writing code into
//! `FormState` — there is no store-bypass that could make it a tautology.

use egui::accesskit::{Action, ActionData, ActionRequest, Role};
use egui_kittest::kittest::Queryable;
use egui_kittest::Harness;

use mnemonic_gui::app::CliTab;
use mnemonic_gui::form::secret_widget::SecretLineEdit;
use mnemonic_gui::form::widget::render_with_dispatch;
use mnemonic_gui::schema::{
    FlagKind, FlagSchema, FlagValue, FormState, PositionalArgSchema, Schema, SubcommandSchema,
    TaggedOrIndexedValue, TimestampValue, Visibility,
};

// ─── §3 identity-kind classifier + enumerator ──────────────────────────────

/// The five §3 identity-mapped FlagKinds (value-in == value-in-argv).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum IdentityKind {
    Text,
    Number,
    Dropdown,
    Boolean,
    Path,
}

/// Classify a `FlagKind` as one of the five identity kinds, or `None` for the
/// transform kinds (Range / Timestamp / NodeValueComposite / TaggedOrIndexed)
/// which P1 explicitly leaves to hand-authored cells (§3).
pub fn identity_kind(kind: &FlagKind) -> Option<IdentityKind> {
    match kind {
        FlagKind::Text => Some(IdentityKind::Text),
        FlagKind::Number { .. } => Some(IdentityKind::Number),
        FlagKind::Dropdown(_) => Some(IdentityKind::Dropdown),
        FlagKind::Boolean => Some(IdentityKind::Boolean),
        FlagKind::Path { .. } => Some(IdentityKind::Path),
        FlagKind::Range
        | FlagKind::Timestamp
        | FlagKind::NodeValueComposite(_)
        | FlagKind::TaggedOrIndexed(_) => None,
    }
}

/// Enumerate the `(flag, identity-kind)` pairs of `sub` eligible for P1's
/// enumerated I1 round-trip.
///
/// Excluded (intentionally, per the §3 / plan-P1 reach):
/// - **Transform kinds** (`identity_kind` returns `None`) — hand-cells.
/// - **Secret flags** (`secrets::flag_is_secret`) — these route to the
///   `SecretLineEdit` / disabled-stdin-toggle paths in `render_with_dispatch`,
///   NOT `state.values`; their persistence net is P3/I3.  (Mirrors the *render
///   dispatch's* own predicate, which keys on `flag_is_secret`, not just the
///   schema `secret` bool.)
/// - **Repeating flags** — driven via the multi-row `render_repeating` widget
///   (per-row targeting); out of P1's scalar-identity scope.
pub fn identity_flags(
    sub: &'static SubcommandSchema,
) -> impl Iterator<Item = (&'static FlagSchema, IdentityKind)> {
    sub.flags.iter().filter_map(|flag| {
        if flag.repeating || mnemonic_gui::secrets::flag_is_secret(flag) {
            return None;
        }
        identity_kind(&flag.kind).map(|k| (flag, k))
    })
}

/// Resolve a `CliTab` to its pinned `Schema`.
pub fn schema_for(tab: CliTab) -> &'static Schema {
    match tab {
        CliTab::Mnemonic => &mnemonic_gui::schema::mnemonic::SCHEMA,
        CliTab::Md => &mnemonic_gui::schema::md::SCHEMA,
        CliTab::Ms => &mnemonic_gui::schema::ms::SCHEMA,
        CliTab::Mk => &mnemonic_gui::schema::mk::SCHEMA,
    }
}

/// Look up a subcommand by name in a CLI's schema (panics if absent — a test
/// referencing a non-existent subcommand is a coding error, not a flake).
pub fn sub_of(tab: CliTab, name: &str) -> &'static SubcommandSchema {
    schema_for(tab)
        .subcommands
        .iter()
        .find(|s| s.name == name)
        .unwrap_or_else(|| panic!("{}: subcommand `{name}` not in schema", tab.bin_name()))
}

/// Look up a flag by name within a subcommand (panics if absent).
pub fn flag_of(sub: &'static SubcommandSchema, name: &str) -> &'static FlagSchema {
    sub.flags
        .iter()
        .find(|f| f.name == name)
        .unwrap_or_else(|| panic!("subcommand `{}` has no flag `{name}`", sub.name))
}

// ─── seed table (per-subcommand minimal-valid context) ──────────────────────

/// Minimal-valid base `FormState` per subcommand: seeds ONLY the *context*
/// flags a subcommand needs for a coherent form, NEVER the under-test flag
/// (the test strips the under-test flag before injecting through the widget,
/// enforcing the §5 I1 discipline that the round-trip-asserted value is
/// widget-injected, not hand-seeded).
///
/// Hand-seeded for the P1 vertical-slice subcommands; an empty base for the
/// rest (P5's sweep will extend this table to all 61 subcommands — plan §7
/// "the per-subcommand seed table is the O(flags) bulk").
pub fn base_state(tab: CliTab, sub_name: &str) -> FormState {
    let pairs: &[(&str, FlagValue)] = match (tab, sub_name) {
        // `mnemonic addresses` — a watch-only address listing; `--from` (seed
        // source) + `--network` are the context any cell wants present.
        (CliTab::Mnemonic, "addresses") => &[
            ("--from", FlagValue::Text(String::new())),
            ("--network", FlagValue::Dropdown(String::new())),
        ],
        // The other slice subcommands take their primary input as a positional
        // (md1/ms1/mk1 strings) which I1 does not assert; no context flags.
        _ => &[],
    };
    FormState::from_pairs(
        pairs
            .iter()
            .map(|(k, v)| ((*k).to_string(), v.clone())),
    )
}

// ─── form-level render of one flag ──────────────────────────────────────────

/// Render `flag` through the real form-level per-flag path
/// (`render_with_dispatch`, with the same visibility / `disabled_options` /
/// `add_enabled_ui` wrapping `src/main.rs:624-710` applies), inside a fresh
/// `egui_kittest::Harness` seeded with `base`.
pub fn render_flag_harness(
    tab: CliTab,
    sub: &'static SubcommandSchema,
    flag: &'static FlagSchema,
    base: FormState,
) -> Harness<'static, FormState> {
    Harness::new_ui_state(
        move |ui, state: &mut FormState| {
            render_one_flag(ui, tab, sub, flag, state);
        },
        base,
    )
}

/// The body of `src/main.rs`'s per-flag render loop, specialized to one flag:
/// compute the conditional visibility, skip if Hidden, extract this flag's
/// `DisableOptions`, and dispatch through `render_with_dispatch` inside the
/// same `add_enabled_ui(!Disabled)` gate.
pub fn render_one_flag(
    ui: &mut egui::Ui,
    tab: CliTab,
    sub: &'static SubcommandSchema,
    flag: &'static FlagSchema,
    state: &mut FormState,
) {
    let vis: Vec<(&'static str, Visibility)> =
        sub.conditional.map(|f| f(state)).unwrap_or_default();
    let v = vis
        .iter()
        .find(|(k, _)| *k == flag.name)
        .map(|(_, v)| v.clone())
        .unwrap_or(Visibility::Visible);
    if matches!(v, Visibility::Hidden) {
        return;
    }
    let disabled_options: Vec<String> = vis
        .iter()
        .filter(|(k, _)| *k == flag.name)
        .filter_map(|(_, v)| match v {
            Visibility::DisableOptions { values } => Some(values.clone()),
            _ => None,
        })
        .flatten()
        .collect();
    ui.add_enabled_ui(!matches!(v, Visibility::Disabled), |ui| {
        render_with_dispatch(ui, tab, sub.name, flag, state, &disabled_options);
    });
}

// ─── drive dispatch (reuses the P0 spike primitives verbatim) ───────────────

/// A distinguishable value to widget-inject into the under-test flag.
#[derive(Clone, Debug)]
pub enum Injected {
    /// For `Text` and `Path` kinds — typed via the TextEdit.
    Text(&'static str),
    /// For `Number` — driven via the DragValue's AccessKit `SetValue` action.
    /// MUST be within the flag's `[min, resolved_max]` range AND distinct from
    /// the flag's schema default (else `is_at_default` would suppress emission).
    Number(i64),
    /// For `Dropdown` — an option value to select via the ComboBox popup. MUST
    /// differ from `opts[0]` (the rendered default selection) so the popup row
    /// is uniquely labelled AND the injected value is observably distinct.
    Dropdown(&'static str),
    /// For `Boolean` — `true` flips the checkbox on (presence emission).
    Boolean(bool),
}

impl Injected {
    /// The argv VALUE token this injection should produce immediately after the
    /// flag name, or `None` for `Boolean` (presence-only emission, no value
    /// token).
    pub fn expected_token(&self) -> Option<String> {
        match self {
            Injected::Text(s) => Some((*s).to_string()),
            Injected::Number(n) => Some(n.to_string()),
            Injected::Dropdown(s) => Some((*s).to_string()),
            Injected::Boolean(_) => None,
        }
    }

    /// The identity kind this injection drives (cross-checked against the
    /// flag's real kind by [`drive`]).
    pub fn kind(&self) -> &'static [IdentityKind] {
        match self {
            // Text injection drives both Text and Path widgets (Path ~ Text).
            Injected::Text(_) => &[IdentityKind::Text, IdentityKind::Path],
            Injected::Number(_) => &[IdentityKind::Number],
            Injected::Dropdown(_) => &[IdentityKind::Dropdown],
            Injected::Boolean(_) => &[IdentityKind::Boolean],
        }
    }
}

/// Drive the RENDERED widget for `kind` with `injected`, via the P0-proven
/// primitive for that kind. The harness must already have rendered the
/// under-test flag in isolation (so exactly one widget of the target Role
/// exists). Followed internally by run-to-stable (`harness.run()`), never a
/// fixed frame count (SPEC §10).
pub fn drive(harness: &mut Harness<'static, FormState>, kind: IdentityKind, injected: &Injected) {
    assert!(
        injected.kind().contains(&kind),
        "drive: injected value {injected:?} does not match flag kind {kind:?}"
    );
    match (kind, injected) {
        // Text / Path — type into the TextEdit (Path is ~ Text). The
        // under-test flag starts EMPTY (the seed table never seeds it and the
        // caller strips it), so `type_text` lands exactly the injected string.
        (IdentityKind::Text | IdentityKind::Path, Injected::Text(s)) => {
            harness.get_by_role(Role::TextInput).type_text(*s);
            harness.run();
            harness.run(); // settle: buffer write-back lands at frame end
        }
        // Number — first click past the Unset `Set` affordance into a rendered
        // DragValue, then drive the DragValue's AccessKit `SetValue` action
        // (spike option (b); kittest exposes no `set_value()` Node helper).
        (IdentityKind::Number, Injected::Number(n)) => {
            if harness.query_by_label("Set").is_some() {
                harness.get_by_label("Set").click();
                harness.run();
            }
            // Read the node id out BEFORE `input_mut()` (Node borrows the
            // harness; NodeId is Copy).
            let id = harness.get_by_role(Role::SpinButton).id();
            harness
                .input_mut()
                .events
                .push(egui::Event::AccessKitActionRequest(ActionRequest {
                    action: Action::SetValue,
                    target: id,
                    data: Some(ActionData::NumericValue(*n as f64)),
                }));
            harness.run();
        }
        // Dropdown — open the ComboBox popup by Role, click the option by label
        // (options render as SelectableLabel ⇒ Role::Button, queried by label).
        (IdentityKind::Dropdown, Injected::Dropdown(opt)) => {
            harness.get_by_role(Role::ComboBox).click();
            harness.run();
            harness.get_by_label(opt).click();
            harness.run();
        }
        // Boolean — a kittest click flips the checkbox (default-false → true).
        (IdentityKind::Boolean, Injected::Boolean(target)) => {
            if *target {
                harness.get_by_role(Role::CheckBox).click();
                harness.run();
            }
        }
        _ => unreachable!("drive: kind/injected guard above is exhaustive"),
    }
}

// ════════════════════════════════════════════════════════════════════════
// P2 / I2 — whole-form render + conditional-effect machinery
// ════════════════════════════════════════════════════════════════════════
//
// P1 (I1) renders ONE flag in isolation (its render→store→argv wiring). I2
// must render the WHOLE subcommand form so the subcommand's `conditional(state)`
// fn actually runs and GATES the fields (plan P2 m3 — the rule is applied at the
// form loop, NOT inside the single-flag dispatch). These helpers are the I2
// counterpart to P1's `render_one_flag`; P1's path is kept untouched.
//
// ## Per-flag targeting in the whole form (the egui no-label↔input problem)
// egui attaches no association between a flag's name-label and its input
// node, so many same-`Role` widgets (TextInput / ComboBox / SpinButton /
// CheckBox) are NOT individually `get_by_role`-targetable in a full form.
// The robust handle is the flag's **name LABEL**: the real per-flag renderer
// always emits a node carrying the flag-name text —
//   - scalar non-secret: `ui.label(flag.name)` (`widget.rs::render_row`),
//   - secret Text:        `SecretLineEdit::show` → `ui.label(flag.name)`,
//   - secret Boolean:     `Checkbox::new(_, flag.name)` (label IS the name),
//   - repeating:          header `ui.label(flag.name)`.
// `query_by_label(flag.name)` is an EXACT match (proved in the I2 cells:
// `--descriptor` does not collide with `--descriptor-file`, nor `--passphrase`
// with `--passphrase-stdin`), so it is a unique per-flag handle. A flag's
// rendered visibility/enable state then reads off that node:
//   - Hidden   → the name-label node is ABSENT (the form loop `continue`s it),
//   - Disabled → present, `is_disabled() == true` (the `add_enabled_ui(false)`
//                wrap propagates to the inner label — proved in the cells),
//   - Visible / Required / PinValue → present, `is_disabled() == false`.
//
// ## Anti-tautology (stated, mirrors P1's note)
// I2 asserts the RENDERER faithfully applies `conditional()` (catches a
// renderer↔rule DESYNC + that egui→AccessKit actually yields the expected
// absent/disabled states). It does NOT re-assert the rule's *correctness* —
// a wrong rule is owned by `tests/conditional_visibility.rs` /
// `gui_schema_conditional_drift.rs`.

/// True iff the real form loop (`src/main.rs:624-647`) `continue`s `flag_name`
/// out of the GENERIC render path for `state` — i.e. it is render-suppressed by
/// a MODE mutex, not by the conditional's `Hidden`. Mirrors the three name-set
/// `continue`s verbatim: the `--slot` SlotEditor handoff, build-descriptor
/// tree-mode suppression, and build-descriptor archetype-mode suppression.
pub fn is_render_suppressed(
    sub: &'static SubcommandSchema,
    flag_name: &str,
    state: &FormState,
) -> bool {
    if flag_name == "--slot" && sub.allows_slots {
        return true;
    }
    // P1 `gui`-feature split: the mode predicates now live in the non-gated
    // `mode_predicates` module (the gated `tree_form`/`archetype_form` re-export
    // them too, but use the canonical home here so this harness — reused by the
    // P3 faithfulness test — points at the egui-free source of truth).
    let tree_mode = sub.name == "build-descriptor"
        && mnemonic_gui::form::mode_predicates::tree_enabled(state);
    if tree_mode && mnemonic_gui::form::mode_predicates::suppressed_in_tree_mode(flag_name) {
        return true;
    }
    let archetype_mode = sub.name == "build-descriptor"
        && !tree_mode
        && mnemonic_gui::form::mode_predicates::active_archetype(state).is_some();
    if archetype_mode
        && mnemonic_gui::form::mode_predicates::suppressed_in_archetype_mode(flag_name)
    {
        return true;
    }
    false
}

/// The body of `src/main.rs`'s per-flag render loop (`main.rs:601-686`),
/// rendering the WHOLE subcommand form: compute the per-frame conditional
/// visibility, apply the three mode `continue`s + the `Hidden → skip` +
/// `DisableOptions` extraction + the `add_enabled_ui(!Disabled, …)` gate, and
/// dispatch every flag through the real `render_with_dispatch`.
///
/// Deliberately omits the bespoke SUB-SURFACES (`SlotEditor`, the archetype
/// param sub-form, the tree builder, positional widgets): those render
/// ADDITIONAL widgets but do NOT change the per-flag visibility GATE that I2
/// tests, and rendering the archetype sub-form would duplicate flag-name
/// labels (breaking the exact-label handle). The per-flag gate reproduced here
/// is byte-identical to the real loop. Positional-gated conditionals
/// (`md_encode` / `md_address`) read `state.positionals` directly, which the
/// caller seeds — no positional widget render is needed for the gate.
pub fn render_whole_form(
    ui: &mut egui::Ui,
    tab: CliTab,
    sub: &'static SubcommandSchema,
    state: &mut FormState,
) {
    let vis: Vec<(&'static str, Visibility)> =
        sub.conditional.map(|f| f(state)).unwrap_or_default();
    let visibility_of = |name: &str| -> Visibility {
        vis.iter()
            .find(|(k, _)| *k == name)
            .map(|(_, v)| v.clone())
            .unwrap_or(Visibility::Visible)
    };
    for flag in sub.flags {
        if is_render_suppressed(sub, flag.name, state) {
            continue;
        }
        let v = visibility_of(flag.name);
        if matches!(v, Visibility::Hidden) {
            continue;
        }
        let disabled_options: Vec<String> = vis
            .iter()
            .filter(|(k, _)| *k == flag.name)
            .filter_map(|(_, v)| match v {
                Visibility::DisableOptions { values } => Some(values.clone()),
                _ => None,
            })
            .flatten()
            .collect();
        ui.add_enabled_ui(!matches!(v, Visibility::Disabled), |ui| {
            render_with_dispatch(ui, tab, sub.name, flag, state, &disabled_options);
        });
    }
}

/// Construct an `egui_kittest::Harness` rendering the whole `sub` form from
/// `base` (the I2 counterpart to P1's `render_flag_harness`).
pub fn render_whole_form_harness(
    tab: CliTab,
    sub: &'static SubcommandSchema,
    base: FormState,
) -> Harness<'static, FormState> {
    Harness::new_ui_state(
        move |ui, state: &mut FormState| {
            render_whole_form(ui, tab, sub, state);
        },
        base,
    )
}

// ════════════════════════════════════════════════════════════════════════
// Extended render path — positionals + action bar (visual track P1 / I1)
// ════════════════════════════════════════════════════════════════════════
//
// Promoted VERBATIM from `tests/gui_render_faithfulness.rs` (where they began
// as private fns; plan `IMPLEMENTATION_PLAN_gui_visual_screenshot_track.md`
// P1 item I1), so the P3 faithfulness gate and the permanent form-snapshot
// suite (`tests/gui_form_snapshots.rs`) render THE SAME extended whole form
// — flag grid + positionals + `[ Run ]` — through ONE shared code path.
// Behavior-identical move: bodies unchanged, byte-for-byte.

/// Render ONE positional through the production widget path — a byte-mirror of
/// `src/main.rs:832`'s positional loop body (minus the multi-row +/✕ chrome,
/// which is not part of the presence/masking projection). Non-secret → a plain
/// `text_edit_singleline` (`Role::TextInput`); secret → a masked
/// `SecretLineEdit` (`Role::PasswordInput`).
pub fn render_one_positional(
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
pub fn render_positionals(
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
pub fn render_action_bar(ui: &mut egui::Ui) {
    ui.add_enabled(true, egui::Button::new("Run"));
}

/// The extended whole-form harness: the PR #24 flag-grid render PLUS the
/// positionals and the action bar, over the canonical `base` state.
pub fn render_extended_form_harness(
    tab: CliTab,
    sub: &'static SubcommandSchema,
    base: FormState,
) -> Harness<'static, FormState> {
    Harness::new_ui_state(
        move |ui, state: &mut FormState| {
            render_whole_form(ui, tab, sub, state);
            render_positionals(ui, sub, state);
            render_action_bar(ui);
        },
        base,
    )
}

/// The conditional EFFECT for `flag_name` under `state` (first-rule-wins via
/// the find), defaulting to `Visible` when the conditional emits no override.
/// Mirrors `main.rs::visibility_of` / `invocation.rs::visibility_of`.
pub fn effect_of(
    sub: &'static SubcommandSchema,
    state: &FormState,
    flag_name: &str,
) -> Visibility {
    sub.conditional
        .map(|f| f(state))
        .unwrap_or_default()
        .iter()
        .find(|(k, _)| *k == flag_name)
        .map(|(_, v)| v.clone())
        .unwrap_or(Visibility::Visible)
}

/// A coarse, comparable tag for one `Visibility` effect — the projection unit
/// for the I2 toggle-round-trip equivalence (invariant 3). Captures the
/// argv-relevant identity of the effect (incl. PinValue's pinned value /
/// DisableOptions' set) while sidestepping `serde_json::Value`'s ordering.
pub fn vis_tag(v: &Visibility) -> String {
    match v {
        Visibility::Visible => "visible".to_string(),
        Visibility::Hidden => "hidden".to_string(),
        Visibility::Required => "required".to_string(),
        Visibility::Disabled => "disabled".to_string(),
        Visibility::PinValue { value } => format!("pin:{value}"),
        Visibility::DisableOptions { values } => {
            let mut v = values.clone();
            v.sort();
            format!("disable_options:{v:?}")
        }
    }
}

/// The full **visibility-state projection** of `conditional(state)` over EVERY
/// flag of `sub` — the explicit equivalence relation for the I2 toggle
/// round-trip (spec §5 I2 IMP-6: equivalence is over the visibility projection,
/// NOT the stored values, which a toggle may legitimately destroy). Two states
/// are visibility-equivalent iff their projections compare equal.
pub fn visibility_projection(
    sub: &'static SubcommandSchema,
    state: &FormState,
) -> std::collections::BTreeMap<&'static str, String> {
    sub.flags
        .iter()
        .map(|flag| (flag.name, vis_tag(&effect_of(sub, state, flag.name))))
        .collect()
}

/// True iff `flag` renders a flag-name LABEL whose presence/`is_disabled()`
/// faithfully reflects `conditional(state)`'s effect — i.e. it is eligible for
/// the label-based render assertion. EXCLUDES:
///   - flags MODE-suppressed out of the generic loop ([`is_render_suppressed`]),
///   - **secret Boolean** `*-stdin` toggles, which render ALWAYS-disabled by the
///     v0.37.0 grey-out (governed by the renderer, NOT the conditional —
///     `tests/greyout_stdin_toggles_v0_37_0.rs`), so their disabled state would
///     false-RED the conditional-faithfulness assertion.
pub fn eligible_for_label_check(
    sub: &'static SubcommandSchema,
    flag: &'static FlagSchema,
    state: &FormState,
) -> bool {
    if is_render_suppressed(sub, flag.name, state) {
        return false;
    }
    if mnemonic_gui::secrets::flag_is_secret(flag) && matches!(flag.kind, FlagKind::Boolean) {
        return false;
    }
    true
}

/// `query_by_label(flag_name).is_some()` — the flag's name-label node is present.
pub fn label_present(harness: &Harness<'static, FormState>, flag_name: &str) -> bool {
    harness.query_by_label(flag_name).is_some()
}

/// `Some(is_disabled)` when the flag's name-label node is present, else `None`.
pub fn label_disabled(harness: &Harness<'static, FormState>, flag_name: &str) -> Option<bool> {
    harness.query_by_label(flag_name).map(|n| n.is_disabled())
}

/// Construct a "present" `FlagValue` of the right variant for `flag`'s kind,
/// carrying `payload`. Used to seed CONTEXT/GATING flags a conditional reads
/// (`has_value` / `dropdown_value` / `text_value` / `has_positional`); the
/// variant must match the kind so `render_with_dispatch` takes the real branch
/// (not the value-shape-mismatch recovery).
pub fn present_value(flag: &'static FlagSchema, payload: &str) -> FlagValue {
    match flag.kind {
        FlagKind::Text => FlagValue::Text(payload.to_string()),
        FlagKind::Dropdown(_) => FlagValue::Dropdown(payload.to_string()),
        FlagKind::Path { .. } => FlagValue::Path(payload.to_string()),
        FlagKind::Boolean => FlagValue::Boolean(true),
        FlagKind::Number { .. } => FlagValue::Number(payload.parse().unwrap_or(1)),
        FlagKind::NodeValueComposite(opts) => FlagValue::NodeValueComposite {
            node: opts.first().map(|s| (*s).to_string()).unwrap_or_default(),
            value: payload.to_string(),
        },
        FlagKind::Range => FlagValue::Range(0, payload.parse().unwrap_or(1)),
        FlagKind::Timestamp => FlagValue::Timestamp(TimestampValue::Now),
        FlagKind::TaggedOrIndexed(tags) => FlagValue::TaggedOrIndexed(
            TaggedOrIndexedValue::Tag(tags.first().map(|s| (*s).to_string()).unwrap_or_default()),
        ),
    }
}

/// Seed a context/gating flag into `state.values` with the right variant.
pub fn seed(
    state: &mut FormState,
    sub: &'static SubcommandSchema,
    flag_name: &str,
    payload: &str,
) {
    let flag = flag_of(sub, flag_name);
    state
        .values
        .push((flag_name.to_string(), present_value(flag, payload)));
}

/// Seed a `NodeValueComposite` flag with a specific `node` (the conditional
/// reads it via `FormState::composite_node`, e.g. `slip39-split --from`).
pub fn seed_composite(state: &mut FormState, flag_name: &str, node: &str, value: &str) {
    state.values.push((
        flag_name.to_string(),
        FlagValue::NodeValueComposite {
            node: node.to_string(),
            value: value.to_string(),
        },
    ));
}

// ════════════════════════════════════════════════════════════════════════
// P5 — the ALL-61-subcommand seed table (sweep coverage bug-finder)
// ════════════════════════════════════════════════════════════════════════
//
// The plan-P1 `base_state` above hand-seeds only the P1 vertical-slice subs.
// P5 forces the rest: this section is the seed-table extension to **all 61
// subcommands** (mnemonic 32 + md 10 + ms 10 + mk 9). It feeds the I1 wiring
// round-trip sweep in `tests/ui_harness_sweep.rs`.
//
// ## Why CANDIDATE bases (a Vec), not a single base — the coverage maximizer
// The I1 round-trip can only be asserted for an identity flag that the form
// renders **Visible or Required** (not Hidden / Disabled / PinValue, and not
// mode-suppressed). For a `conditional: None` subcommand EVERY flag is always
// Visible, so a single EMPTY base exposes them all. For a CONDITIONAL
// subcommand, different flags are exposed under different gate states (e.g.
// `compare-cost --descriptor` is Disabled once `--miniscript` is populated, and
// vice-versa), so NO single base exposes every flag. `sweep_candidate_bases`
// therefore returns one-or-more bases; the sweep tests each identity flag under
// the FIRST candidate that renders it Visible/Required (a per-flag union over
// the candidates), maximizing covered flags without ever asserting a round-trip
// on a legitimately-suppressed flag.
//
// ## M1 (P1-R0 carry) — render-suppression discipline
// `render_one_flag` (used by the sweep's per-flag isolation render) omits the
// real form loop's three mode `continue`s. To stay byte-faithful under M1 the
// sweep (a) keeps every candidate base **mode-free for `build-descriptor`** (no
// `tree`, no archetype seeded) so no generic flag is mode-suppressed, and (b)
// hard-guards every flag with [`is_render_suppressed`] before driving — a flag
// the real form would `continue` is SKIPPED, never round-tripped. The empty /
// lightly-seeded candidate bases here never activate tree or archetype mode.

/// Build a `FormState` from `(flag_name, payload)` context seeds, using each
/// flag's real kind to pick the right `FlagValue` variant (so the conditional
/// reads it via the production accessor). Composite seeds use `node=value`.
fn base_from(sub: &'static SubcommandSchema, seeds: &[(&str, &str)]) -> FormState {
    let mut st = FormState::default();
    for (name, payload) in seeds {
        seed(&mut st, sub, name, payload);
    }
    st
}

/// One-or-more minimal-VALID candidate base `FormState`s for `sub_name`, the
/// per-flag union the I1 sweep tests against (see the section doc). Covers ALL
/// 61 subcommands: the 44 `conditional: None` subs fall through to the single
/// EMPTY base (every flag Visible there — no context needed); the 17 conditional
/// subs add seeded variants that expose gate-hidden flags. Context seeds are
/// CONTEXT only — the sweep strips the under-test flag before injecting, so a
/// seed never pre-fills the round-trip-asserted value.
pub fn sweep_candidate_bases(tab: CliTab, sub_name: &str) -> Vec<FormState> {
    let sub = sub_of(tab, sub_name);
    // Empty base is always a candidate (the blank-form state).
    let mut bases = vec![FormState::default()];
    // Conditional subs: add seeded variants exposing the gate-hidden flags.
    let extra: Vec<FormState> = match (tab, sub_name) {
        (CliTab::Mnemonic, "bundle") => vec![
            base_from(sub, &[("--descriptor", "wpkh(@0)")]), // canonical → account pin
            base_from(sub, &[("--template", "bip84")]),      // single-sig
        ],
        (CliTab::Mnemonic, "verify-bundle") => {
            vec![base_from(sub, &[("--bundle-json", "{}")])]
        }
        (CliTab::Mnemonic, "convert") => vec![
            {
                // --to address (no template): exposes --script-type + --path.
                let mut st = FormState::default();
                seed_composite(&mut st, "--from", "phrase", "x");
                seed(&mut st, sub, "--to", "address");
                seed(&mut st, sub, "--template", "");
                st
            },
            {
                // --from electrum-phrase: exposes --electrum-version/-language.
                let mut st = FormState::default();
                seed_composite(&mut st, "--from", "electrum-phrase", "x");
                st
            },
            {
                // --to xpub: exposes --xpub-prefix (+ --template via xpub deriv).
                let mut st = FormState::default();
                seed_composite(&mut st, "--from", "phrase", "x");
                seed(&mut st, sub, "--to", "xpub");
                st
            },
            {
                let mut st = FormState::default();
                seed_composite(&mut st, "--from", "phrase", "x");
                seed(&mut st, sub, "--to", "phrase");
                st
            },
        ],
        (CliTab::Mnemonic, "export-wallet") => vec![
            base_from(sub, &[("--descriptor", "wsh(sortedmulti(2,@0,@1))"), ("--template", "")]),
            base_from(sub, &[("--template", "tr-multi-a")]),
            base_from(sub, &[("--template", "bip84")]),
        ],
        (CliTab::Mnemonic, "restore") => {
            vec![base_from(sub, &[("--md1", "md1xyz")])]
        }
        (CliTab::Mnemonic, "derive-child") => {
            vec![base_from(sub, &[("--application", "dice")])]
        }
        (CliTab::Mnemonic, "slip39-split") => {
            let mut st = FormState::default();
            seed_composite(&mut st, "--from", "phrase", "abandon abandon");
            vec![st]
        }
        (CliTab::Mnemonic, "slip39-combine") => {
            vec![base_from(sub, &[("--to", "phrase")])]
        }
        (CliTab::Mnemonic, "compare-cost") => vec![
            base_from(sub, &[("--miniscript", "pk(A)")]),
            base_from(sub, &[("--descriptor", "wsh(pk(A))")]),
        ],
        (CliTab::Md, "encode") => vec![
            base_from(sub, &[("--from-policy", "and(pk(A),pk(B))")]),
            base_from(sub, &[("--from-policy", "and(pk(A),pk(B))"), ("--context", "segwitv0")]),
        ],
        (CliTab::Md, "compile") => {
            vec![base_from(sub, &[("--context", "segwitv0")])]
        }
        (CliTab::Ms, "encode") => {
            vec![base_from(sub, &[("--hex", "00ff")])]
        }
        (CliTab::Mk, "encode") => vec![
            base_from(sub, &[("--origin-fingerprint", "deadbeef")]),
            base_from(sub, &[("--privacy-preserving", "true")]),
        ],
        // build-descriptor: empty base only (generic mode; mode-free per M1).
        // verify-bundle / repair / inspect / md address: empty base suffices
        // (their conditional only marks Required / cross-disables on populated
        // siblings — the empty blank-form exposes every flag Visible/Required).
        _ => vec![],
    };
    bases.extend(extra);
    bases
}
