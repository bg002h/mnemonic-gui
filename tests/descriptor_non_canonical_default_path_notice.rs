//! v0.8.1 F3 — banner-helper for non-canonical descriptor + default-path
//! inference. Closes the v0.19.0 I2 perceptibility gap: CLI users see the
//! stderr info notice on default-path emission; GUI users get the same
//! behavior but no visual cue. The banner is rendered adjacent to the slot
//! grid in `main.rs` via `descriptor_non_canonical_default_path_notice`.
//!
//! The banner text is paraphrase-accepted (per v0.20.0 plan §2 GUI row 1):
//! it cribs from `bundle.rs::emit_default_path_notice` but uses a ℹ glyph
//! prefix and literal `<coin>` / `<account>` substitution rather than the
//! toolkit's `idx_list` enumeration (defaulted_indices computation is
//! deferred to a future FOLLOWUP if/when GUI gains descriptor-parse infra).

use mnemonic_gui::form::conditional::descriptor_non_canonical_default_path_notice;

/// Cell 14 — canonical descriptor returns None (no banner).
#[test]
fn canonical_descriptor_returns_none() {
    let banner = descriptor_non_canonical_default_path_notice("pkh(@0)", "mainnet", 0);
    assert!(
        banner.is_none(),
        "canonical descriptor must not render banner; got: {banner:?}"
    );
}

/// Cell 15 — non-canonical wsh(andor(...)) + mainnet + account=0 →
/// banner mentioning `m/48'/0'/0'/2'` AND the literal `--slot @N.path=m/...`
/// override flag (Phase 3 R0 I1 fold — banner must remain operationally
/// complete vs the toolkit's stderr notice).
#[test]
fn non_canonical_mainnet_account_0_renders_banner_with_mainnet_path() {
    let banner = descriptor_non_canonical_default_path_notice(
        "wsh(andor(pkh(@0),after(12000000),pk(@1)))",
        "mainnet",
        0,
    );
    let s = banner.expect("non-canonical descriptor must render banner");
    assert!(
        s.contains("m/48'/0'/0'/2'"),
        "banner must mention the mainnet account-0 default path m/48'/0'/0'/2'; got: {s}"
    );
    assert!(
        s.contains("BIP-48"),
        "banner must mention BIP-48 convention; got: {s}"
    );
    assert!(
        s.contains("--slot @N.path=m/..."),
        "banner must mention the literal --slot @N.path=m/... override flag; got: {s}"
    );
    assert!(
        s.contains("[fp/path]@N"),
        "banner must mention the [fp/path]@N override form; got: {s}"
    );
}

/// Cell 16 — non-canonical + testnet + account=7 → banner mentioning
/// `m/48'/1'/7'/2'` (testnet coin_type=1).
#[test]
fn non_canonical_testnet_account_7_renders_banner_with_testnet_path() {
    let banner = descriptor_non_canonical_default_path_notice(
        "wsh(andor(pkh(@0),after(12000000),pk(@1)))",
        "testnet",
        7,
    );
    let s = banner.expect("non-canonical descriptor must render banner");
    assert!(
        s.contains("m/48'/1'/7'/2'"),
        "banner must mention testnet account-7 default path m/48'/1'/7'/2'; got: {s}"
    );
}

/// Cell 17 — empty descriptor returns None (R3 mitigation: banner-helper
/// must gate on descriptor presence, not just canonicity-classifier
/// verdict). classify_descriptor_canonicity returns NonCanonical for
/// empty per v0.8.0 semantics, so the banner-helper itself must guard.
#[test]
fn empty_descriptor_returns_none() {
    let banner = descriptor_non_canonical_default_path_notice("", "mainnet", 0);
    assert!(
        banner.is_none(),
        "empty descriptor must not render banner; got: {banner:?}"
    );
}
