//! P0 FEASIBILITY SPIKE — can `egui_kittest` 0.31 drive each non-Text
//! identity widget kind through REAL interaction (NOT by mutating backing
//! state directly)?
//!
//! Plan: `design/IMPLEMENTATION_PLAN_gui_ui_test_harness.md` Phase P0.
//! Spec:  `design/SPEC_gui_automated_ui_test_harness.md` §6.
//!
//! The §3 identity-mapped FlagKinds are Text / Number / Dropdown / Boolean /
//! Path. `Text` is already proven in-tree (`type_text` into a TextEdit, e.g.
//! `tests/repeating_secret_rows.rs`). This spike proves the FOUR non-Text
//! kinds with a minimal one-widget form each, driven purely through the
//! kittest harness, then asserts the widget's BACKING value actually changed.
//!
//! DRIVABILITY FINDINGS (all GREEN under `cargo test --test spike_widget_drivers`):
//!
//! ```text
//! Kind      Widget       Verdict     Working primitive
//! --------  -----------  ----------  -----------------------------------------
//! Dropdown  ComboBox     DRIVABLE    get_by_role(ComboBox).click() opens the
//!                                    popup; get_by_label(opt).click() stores
//!                                    the selection (confirms tree_form.rs:669).
//! Boolean   Checkbox     DRIVABLE    get_by_role(CheckBox).click() flips the
//!                                    bool (AccessKit Click → FAKE_PRIMARY_
//!                                    CLICKED → response.clicked()).
//! Path      TextEdit     DRIVABLE    type_text() (focus + Event::Text) stores
//!                                    the path string (Path is ~ Text).
//! Number    DragValue    DRIVABLE    AccessKit SetValue action (option (b)) —
//!                                    see the note below.
//! ```
//!
//! Number / DragValue detail: option (a) — the keyboard-edit path — does NOT
//! work headlessly. Neither `.click()` (which DOES fire `response.clicked()`)
//! nor `.focus()` flips the DragValue into its `is_kb_editing` TextInput mode
//! under the assertion-only AccessKit harness, so there is never a TextInput to
//! `type_text` into. Option (b) — push an `egui::Event::AccessKitActionRequest`
//! with `Action::SetValue` + `ActionData::NumericValue(v)` targeted at the
//! DragValue's node id, through `harness.input_mut().events`, then run-to-stable
//! — DOES work: the DragValue's own code reads the request and writes the value
//! (NOT a direct state mutation). This is the action the widget advertises via
//! `builder.add_action(Action::SetValue)` — the real assistive-tech / automation
//! path. The `state_mut()` fallback (a la `archetype_form.rs:238-247`) is ALSO
//! proven viable below, for any kind P1 elects to hand-cell.
//!
//! GOTCHA for P1: kittest 0.1.0's `Node` only exposes the `Focus` and `Click`
//! AccessKit actions (`focus()` / `click()`). `SetValue` / `Increment` /
//! `Decrement` are NOT exposed as Node helpers — drive them by pushing a raw
//! `egui::Event::AccessKitActionRequest` onto `harness.input_mut().events`,
//! using the node id from `node.id()` (kittest `Node` derefs to
//! `accesskit_consumer::Node`). Read the id out BEFORE calling `input_mut()`
//! (the Node borrows the harness; `NodeId` is `Copy`).
//!
//! Determinism: every drive is followed by `harness.run()` — egui_kittest's
//! run-to-stable (steps until no immediate repaint is requested), NOT a fixed
//! frame count (the real flake vector, SPEC §10).

use egui::accesskit::{Action, ActionData, ActionRequest};
use egui_kittest::kittest::Queryable;
use egui_kittest::Harness;

/// Backing values for the one-widget spike forms. Each test renders exactly
/// ONE widget bound to one field and asserts that field after driving.
#[derive(Default)]
struct SpikeState {
    dropdown: String,
    boolean: bool,
    path: String,
    number: i64,
}

// ─── Dropdown (egui ComboBox) — CONFIRMATORY ────────────────────────────────

#[test]
fn dropdown_combobox_select_stores_value() {
    // Reuses the proven `tests/tree_form.rs:669-675` primitive on a bare
    // combo: open the popup by role, click a NON-default option by label,
    // assert the backing value is the selected option.
    let mut harness = Harness::new_ui_state(
        |ui, state: &mut SpikeState| {
            egui::ComboBox::from_label("pick")
                .selected_text(state.dropdown.clone())
                .show_ui(ui, |ui| {
                    for opt in ["alpha", "beta", "gamma"] {
                        ui.selectable_value(&mut state.dropdown, opt.to_string(), opt);
                    }
                });
        },
        SpikeState { dropdown: "alpha".to_string(), ..SpikeState::default() },
    );
    harness.run();

    // Open the popup.
    harness.get_by_role(egui::accesskit::Role::ComboBox).click();
    harness.run();

    // Select a non-default option (combo options render as SelectableLabel ⇒
    // Role::Button, so query by label, per the reference pattern).
    harness.get_by_label("gamma").click();
    harness.run();

    assert_eq!(
        harness.state().dropdown,
        "gamma",
        "ComboBox popup selection must store the chosen option"
    );
}

