//! v0.38.0 — secret-bearing slot values render MASKED + zeroize on remove.
//!
//! SPEC: `design/SPEC_gui_v0_38_0_slot_secret_mask.md`.
//!
//! - T1 (zeroize-on-remove, pure logic): a secret-bearing row's value is
//!   zeroed by `zeroize_if_secret`; a non-secret row's is untouched.
//! - T2 (render mask, kittest): a secret-bearing slot value registers
//!   `Role::PasswordInput`; a non-secret one does not.
//! - T3 (split-brain pin): the render/zeroize gate (`is_secret_bearing`)
//!   equals the persistence gate (`slot_subkey_is_secret`) for ALL variants.

use eframe::egui;
use egui_kittest::kittest::Queryable;
use egui_kittest::Harness;

use mnemonic_gui::form::slot_editor::{self, SlotRow, SlotState, SlotSubkey};
use mnemonic_gui::secrets;

// ── T1 — zeroize on remove ──────────────────────────────────────────────────

#[test]
fn t1_zeroize_if_secret_empties_secret_value_only() {
    let mut secret = SlotRow {
        index: 0,
        subkey: SlotSubkey::Phrase,
        value: "abandon abandon abandon ... art".to_string(),
    };
    secret.zeroize_if_secret();
    assert!(
        secret.value.is_empty(),
        "a secret-bearing slot value must be zeroed on remove; got {:?}",
        secret.value
    );

    let mut watch_only = SlotRow {
        index: 0,
        subkey: SlotSubkey::Xpub,
        value: "xpub6CUGRUo...".to_string(),
    };
    watch_only.zeroize_if_secret();
    assert_eq!(
        watch_only.value, "xpub6CUGRUo...",
        "a non-secret (watch-only) slot value must be left untouched"
    );
}

#[test]
fn t1b_remove_row_removes_the_row() {
    let mut state = SlotState {
        rows: vec![SlotRow {
            index: 0,
            subkey: SlotSubkey::Ms1,
            value: "ms1secretvalue".to_string(),
        }],
    };
    // remove_row zeroizes the secret value (pinned discriminatingly by T1 on
    // the zeroize_if_secret seam it calls) THEN removes; this pins the
    // removal half + that the wrapper doesn't panic on a secret row.
    slot_editor::remove_row(&mut state, 0);
    assert!(state.rows.is_empty(), "the row must be removed");
}

// ── T2 — masked render ──────────────────────────────────────────────────────

fn slot_harness(rows: Vec<SlotRow>) -> Harness<'static, SlotState> {
    let state = SlotState { rows };
    Harness::new_ui_state(
        |ui, st: &mut SlotState| {
            slot_editor::render(ui, st, None);
        },
        state,
    )
}

#[test]
fn t2_secret_slot_value_renders_as_password_field() {
    let mut harness = slot_harness(vec![SlotRow {
        index: 0,
        subkey: SlotSubkey::Phrase,
        value: String::new(),
    }]);
    harness.run();
    assert!(
        harness
            .query_by_role(egui::accesskit::Role::PasswordInput)
            .is_some(),
        "a secret-bearing slot value must render as a masked (password) field"
    );
}

#[test]
fn t2b_non_secret_slot_value_is_not_masked() {
    let mut harness = slot_harness(vec![SlotRow {
        index: 0,
        subkey: SlotSubkey::Xpub,
        value: String::new(),
    }]);
    harness.run();
    assert!(
        harness
            .query_by_role(egui::accesskit::Role::PasswordInput)
            .is_none(),
        "a non-secret (watch-only) slot value must NOT render as a password field"
    );
}

// ── T3 — split-brain pin (render/zeroize gate == persistence gate) ──────────

#[test]
fn t3_is_secret_bearing_equals_persistence_gate_for_all_subkeys() {
    // Tripwire: ALL is hand-maintained and ungated — a forgotten future
    // variant would silently skip this loop.
    assert_eq!(
        SlotSubkey::ALL.len(),
        10,
        "SlotSubkey::ALL changed — update this test + verify the split-brain pin still covers every variant"
    );
    for sk in SlotSubkey::ALL {
        assert_eq!(
            sk.is_secret_bearing(),
            secrets::slot_subkey_is_secret(*sk),
            "split-brain at {sk:?}: the render/zeroize gate (is_secret_bearing) \
             disagrees with the persistence gate (SECRET_SLOT_SUBKEYS) — a value \
             could render plaintext while being persist-redacted, or vice-versa"
        );
    }
}
