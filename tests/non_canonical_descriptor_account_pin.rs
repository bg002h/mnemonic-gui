//! v0.8.0 Phase 6 — canonicity-aware `--account → PinValue(0)` override.
//!
//! Documents the intentional Option-A divergence between the toolkit-
//! emitted `gui-schema` rule (pin unconditionally when `--descriptor`
//! present) and the GUI's hand-coded `bundle()` conditional (lift the pin
//! when descriptor is non-canonical so `--account N` flows through to
//! the toolkit per SPEC §4.12.b default-path inference).
//!
//! The drift gate at `gui_schema_conditional_drift.rs` uses a canonical
//! exemplar value (`pkh(@0)`) when synthesizing FormState for the
//! `--descriptor` flag predicate, so the drift gate continues to pass for
//! the canonical case. THIS cell exercises the non-canonical override.
//!
//! Companion documentation: SPEC §6.10.7 mapping table for §6.6 row 12,
//! marked "v0.19.0 Option-A canonicity override."

use mnemonic_gui::form::conditional::bundle;
use mnemonic_gui::schema::{FlagValue, FormState, Visibility};

fn state_with_descriptor(desc: &str) -> FormState {
    let mut s = FormState::default();
    s.values.push(("--descriptor".into(), FlagValue::Text(desc.into())));
    s
}

#[test]
fn canonical_descriptor_pins_account_to_zero() {
    // Canonical descriptor (md-codec's canonical_origin table supplies the
    // per-shape default origin path). User-typed `--account 5` is silently
    // coerced to 0 by the pin per SPEC §6.10.4 emission table.
    let state = state_with_descriptor("pkh(@0)");
    let vis = bundle(&state);
    let pinned = vis.iter().any(|(k, v)| {
        *k == "--account"
            && matches!(v, Visibility::PinValue { value } if value == &serde_json::json!(0))
    });
    assert!(
        pinned,
        "canonical descriptor should pin --account to 0; got vis: {vis:?}"
    );
}

#[test]
fn suffix_form_origin_descriptor_pins_account_to_zero() {
    // L12 (constellation bughunt): the SUFFIX-origin form `@N[fp/path]` is
    // accepted by the toolkit and classified CANONICAL (empirically, the
    // v0.60.0 binary: `gui-schema --classify-descriptor` → `canonical`).
    // The GUI regex anchored the origin bracket BEFORE `@N` only, so it
    // misclassified this NonCanonical → LIFTED the pin → user `--account N`
    // reached the toolkit → toolkit hard-errored `DESCRIPTOR_WITH_NONZERO_
    // ACCOUNT` on a valid input. After the regex fix the GUI agrees
    // (Canonical → pin fires).
    let state = state_with_descriptor("wpkh(@0[deadbeef/84'/0'/0']/<0;1>/*)");
    let vis = bundle(&state);
    let pinned = vis.iter().any(|(k, v)| {
        *k == "--account"
            && matches!(v, Visibility::PinValue { value } if value == &serde_json::json!(0))
    });
    assert!(
        pinned,
        "suffix-origin descriptor should pin --account to 0; got vis: {vis:?}"
    );
}

#[test]
fn prefix_form_origin_still_pins() {
    // Positive control: the PREFIX-origin form `[fp/path]@N` must STILL pin
    // (the reposition keeps the prefix bracket group — no regression).
    let state = state_with_descriptor("wpkh([deadbeef/84'/0'/0']@0/<0;1>/*)");
    let vis = bundle(&state);
    let pinned = vis.iter().any(|(k, v)| {
        *k == "--account"
            && matches!(v, Visibility::PinValue { value } if value == &serde_json::json!(0))
    });
    assert!(
        pinned,
        "prefix-origin descriptor should still pin --account to 0; got vis: {vis:?}"
    );
}

#[test]
fn no_origin_form_still_pins() {
    // Positive control: the no-origin form `@N` must STILL pin (both
    // optional bracket groups skipped — no regression).
    let state = state_with_descriptor("wpkh(@0)");
    let vis = bundle(&state);
    let pinned = vis.iter().any(|(k, v)| {
        *k == "--account"
            && matches!(v, Visibility::PinValue { value } if value == &serde_json::json!(0))
    });
    assert!(
        pinned,
        "no-origin descriptor should still pin --account to 0; got vis: {vis:?}"
    );
}

#[test]
fn non_canonical_descriptor_lifts_account_pin() {
    // Non-canonical descriptor — pin is LIFTED so the user-typed
    // `--account N` flows through to the toolkit, which consumes it for
    // §4.12.b default-path inference (`m/48'/<coin>'/N'/2'`).
    let state = state_with_descriptor(
        "wsh(andor(pkh(@0),after(12000000),or_i(and_v(v:pkh(@1),older(4032)),and_v(v:pkh(@2),older(32768)))))"
    );
    let vis = bundle(&state);
    let has_account_pin = vis
        .iter()
        .any(|(k, v)| *k == "--account" && matches!(v, Visibility::PinValue { .. }));
    assert!(
        !has_account_pin,
        "non-canonical descriptor should NOT pin --account; got vis: {vis:?}"
    );
}

#[test]
fn tr_with_taptree_non_canonical_lifts_pin() {
    // `tr(@N, TapTree)` is non-canonical per md-codec's table; same
    // override applies.
    let state = state_with_descriptor("tr(@0,multi_a(2,@1,@2))");
    let vis = bundle(&state);
    let has_account_pin = vis
        .iter()
        .any(|(k, v)| *k == "--account" && matches!(v, Visibility::PinValue { .. }));
    assert!(
        !has_account_pin,
        "tr-with-taptree should not pin --account; got vis: {vis:?}"
    );
}

#[test]
fn no_descriptor_no_account_pin() {
    // Without `--descriptor`, the v0.17.0 pin doesn't fire at all
    // (template mode owns `--account`). This is unchanged by Phase 6.
    let state = FormState::default();
    let vis = bundle(&state);
    let has_account_pin = vis
        .iter()
        .any(|(k, v)| *k == "--account" && matches!(v, Visibility::PinValue { .. }));
    assert!(
        !has_account_pin,
        "no --descriptor should not pin --account; got vis: {vis:?}"
    );
}

#[test]
fn descriptor_other_disables_still_fire_regardless_of_canonicity() {
    // `--template`, `--threshold`, `--multisig-path-family` disables fire
    // for ANY `--descriptor` presence, canonical or not — they have no
    // canonicity dependence.
    let canonical = state_with_descriptor("pkh(@0)");
    let non_canonical = state_with_descriptor("wsh(andor(pkh(@0),after(N),pk(@1)))");
    for state in [canonical, non_canonical] {
        let vis = bundle(&state);
        for flag in ["--template", "--threshold", "--multisig-path-family"] {
            let disabled = vis
                .iter()
                .any(|(k, v)| *k == flag && matches!(v, Visibility::Disabled));
            assert!(
                disabled,
                "{flag} should be disabled for any --descriptor presence; got vis: {vis:?}"
            );
        }
    }
}
