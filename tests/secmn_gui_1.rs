//! Secret M/N batch gui-1 (2026-11-25): rows gui-slot-field-paste-warn-uncovered
//! (and its duplicate gui-slot-paste-warn-uncovered) and gui-mask-failopen-release.

use eframe::egui;
use egui_kittest::kittest::Queryable;
use egui_kittest::Harness;

use mnemonic_gui::form::invocation::{render_copy_command_masked, ShellFlavor, SECRET_MASK};
use mnemonic_gui::form::secret_widget::paste_warn_id;
use mnemonic_gui::form::slot_editor::{self, SlotRow, SlotState, SlotSubkey};
use mnemonic_gui::secrets::PASTE_WARN_THRESHOLD;

fn slot_harness(subkey: SlotSubkey) -> Harness<'static, SlotState> {
    let state = SlotState {
        rows: vec![SlotRow { index: 0, subkey, value: String::new() }],
    };
    Harness::new_ui_state(|ui, s: &mut SlotState| slot_editor::render(ui, s, None), state)
}

fn bus_flag(h: &mut Harness<'static, SlotState>) -> Option<bool> {
    h.ctx.data_mut(|d| d.get_temp::<bool>(paste_warn_id()))
}

#[test]
fn gui_slot_field_paste_warn_uncovered_over_threshold_paste_raises_flag() {
    let mut h = slot_harness(SlotSubkey::Xprv);
    h.run();
    h.get_by_role(egui::accesskit::Role::PasswordInput).focus();
    h.run();
    h.input_mut()
        .events
        .push(egui::Event::Paste("x".repeat(PASTE_WARN_THRESHOLD + 4)));
    h.run();
    assert_eq!(
        bus_flag(&mut h),
        Some(true),
        "an over-threshold paste into a secret slot value must raise the paste-warn flag"
    );
}

#[test]
fn gui_slot_field_paste_warn_uncovered_under_threshold_paste_no_flag() {
    let mut h = slot_harness(SlotSubkey::Xprv);
    h.run();
    h.get_by_role(egui::accesskit::Role::PasswordInput).focus();
    h.run();
    h.input_mut()
        .events
        .push(egui::Event::Paste("x".repeat(PASTE_WARN_THRESHOLD - 1)));
    h.run();
    assert_ne!(bus_flag(&mut h), Some(true));
}

#[test]
fn gui_slot_field_paste_warn_uncovered_watch_only_slot_no_flag() {
    // Xpub is public: a long paste into it must not raise the secret warning.
    let mut h = slot_harness(SlotSubkey::Xpub);
    h.run();
    h.get_by_role(egui::accesskit::Role::TextInput).focus();
    h.run();
    h.input_mut()
        .events
        .push(egui::Event::Paste("x".repeat(PASTE_WARN_THRESHOLD + 4)));
    h.run();
    assert_ne!(bus_flag(&mut h), Some(true));
}

#[test]
fn gui_mask_failopen_release_short_mask_masks_tail() {
    let argv: Vec<String> = ["mnemonic", "x", "SECRETVALUE"].iter().map(|s| s.to_string()).collect();
    // The mask is one entry short: the unmasked tail must render masked, not cleartext.
    let mask = vec![false, false];
    for flavor in [ShellFlavor::Posix, ShellFlavor::WindowsCmd] {
        let out = render_copy_command_masked(&argv, &mask, flavor);
        assert!(!out.contains("SECRETVALUE"), "fail-open: {out}");
        assert!(out.contains(SECRET_MASK), "tail must be masked: {out}");
    }
}
