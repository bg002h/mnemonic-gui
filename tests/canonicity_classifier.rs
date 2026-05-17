//! v0.8.0 Phase 6 — canonicity classifier corpus.
//!
//! Exercises `classify_descriptor_canonicity` against the 5 canonical
//! wrapper shapes from md-codec's `canonical_origin` table + a
//! representative sample of non-canonical shapes. The classifier MUST
//! be drift-faithful with md-codec's source-of-truth; if md-codec adds a
//! new canonical shape, this corpus needs a paired entry.
//!
//! v0.8.0 SPEC §6.10.7 row 15 — Option-A pattern (no schema encoding):
//! the GUI's `bundle()` Option-A override consumes this classifier to
//! lift the `--account → PinValue(0)` pin in non-canonical mode (the
//! toolkit then consumes `--account N` for §4.12.b default-path
//! inference).

use mnemonic_gui::form::conditional::{classify_descriptor_canonicity, Canonicity};

#[test]
fn classify_pkh_single_key_canonical() {
    assert_eq!(classify_descriptor_canonicity("pkh(@0)"), Canonicity::Canonical);
    assert_eq!(
        classify_descriptor_canonicity("pkh(@0/<0;1>/*)"),
        Canonicity::Canonical
    );
    assert_eq!(
        classify_descriptor_canonicity("pkh(@0/**)"),
        Canonicity::Canonical
    );
    assert_eq!(
        classify_descriptor_canonicity("pkh([deadbeef/44'/0'/0']@0/<0;1>/*)"),
        Canonicity::Canonical
    );
}

#[test]
fn classify_wpkh_single_key_canonical() {
    assert_eq!(classify_descriptor_canonicity("wpkh(@0)"), Canonicity::Canonical);
    assert_eq!(
        classify_descriptor_canonicity("wpkh(@0/<0;1>/*)"),
        Canonicity::Canonical
    );
    assert_eq!(
        classify_descriptor_canonicity("wpkh(@0/**)"),
        Canonicity::Canonical
    );
}

#[test]
fn classify_tr_keypath_only_canonical() {
    // tr(@N) with NO second arg → BIP-86 canonical.
    assert_eq!(classify_descriptor_canonicity("tr(@0)"), Canonicity::Canonical);
    assert_eq!(
        classify_descriptor_canonicity("tr(@0/<0;1>/*)"),
        Canonicity::Canonical
    );
    assert_eq!(
        classify_descriptor_canonicity("tr([deadbeef/86'/0'/0']@0/**)"),
        Canonicity::Canonical
    );
}

#[test]
fn classify_wsh_multi_canonical() {
    assert_eq!(
        classify_descriptor_canonicity("wsh(multi(2,@0,@1,@2))"),
        Canonicity::Canonical
    );
    assert_eq!(
        classify_descriptor_canonicity("wsh(sortedmulti(2,@0,@1))"),
        Canonicity::Canonical
    );
}

#[test]
fn classify_sh_wsh_multi_canonical() {
    assert_eq!(
        classify_descriptor_canonicity("sh(wsh(multi(2,@0,@1)))"),
        Canonicity::Canonical
    );
    assert_eq!(
        classify_descriptor_canonicity("sh(wsh(sortedmulti(2,@0,@1)))"),
        Canonicity::Canonical
    );
}

#[test]
fn classify_user_andor_example_non_canonical() {
    // SPEC §4.12.a — `wsh(andor(...))` is non-canonical (md-codec's table
    // requires multi/sortedmulti directly inside wsh). The user's flagship
    // 3-key time-locked example.
    assert_eq!(
        classify_descriptor_canonicity(
            "wsh(andor(pkh(@0),after(12000000),or_i(and_v(v:pkh(@1),older(4032)),and_v(v:pkh(@2),older(32768)))))"
        ),
        Canonicity::NonCanonical
    );
}

#[test]
fn classify_tr_with_taptree_non_canonical() {
    // tr(@N, <TapTree>) — non-canonical per md-codec's table.
    assert_eq!(
        classify_descriptor_canonicity("tr(@0,multi_a(2,@1,@2))"),
        Canonicity::NonCanonical
    );
    assert_eq!(
        classify_descriptor_canonicity("tr(@0,{pk(@1),pk(@2)})"),
        Canonicity::NonCanonical
    );
}

#[test]
fn classify_tr_nums_non_canonical() {
    // tr(NUMS, <ms>) — non-canonical (NUMS internal key + tap-tree).
    assert_eq!(
        classify_descriptor_canonicity("tr(NUMS,and_v(v:pk(@0),after(12000000)))"),
        Canonicity::NonCanonical
    );
}

#[test]
fn classify_sh_sortedmulti_legacy_non_canonical() {
    // sh(sortedmulti(...)) without nested wsh — non-canonical (legacy P2SH).
    assert_eq!(
        classify_descriptor_canonicity("sh(sortedmulti(2,@0,@1))"),
        Canonicity::NonCanonical
    );
    assert_eq!(
        classify_descriptor_canonicity("sh(multi(2,@0,@1))"),
        Canonicity::NonCanonical
    );
}

#[test]
fn classify_bare_wsh_at_n_non_canonical() {
    // wsh(@N) bare single-key — non-canonical (no inner multi/sortedmulti).
    assert_eq!(classify_descriptor_canonicity("wsh(@0)"), Canonicity::NonCanonical);
    assert_eq!(
        classify_descriptor_canonicity("wsh(pk(@0))"),
        Canonicity::NonCanonical
    );
}

#[test]
fn classify_empty_string_non_canonical() {
    // Empty descriptor (e.g., user hasn't typed anything yet) →
    // NonCanonical. Banner logic gates on `--descriptor` presence
    // separately, so an empty value never triggers the info banner.
    assert_eq!(classify_descriptor_canonicity(""), Canonicity::NonCanonical);
}
