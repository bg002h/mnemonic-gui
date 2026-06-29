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
use mnemonic_gui::form::widget::render_with_dispatch;
use mnemonic_gui::schema::{
    FlagKind, FlagSchema, FlagValue, FormState, Schema, SubcommandSchema, Visibility,
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
