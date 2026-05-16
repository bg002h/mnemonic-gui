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

use mnemonic_gui::secrets::{SECRET_NODE_TYPES, SECRET_SLOT_SUBKEYS};

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
