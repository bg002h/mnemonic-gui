//! v0.27.x toolkit envelope-shape smoke cells. Verifies the GUI's envelope
//! consumers parse the post-v0.26.0 wire shapes without panic / shape drift.
//! Added in mnemonic-gui v0.11.1 alongside the toolkit pin bump.
//!
//! Fixtures are copies of the canonical toolkit fixtures from
//! `crates/mnemonic-toolkit/tests/fixtures/` at mnemonic-toolkit-v0.27.2.
//! They live in `tests/fixtures/` so CI doesn't need the toolkit checkout
//! alongside the GUI repo.

#[test]
fn import_wallet_json_envelope_parses_v0_27_x_shape() {
    // Grep-verified at envelope_v0_27_0.json: top-level shape is a JSON ARRAY
    // (each element = one bundle). Per-bundle keys: schema_version, source_format,
    // bundle, roundtrip.
    let fixture = include_str!("fixtures/wallet_import/envelope_v0_27_0.json");
    let parsed: serde_json::Value = serde_json::from_str(fixture).expect("v0.27.0 envelope parses");
    let entries = parsed.as_array().expect("top-level is a JSON array");
    assert!(!entries.is_empty(), "envelope has at least one bundle entry");
    let entry = &entries[0];
    assert_eq!(entry.get("schema_version").and_then(|v| v.as_str()), Some("1"));
    assert!(entry.get("bundle").is_some(), "entry has bundle field");
    // v0.27.0 replaced compact-summary with full BundleJson; verify the new shape
    let bundle = entry.get("bundle").unwrap();
    assert!(bundle.get("descriptor").is_some(), "bundle has descriptor field (full BundleJson, not compact)");
}

#[test]
fn xpub_search_path_of_xpub_match_envelope_parses() {
    let fixture = include_str!("fixtures/v0_27_0_envelopes/path_of_xpub.match.json");
    let parsed: serde_json::Value = serde_json::from_str(fixture).expect("path_of_xpub match envelope parses");
    assert_eq!(parsed.get("result").and_then(|v| v.as_str()), Some("match"));
    assert!(parsed.get("path").is_some());
}

#[test]
fn xpub_search_path_of_xpub_no_match_envelope_parses() {
    let fixture = include_str!("fixtures/v0_27_0_envelopes/path_of_xpub.no_match.json");
    let parsed: serde_json::Value = serde_json::from_str(fixture).expect("path_of_xpub no_match envelope parses");
    assert_eq!(parsed.get("result").and_then(|v| v.as_str()), Some("no_match"));
}

#[test]
fn xpub_search_account_of_descriptor_envelope_parses() {
    // Grep-verified at account_of_descriptor.match.json: per-mode shape has
    // matched_cosigners[i].{cosigner_index, path, template, account} — there
    // is NO top-level `account` field.
    let fixture = include_str!("fixtures/v0_27_0_envelopes/account_of_descriptor.match.json");
    let parsed: serde_json::Value = serde_json::from_str(fixture).expect("account_of_descriptor match envelope parses");
    let matched = parsed.get("matched_cosigners").expect("matched_cosigners present");
    let first = matched.get(0).expect("at least one matched cosigner");
    assert!(first.get("account").is_some(), "matched_cosigners[0].account present");
}

#[test]
fn xpub_search_passphrase_of_xpub_envelope_parses() {
    let fixture = include_str!("fixtures/v0_27_0_envelopes/passphrase_of_xpub.match.json");
    let parsed: serde_json::Value = serde_json::from_str(fixture).expect("passphrase_of_xpub match envelope parses");
    assert_eq!(parsed.get("result").and_then(|v| v.as_str()), Some("match"));
}
