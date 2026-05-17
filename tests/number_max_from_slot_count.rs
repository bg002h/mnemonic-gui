//! v0.7.0 SPEC §6.6 row 9 — `NumberMax::FromSlotCount` resolve tests.
//!
//! `NumberMax` is the GUI-internal upper-bound enum for `FlagKind::Number`.
//! `Static(i64)` is the compile-time-constant case (most flags); `FromSlotCount`
//! resolves at render time to `state.slot_count().max(1) as i64`. The
//! `--threshold` flag at `src/schema/mnemonic.rs` uses `FromSlotCount` (3
//! sites: lines 228 / 388 / 569 for bundle / verify-bundle / export-wallet
//! respectively).
//!
//! Closes SPEC §6.10.7 row 9 (GUI-internal — no toolkit wire-format change
//! per the v0.7.0 Option A design decision).

use mnemonic_gui::form::slot_editor::SlotRow;
use mnemonic_gui::schema::{FormState, NumberMax};

fn state_with_slot_count(count: usize) -> FormState {
    let mut state = FormState::default();
    while state.slots.rows.len() < count {
        state.slots.rows.push(SlotRow::default());
    }
    state.slots.rows.truncate(count);
    state
}

#[test]
fn flag_kind_number_max_from_slot_count_resolves_to_slot_count() {
    // Happy path: slot_count > 0, resolved max equals slot_count exactly.
    let state = state_with_slot_count(3);
    assert_eq!(
        NumberMax::FromSlotCount.resolve(&state),
        3,
        "FromSlotCount must resolve to state.slot_count() when > 0"
    );
}

#[test]
fn flag_kind_number_max_from_slot_count_clamps_to_one_when_slot_count_zero() {
    // Defensive bound: when slot_count == 0 (no slots configured), the
    // resolved max is 1 (matching `min: 1` declared on the toolkit's
    // clap-derive `--threshold` value_parser!(1..16)). The degenerate
    // range `1..=1` is valid (user can only enter 1, which is also the
    // only valid threshold for a 1-slot config). CLI row 9 catches the
    // residual if argv is emitted from that state.
    let state = state_with_slot_count(0);
    assert_eq!(
        NumberMax::FromSlotCount.resolve(&state),
        1,
        "FromSlotCount must clamp to 1 when slot_count == 0 (degenerate \
         but valid range; matches `--threshold` clap-derive min)"
    );
}

#[test]
fn flag_kind_number_max_static_resolves_to_static_value() {
    // Regression guard: Static(N) must resolve to N regardless of
    // FormState contents. Catches a hypothetical refactor that mishandles
    // the non-FromSlotCount arm.
    let state_empty = FormState::default();
    let state_three = state_with_slot_count(3);
    let state_ten = state_with_slot_count(10);
    for state in [&state_empty, &state_three, &state_ten] {
        assert_eq!(
            NumberMax::Static(42).resolve(state),
            42,
            "Static(N) must resolve to N regardless of slot_count"
        );
        assert_eq!(NumberMax::Static(0).resolve(state), 0);
        assert_eq!(NumberMax::Static(i64::MAX).resolve(state), i64::MAX);
    }
}

#[test]
fn flag_kind_number_max_from_slot_count_tracks_slot_count_increases() {
    // Pin the dynamic behavior: increasing slot_count must immediately
    // increase the resolved max (no caching, no laziness — the resolve is
    // a pure function of state at the moment of call).
    for n in 1..=8usize {
        let state = state_with_slot_count(n);
        assert_eq!(
            NumberMax::FromSlotCount.resolve(&state),
            n as i64,
            "FromSlotCount must track slot_count = {n}"
        );
    }
}
