//! cycle-3 H3 surface (iii) — node-aware composite paste-warn.
//!
//! The composite VALUE field (`form/widget.rs` NodeValueComposite arm) was a
//! bare `ui.text_edit_singleline(value)` with NO paste detection — so an
//! over-threshold paste of a `--from minikey=<KEY>` value never raised the
//! paste-warn bus flag. This wires node-aware detection (mirroring
//! `SecretLineEdit::show`), gated on `node_type_is_argv_secret(node)`.
//!
//! Mirrors `tests/paste_warn_wiring_v0_40_0.rs`. cycle-15 Lane G slug-3 now
//! masks the SECRET-class composite value on screen, so a secret node's value
//! field is `Role::PasswordInput` while a non-secret (xpub) node stays
//! `Role::TextInput`; the focus helper handles both. The paste-warn co-fire is
//! the property under test (the `.password` swap must not regress it — M3).
//!
//! - secret node (minikey), over-threshold paste → flag RAISED.
//! - non-secret node (xpub), over-threshold paste → flag NOT raised (negative).

use eframe::egui;
use egui_kittest::kittest::Queryable;
use egui_kittest::Harness;

use mnemonic_gui::app::CliTab;
use mnemonic_gui::form::secret_widget::paste_warn_id;
use mnemonic_gui::form::widget;
use mnemonic_gui::schema::{self, FlagSchema, FlagValue, FormState};
use mnemonic_gui::secrets::PASTE_WARN_THRESHOLD;

/// The `convert --from` FlagSchema (NodeValueComposite over
/// CONVERT_FROM_NODES, which includes `minikey`).
fn from_flag() -> &'static FlagSchema {
    schema::mnemonic::SCHEMA
        .subcommands
        .iter()
        .find(|s| s.name == "convert")
        .expect("convert subcommand")
        .flags
        .iter()
        .find(|f| f.name == "--from")
        .expect("convert --from flag")
}

/// Build a harness rendering a single `--from <node>=<value>` composite via the
/// public `widget::render` entry point. The harness owns the `FlagValue` so the
/// node is fixed to `node` and we drive paste into the value field.
fn composite_harness(node: &'static str) -> Harness<'static, FlagValue> {
    let initial = FlagValue::NodeValueComposite {
        node: node.into(),
        value: String::new(),
    };
    Harness::new_ui_state(
        move |ui, value: &mut FlagValue| {
            let state = FormState::default();
            widget::render(
                ui,
                CliTab::Mnemonic,
                "convert",
                from_flag(),
                value,
                &state,
                &[],
            );
        },
        initial,
    )
}

fn bus_flag(h: &mut Harness<'static, FlagValue>) -> Option<bool> {
    h.ctx.data_mut(|d| d.get_temp::<bool>(paste_warn_id()))
}

/// Focus the composite value field and inject a paste event of `len` chars.
///
/// cycle-15 Lane G slug-3: a SECRET-class node's value field now renders
/// `.password`-masked → its accessibility role is `PasswordInput`, NOT
/// `TextInput`. A non-secret (xpub) node stays a plain `TextInput`. The
/// composite has exactly one editable text field either way, so we focus
/// whichever of the two roles is present (the ComboBox is a button/popup).
fn paste_into_value_field(h: &mut Harness<'static, FlagValue>, len: usize) {
    h.run();
    if let Some(field) = h.query_by_role(egui::accesskit::Role::PasswordInput) {
        field.focus();
    } else {
        h.get_by_role(egui::accesskit::Role::TextInput).focus();
    }
    h.run();
    h.input_mut()
        .events
        .push(egui::Event::Paste("x".repeat(len)));
    h.run();
}

#[test]
fn surface_iii_minikey_over_threshold_paste_raises_warn() {
    let mut h = composite_harness("minikey");
    paste_into_value_field(&mut h, PASTE_WARN_THRESHOLD + 4);
    assert_eq!(
        bus_flag(&mut h),
        Some(true),
        "an over-threshold paste into a --from minikey=… composite value must \
         raise the paste-warn bus flag"
    );
}

#[test]
fn surface_iii_negative_control_xpub_paste_does_not_raise_warn() {
    let mut h = composite_harness("xpub");
    paste_into_value_field(&mut h, PASTE_WARN_THRESHOLD + 4);
    assert_ne!(
        bus_flag(&mut h),
        Some(true),
        "a paste into a watch-only --from xpub=… composite must NOT raise the \
         paste-warn flag (node-aware gate)"
    );
}

#[test]
fn surface_iii_minikey_under_threshold_paste_does_not_raise_warn() {
    let mut h = composite_harness("minikey");
    paste_into_value_field(&mut h, PASTE_WARN_THRESHOLD - 1);
    assert_ne!(
        bus_flag(&mut h),
        Some(true),
        "an under-threshold paste must NOT raise the paste-warn flag"
    );
}
