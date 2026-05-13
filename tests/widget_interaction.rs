//! Phase A.3 (v0.2 T-1): egui_kittest harness scaffold.
//!
//! Introduces the `egui_kittest::Harness` test harness for driving the
//! GUI's widget renderers in integration tests, complementing the
//! existing pure-logic tests (`argv_assembler*`, `conditional_visibility`,
//! `secrets`, etc.) which exercise the model layer without going through
//! egui rendering.
//!
//! Two cells (SPEC §6; Section A §2.4 Option B — assertion-only, no
//! snapshot `.png` files committed):
//!
//! 1. `cell_1_slot_editor_add_remove_writeback` — drives
//!    `slot_editor::render()` through the harness: add row → remove →
//!    add again; asserts the resulting `SlotState::to_slot_argv()`
//!    matches the expected byte-exact argv emission.
//! 2. `cell_2_conditional_visibility_toggle` — drives the
//!    `mnemonic export-wallet` form; mutates `FormState` to toggle
//!    `--template`; asserts `conditional::export_wallet()` returns the
//!    `FlagVisibility` map specified by SPEC §5 (mutual exclusion +
//!    runtime-pre-check Required).
//!
//! The originally-spec'd cell 3 (`cell_3_paste_warn_modal_trigger`) is
//! split into a separate file `tests/widget_secret.rs` landed in Phase
//! B.1 once `SecretLineEdit` exists (R1 C-1 fold — referencing a B.1
//! type from this A.3 file would block A.3's compile gate).

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;

use mnemonic_gui::form::conditional;
use mnemonic_gui::form::slot_editor::{self, SlotState};
use mnemonic_gui::schema::{FlagValue, FormState, Visibility};

#[test]
fn cell_1_slot_editor_add_remove_writeback() {
    let initial = SlotState::default();
    let mut harness = Harness::new_ui_state(
        |ui, state| {
            slot_editor::render(ui, state);
        },
        initial,
    );

    // Initial state: zero rows; argv emission empty.
    assert!(
        harness.state().rows.is_empty(),
        "fresh SlotState should have no rows"
    );

    // Add a row. The "+ Add slot" label is rendered by slot_editor::render
    // at the bottom of the scroll area (slot_editor.rs:177).
    harness.get_by_label("+ Add slot").click();
    harness.run();
    assert_eq!(
        harness.state().rows.len(),
        1,
        "click on `+ Add slot` should append one row to the SlotState"
    );

    // The new row has default values: index = 0, subkey = SlotSubkey::Xpub
    // (hard-coded in SlotRow::default() at slot_editor.rs:83-90 — note
    // that this is NOT SlotSubkey::ALL[0] which is Phrase; the default
    // impl picks Xpub explicitly), value = "".

    // Remove the first row by clicking the ✕ button. Width-1 emoji
    // label; per the v0.1.2 dropdown-id fix convention, the button has
    // no shared-ID confusion. The label-find returns the first match
    // (there is only one ✕ since there is only one row).
    harness.get_by_label("✕").click();
    harness.run();
    assert!(
        harness.state().rows.is_empty(),
        "click on ✕ should remove the row"
    );

    // Add again. With one row added and value still empty,
    // SlotState::to_slot_argv() should emit no argv pairs (empty-value
    // rows are skipped per SPEC §6.7).
    harness.get_by_label("+ Add slot").click();
    harness.run();
    assert_eq!(harness.state().rows.len(), 1);

    let argv = harness.state().to_slot_argv();
    assert_eq!(
        argv,
        Vec::<String>::new(),
        "to_slot_argv() must emit no pairs when the only row has empty value (SPEC §6.7 empty-value omission)"
    );
}

#[test]
fn cell_2_conditional_visibility_toggle() {
    // Drive a FormState that mirrors the `mnemonic export-wallet` form.
    // The harness renders an unrelated probe widget (the export-wallet
    // form's per-flag widget rendering is exercised separately in v0.1
    // pure-logic tests); the focus here is harness-driven state
    // mutation followed by the conditional::export_wallet() visibility
    // query. The probe widget gives the harness something to render so
    // its frame-step actually runs.
    let initial = FormState::default();
    let mut harness = Harness::new_ui_state(
        |ui, state| {
            // Minimal probe: a button labeled "set-template-bip84" that,
            // when clicked, mutates the FormState to set --template.
            // A real form renderer is unnecessary for the visibility
            // query — the assertion is on the conditional fn output,
            // not on rendered geometry.
            if ui.button("set-template-bip84").clicked() {
                state.values.push((
                    "--template".to_string(),
                    FlagValue::Dropdown("bip84".to_string()),
                ));
            }
            if ui.button("set-descriptor").clicked() {
                state.values.push((
                    "--descriptor".to_string(),
                    FlagValue::Text("wpkh(@0/**)".to_string()),
                ));
            }
            if ui.button("clear-form").clicked() {
                state.values.clear();
            }
        },
        initial,
    );

    // Baseline: no template, no descriptor → conditional marks BOTH as
    // Required (export_wallet runtime pre-check, conditional.rs:120-126).
    let vis = conditional::export_wallet(harness.state());
    assert_eq!(
        vis_for(&vis, "--template"),
        Some(Visibility::Required),
        "with neither template nor descriptor, --template should be Required"
    );
    assert_eq!(
        vis_for(&vis, "--descriptor"),
        Some(Visibility::Required),
        "with neither template nor descriptor, --descriptor should be Required"
    );

    // Toggle template via the harness click.
    harness.get_by_label("set-template-bip84").click();
    harness.run();
    let vis = conditional::export_wallet(harness.state());
    assert_eq!(
        vis_for(&vis, "--descriptor"),
        Some(Visibility::Disabled),
        "with template set, --descriptor should be Disabled (mutual exclusion)"
    );
    assert_ne!(
        vis_for(&vis, "--template"),
        Some(Visibility::Required),
        "with template set, --template should no longer be Required"
    );

    // Clear and re-test the descriptor path.
    harness.get_by_label("clear-form").click();
    harness.run();
    harness.get_by_label("set-descriptor").click();
    harness.run();
    let vis = conditional::export_wallet(harness.state());
    assert_eq!(
        vis_for(&vis, "--template"),
        Some(Visibility::Disabled),
        "with descriptor set, --template should be Disabled (mutual exclusion)"
    );
}

fn vis_for(vis: &mnemonic_gui::schema::FlagVisibility, flag: &str) -> Option<Visibility> {
    vis.iter().find(|(k, _)| *k == flag).map(|(_, v)| *v)
}
