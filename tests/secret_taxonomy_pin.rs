//! Backstop assertions for `mnemonic_gui::secrets::SECRET_*`.
//!
//! Runs unconditionally under `cargo test` (no `#[ignore]`, no env-var
//! requirements). Catches regression to empty arrays or unexpected
//! membership changes via a minimum-membership pin.
//!
//! The toolkit-side parity tests in `mnemonic-toolkit` v0.14.0+ enforce
//! the SECRET_* taxonomy matches the in-tree `is_secret_bearing()` impls
//! at toolkit-build-time; the compile-time supply-chain guard in
//! `src/secrets.rs` enforces the toolkit's emitted constants match the
//! GUI's v0.3.3 snapshot. This file is the *third* layer of defense:
//! a runtime assertion at GUI test time that the imported constants
//! contain at minimum the four BIP-39-class entries that the GUI's
//! redaction logic depends on. If any future change to the taxonomy
//! drops one of those entries (e.g., refactoring `phrase` to
//! `bip39-phrase`), the persistence-redaction path silently misses
//! the new name; this test flags the regression even if the macro
//! parity tests on the toolkit side somehow miss it.

use mnemonic_gui::secrets::{
    node_type_is_argv_secret, node_type_is_secret, SECRET_NODE_TYPES, SECRET_NODE_TYPES_ARGV,
    SECRET_SLOT_SUBKEYS,
};

#[test]
fn secret_node_types_non_empty() {
    assert!(
        !SECRET_NODE_TYPES.is_empty(),
        "SECRET_NODE_TYPES is empty — persistence::redact_for_persistence \
         silently disables NodeValueComposite redaction. Regression of the \
         v0.3.0..v0.3.2 stub-fallback class of bug."
    );
}

#[test]
fn secret_slot_subkeys_non_empty() {
    assert!(
        !SECRET_SLOT_SUBKEYS.is_empty(),
        "SECRET_SLOT_SUBKEYS is empty — persistence::redact_for_persistence \
         silently disables slot-row redaction. Regression of the \
         v0.3.0..v0.3.2 stub-fallback class of bug."
    );
}

#[test]
fn secret_node_types_contains_bip39_class() {
    for required in &["phrase", "entropy", "xprv", "wif"] {
        assert!(
            SECRET_NODE_TYPES.contains(required),
            "SECRET_NODE_TYPES missing required BIP-39-class entry {:?}; \
             persistence-redaction may silently miss it",
            required
        );
    }
}

#[test]
fn secret_slot_subkeys_contains_bip39_class() {
    for required in &["phrase", "entropy", "xprv", "wif"] {
        assert!(
            SECRET_SLOT_SUBKEYS.contains(required),
            "SECRET_SLOT_SUBKEYS missing required BIP-39-class entry {:?}; \
             persistence-redaction may silently miss it",
            required
        );
    }
}

// ── cycle-3 H3 — wide argv/redaction set drift-guard ────────────────────────
//
// The GUI's argv-mask / run-confirm / persist-redact / paste-warn surfaces key
// on the WIDER `SECRET_NODE_TYPES_ARGV` (narrow + `minikey`) rather than the
// narrow `SECRET_NODE_TYPES`. These runtime pins make a future toolkit-side
// widening of `_ARGV` that the GUI fails to track a TEST FAILURE, not a silent
// secret leak (the H3 class). Mirrors the established min-membership posture.

#[test]
fn secret_node_types_argv_non_empty() {
    assert!(
        !SECRET_NODE_TYPES_ARGV.is_empty(),
        "SECRET_NODE_TYPES_ARGV is empty — the GUI's argv-mask / run-confirm / \
         persist-redact / paste-warn surfaces silently classify NOTHING as \
         secret. Funds-safety regression."
    );
}

#[test]
fn secret_node_types_argv_superset_of_narrow() {
    for narrow in SECRET_NODE_TYPES {
        assert!(
            SECRET_NODE_TYPES_ARGV.contains(narrow),
            "SECRET_NODE_TYPES_ARGV ({:?}) is missing narrow-set entry {:?}; the \
             wide argv/redaction set must be a superset of the narrow persistence \
             set or the wide surfaces silently miss a secret class",
            SECRET_NODE_TYPES_ARGV,
            narrow
        );
    }
}

