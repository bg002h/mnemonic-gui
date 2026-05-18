//! v0.7.1 SPEC §6.6 row 8 — GUI-internal slot-index contiguity check.
//!
//! Pre-validates that slot indices are contiguous starting at @0 (no gaps).
//! Matches the CLI mode-violation ladder row 8 stderr:
//!   `error: slot indices must be contiguous starting at @0; missing @{i}`
//!
//! Option A pattern (mirrors v0.7.0 row 9 closure): pure GUI-side check;
//! no toolkit wire-format change. The CLI still authoritatively rejects
//! non-contiguous bundles at runtime; the GUI's pre-check is purely UX
//! (shows an inline warning banner before the user hits the CLI error).
//!
//! Closes the v0.6.0-cycle FOLLOWUP `gui-schema-cross-slot-predicate-
//! projection` row-8 share; rows 13/14 are closed as wontfix
//! (CLI-rejection is sufficient).
//!
//! Duplicate-index rows are NOT a contiguity violation (multiple subkeys
//! per slot index are a legitimate slot-row shape — e.g., @0.phrase +
//! @0.passphrase). The gap check operates on UNIQUE indices.

use mnemonic_gui::form::slot_editor::{detect_slot_index_gaps, SlotRow, SlotSubkey};

fn row_at(index: u8) -> SlotRow {
    SlotRow {
        index,
        subkey: SlotSubkey::Phrase,
        value: "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about".into(),
    }
}

#[test]
fn empty_slot_set_has_no_gaps() {
    // No slots → no constraint to violate → no gaps reported.
    let rows: Vec<SlotRow> = vec![];
    assert_eq!(
        detect_slot_index_gaps(&rows),
        Vec::<u8>::new(),
        "empty slot set must have no gap warnings"
    );
}

#[test]
fn single_slot_at_zero_is_contiguous() {
    let rows = vec![row_at(0)];
    assert_eq!(
        detect_slot_index_gaps(&rows),
        Vec::<u8>::new(),
        "single slot @0 is trivially contiguous"
    );
}

#[test]
fn contiguous_slots_have_no_gaps() {
    let rows = vec![row_at(0), row_at(1), row_at(2)];
    assert_eq!(
        detect_slot_index_gaps(&rows),
        Vec::<u8>::new(),
        "contiguous @0/@1/@2 must have no gap warnings"
    );
}

#[test]
fn missing_zero_index_reports_gap_at_zero() {
    // @1 + @2 supplied but @0 absent → gap at @0.
    let rows = vec![row_at(1), row_at(2)];
    assert_eq!(
        detect_slot_index_gaps(&rows),
        vec![0],
        "indices starting at @1 must report @0 as missing"
    );
}

#[test]
fn single_slot_at_nonzero_reports_all_lower_indices_as_missing() {
    // Single slot @3 → @0, @1, @2 all missing.
    let rows = vec![row_at(3)];
    assert_eq!(
        detect_slot_index_gaps(&rows),
        vec![0, 1, 2],
        "a single slot at @3 must report @0/@1/@2 as missing"
    );
}

#[test]
fn middle_gap_reports_missing_intermediate_index() {
    // @0 + @2 supplied; @1 missing.
    let rows = vec![row_at(0), row_at(2)];
    assert_eq!(
        detect_slot_index_gaps(&rows),
        vec![1],
        "@0+@2 must report @1 missing per SPEC §6.6 row 8 stderr"
    );
}

#[test]
fn multiple_middle_gaps_reports_all_missing_indices_ascending() {
    // @0 + @4 supplied; @1, @2, @3 all missing.
    let rows = vec![row_at(0), row_at(4)];
    assert_eq!(
        detect_slot_index_gaps(&rows),
        vec![1, 2, 3],
        "@0+@4 must report @1, @2, @3 missing in ascending order"
    );
}

#[test]
fn duplicate_indices_are_not_a_contiguity_violation() {
    // @0 + @0 (e.g., phrase + path) + @1 — duplicates at @0 are OK
    // (different subkeys per slot are a legitimate row-shape, e.g.,
    // phrase paired with an explicit derivation path); contiguity
    // operates on UNIQUE indices. No gap.
    let mut r0_path = row_at(0);
    r0_path.subkey = SlotSubkey::Path;
    r0_path.value = "m/48'/0'/0'/2'".into();
    let rows = vec![row_at(0), r0_path, row_at(1)];
    assert_eq!(
        detect_slot_index_gaps(&rows),
        Vec::<u8>::new(),
        "duplicate indices (different subkeys) are NOT a contiguity violation"
    );
}

#[test]
fn unsorted_input_is_handled() {
    // Input order doesn't matter — detector sorts internally.
    let rows = vec![row_at(3), row_at(0), row_at(2)];
    assert_eq!(
        detect_slot_index_gaps(&rows),
        vec![1],
        "unsorted input must still report @1 missing"
    );
}
