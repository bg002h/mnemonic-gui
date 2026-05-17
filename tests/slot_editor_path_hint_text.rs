//! v0.8.1 F3 — `slot_editor::render` `path_hint` placeholder. When the user
//! selects `SlotSubkey::Path` for a slot row AND the path-hint string is
//! supplied (computed by main.rs when --descriptor is non-canonical) AND the
//! row value is empty, the text-edit widget shows the hint as placeholder.
//!
//! These tests pin the new `render` signature contract (`path_hint:
//! Option<&str>` 3rd arg) and behavior, NOT the visual rendering directly —
//! kittest doesn't expose hint_text as a queryable label, so we exercise the
//! function shape + behavior on row.value mutation.

use egui_kittest::Harness;
use mnemonic_gui::form::slot_editor::{self, SlotRow, SlotState, SlotSubkey};

fn state_with_one_row(subkey: SlotSubkey, value: &str) -> SlotState {
    let mut state = SlotState::default();
    state.rows.push(SlotRow {
        index: 0,
        subkey,
        value: value.into(),
    });
    state
}

/// Cell 18 — path-subkey + path_hint Some + empty value renders without panic.
/// (Visual hint_text assertion is not done in kittest; we pin the contract.)
#[test]
fn path_subkey_with_hint_renders_without_panic() {
    let initial = state_with_one_row(SlotSubkey::Path, "");
    let mut harness = Harness::new_ui_state(
        |ui, state| {
            slot_editor::render(ui, state, Some("m/48'/0'/0'/2'"));
        },
        initial,
    );
    harness.run();
    // hint_text is purely visual; row.value remains empty.
    assert_eq!(harness.state().rows.len(), 1);
    assert_eq!(harness.state().rows[0].value, "");
    assert_eq!(harness.state().rows[0].subkey, SlotSubkey::Path);
}

/// Cell 19 — non-path subkey + path_hint Some + empty value also renders
/// without panic (hint applies only to path subkey, but render must
/// tolerate the parameter regardless of subkey).
#[test]
fn non_path_subkey_with_hint_renders_without_panic() {
    let initial = state_with_one_row(SlotSubkey::Phrase, "");
    let mut harness = Harness::new_ui_state(
        |ui, state| {
            slot_editor::render(ui, state, Some("m/48'/0'/0'/2'"));
        },
        initial,
    );
    harness.run();
    assert_eq!(harness.state().rows.len(), 1);
    assert_eq!(harness.state().rows[0].subkey, SlotSubkey::Phrase);
}

/// Cell 20 — path_hint None preserves the pre-v0.8.1 render shape (no hint,
/// no panic). Regression guard against the new signature breaking existing
/// call sites that pass None.
#[test]
fn path_hint_none_preserves_legacy_render() {
    let initial = state_with_one_row(SlotSubkey::Path, "");
    let mut harness = Harness::new_ui_state(
        |ui, state| {
            slot_editor::render(ui, state, None);
        },
        initial,
    );
    harness.run();
    assert_eq!(harness.state().rows[0].value, "");
}