#[test]
fn secret_node_types_argv_contains_minikey_and_narrow_does_not() {
    assert!(
        SECRET_NODE_TYPES_ARGV.contains(&"minikey"),
        "SECRET_NODE_TYPES_ARGV must contain `minikey` (a Casascius mini PRIVATE \
         KEY) — cycle-3 H3 routes minikey through the wide argv/redaction set"
    );
    assert!(
        !SECRET_NODE_TYPES.contains(&"minikey"),
        "SECRET_NODE_TYPES (narrow) must NOT contain `minikey` — the two sets are \
         intentionally distinct; the delta is exactly {{minikey}}. If this fires, \
         the narrow set was widened and the cycle-3 dual-set rationale needs review"
    );
}

#[test]
fn node_type_predicates_classify_minikey_widely_only() {
    // The two named predicates encode the dual-set contract: the wide predicate
    // sees minikey, the narrow one does not.
    assert!(
        node_type_is_argv_secret("minikey"),
        "node_type_is_argv_secret(\"minikey\") must be true (wide argv/redaction set)"
    );
    assert!(
        !node_type_is_secret("minikey"),
        "node_type_is_secret(\"minikey\") must be false (narrow persistence set)"
    );
    // Sanity: a shared secret-class node is in BOTH; a watch-only node in NEITHER.
    assert!(node_type_is_argv_secret("phrase") && node_type_is_secret("phrase"));
    assert!(!node_type_is_argv_secret("xpub") && !node_type_is_secret("xpub"));
}

/// Compile-time supply-chain guard for the WIDE set, mirroring the narrow set's
/// `v0_3_canonical_fallback` belt-and-suspenders. Lives in a SEPARATE sibling
/// mod (per spec Q3) so it outlives the v0.5.0 retirement of the narrow snapshot.
/// A toolkit pin bump that changes `SECRET_NODE_TYPES_ARGV` fails to COMPILE
/// here until the maintainer explicitly acknowledges the change.
mod argv_canonical_fallback {
    use mnemonic_gui::secrets::SECRET_NODE_TYPES_ARGV;

    /// Snapshot of `SECRET_NODE_TYPES_ARGV` as imported at cycle-3 (toolkit
    /// `mnemonic-toolkit-v0.60.0`): the narrow 8 + `minikey`.
    const ARGV_SNAPSHOT: &[&str] = &[
        "phrase",
        "entropy",
        "xprv",
        "wif",
        "ms1",
        "bip38",
        "electrum-phrase",
        "seedqr",
        "minikey",
    ];

    const fn slice_eq(a: &[&str], b: &[&str]) -> bool {
        if a.len() != b.len() {
            return false;
        }
        let mut i = 0;
        while i < a.len() {
            let (x, y) = (a[i].as_bytes(), b[i].as_bytes());
            if x.len() != y.len() {
                return false;
            }
            let mut j = 0;
            while j < x.len() {
                if x[j] != y[j] {
                    return false;
                }
                j += 1;
            }
            i += 1;
        }
        true
    }

    const _: () = assert!(
        slice_eq(SECRET_NODE_TYPES_ARGV, ARGV_SNAPSHOT),
        "supply-chain drift: mnemonic_toolkit::secret_taxonomy::SECRET_NODE_TYPES_ARGV \
         diverged from the cycle-3 committed snapshot. Either update \
         argv_canonical_fallback::ARGV_SNAPSHOT to match the new toolkit pin (and \
         document the change in CHANGELOG + re-audit the GUI argv/redaction surfaces), \
         or revert the toolkit dep tag bump."
    );

    #[test]
    fn argv_snapshot_matches_toolkit_import() {
        assert_eq!(
            SECRET_NODE_TYPES_ARGV, ARGV_SNAPSHOT,
            "wide-set snapshot drifted from toolkit import"
        );
    }
}