// ─── Boolean (egui Checkbox) ────────────────────────────────────────────────

#[test]
fn boolean_checkbox_click_toggles_value() {
    let mut harness = Harness::new_ui_state(
        |ui, state: &mut SpikeState| {
            ui.checkbox(&mut state.boolean, "flag");
        },
        SpikeState::default(),
    );
    harness.run();
    assert!(!harness.state().boolean, "checkbox starts unchecked");

    harness.get_by_role(egui::accesskit::Role::CheckBox).click();
    harness.run();

    assert!(
        harness.state().boolean,
        "a kittest click on the checkbox must flip the backing bool"
    );
}

// ─── Path (egui TextEdit + stdio sentinel) ──────────────────────────────────

#[test]
fn path_textedit_type_stores_value() {
    // Path is effectively Text: type a path, assert stored.
    let mut harness = Harness::new_ui_state(
        |ui, state: &mut SpikeState| {
            ui.add(egui::TextEdit::singleline(&mut state.path));
        },
        SpikeState::default(),
    );
    harness.run();

    harness
        .get_by_role(egui::accesskit::Role::TextInput)
        .type_text("/tmp/wallet.json");
    harness.run();
    harness.run(); // settle: buffer write-back lands at frame end

    assert_eq!(
        harness.state().path,
        "/tmp/wallet.json",
        "typed path must reach the backing string"
    );
}

#[test]
fn path_textedit_stdio_sentinel_button_stores_dash() {
    // The Path widget's stdio-sentinel affordance: a button that sets the
    // value to the `-` stdin sentinel. Proven the same way (a real click).
    let mut harness = Harness::new_ui_state(
        |ui, state: &mut SpikeState| {
            ui.add(egui::TextEdit::singleline(&mut state.path));
            if ui.button("stdin (-)").clicked() {
                state.path = "-".to_string();
            }
        },
        SpikeState::default(),
    );
    harness.run();

    harness.get_by_label("stdin (-)").click();
    harness.run();

    assert_eq!(
        harness.state().path,
        "-",
        "the stdio-sentinel button must store the `-` sentinel"
    );
}

// ─── Number (egui DragValue) — THE GENUINE UNKNOWN ──────────────────────────

#[test]
fn number_dragvalue_accesskit_setvalue_drives_value() {
    // Option (b): drive the DragValue through the AccessKit `SetValue` action
    // it advertises. The widget's own code reads the request and writes the
    // value via its get/set closure — this is NOT a direct state mutation.
    let mut harness = Harness::new_ui_state(
        |ui, state: &mut SpikeState| {
            ui.add(egui::DragValue::new(&mut state.number));
        },
        SpikeState { number: 7, ..SpikeState::default() },
    );
    harness.run();
    assert_eq!(harness.state().number, 7, "starts at the seeded value");

    // Read the node id out FIRST (Node borrows the harness; NodeId is Copy),
    // then push the SetValue request through the input event queue.
    let id = harness
        .get_by_role(egui::accesskit::Role::SpinButton)
        .id();
    harness
        .input_mut()
        .events
        .push(egui::Event::AccessKitActionRequest(ActionRequest {
            action: Action::SetValue,
            target: id,
            data: Some(ActionData::NumericValue(4242.0)),
        }));
    harness.run();

    assert_eq!(
        harness.state().number,
        4242,
        "AccessKit SetValue must drive the DragValue's backing i64"
    );
}

#[test]
fn number_dragvalue_state_mut_fallback_is_viable() {
    // The documented fallback for any kind P1 elects to hand-cell rather than
    // widget-drive: substitute the backing value via `state_mut()` (à la
    // `archetype_form.rs:238-247`'s `select_archetype`). This proves a
    // hand-cell remains viable even if a widget were undrivable — it does NOT
    // collapse the plan, it narrows enumerated I1 by one kind. Number is in
    // fact DRIVABLE (test above), so this is belt-and-suspenders coverage of
    // the fallback mechanism itself.
    let mut harness = Harness::new_ui_state(
        |ui, state: &mut SpikeState| {
            ui.add(egui::DragValue::new(&mut state.number));
        },
        SpikeState::default(),
    );
    harness.run();

    harness.state_mut().number = 9999;
    harness.run();

    assert_eq!(
        harness.state().number, 9999,
        "state_mut() substitution drives the backing value (fallback path)"
    );
    // And the rendered widget reflects it (AccessKit numeric_value), so a
    // hand-cell that injects via state_mut still exercises the real render.
    let node_value = harness
        .get_by_role(egui::accesskit::Role::SpinButton)
        .numeric_value();
    assert_eq!(
        node_value,
        Some(9999.0),
        "the DragValue renders the substituted value"
    );
}
