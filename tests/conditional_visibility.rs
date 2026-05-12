//! Enumeration tests for every active upstream clap `conflicts_with` /
//! `required_unless_present_any` constraint (SPEC §5 + Phase 5 IMPL_PLAN).
//!
//! 11 cells, one per constraint as enumerated below:
//!
//!   bundle:
//!     1. --template required_unless_present_any [--descriptor, --descriptor-file]
//!     2. --descriptor conflicts_with --descriptor-file
//!   verify-bundle:
//!     3. --template required_unless_present_any [--descriptor, --descriptor-file]
//!     4. --descriptor conflicts_with --descriptor-file
//!     5. --ms1 conflicts_with --bundle-json
//!     6. --mk1 required_unless_present --bundle-json
//!     7. --mk1 conflicts_with --bundle-json
//!     8. --md1 required_unless_present --bundle-json
//!     9. --md1 conflicts_with --bundle-json
//!   convert:
//!    10. --passphrase-stdin conflicts_with --passphrase
//!   export-wallet:
//!    11. --template conflicts_with --descriptor

use mnemonic_gui::schema::{self, FlagValue, FormState, Visibility};

fn subcommand(name: &str) -> &'static schema::SubcommandSchema {
    schema::mnemonic::SCHEMA
        .subcommands
        .iter()
        .find(|s| s.name == name)
        .unwrap_or_else(|| panic!("subcommand {} not in schema", name))
}

fn run_conditional(name: &str, state: &FormState) -> Vec<(&'static str, Visibility)> {
    let sub = subcommand(name);
    sub.conditional
        .unwrap_or_else(|| panic!("subcommand {} has no conditional fn", name))(state)
}

fn vis_of(vis: &[(&'static str, Visibility)], flag: &str) -> Visibility {
    vis.iter()
        .find(|(k, _)| *k == flag)
        .map(|(_, v)| *v)
        .unwrap_or(Visibility::Visible) // default per SPEC §5 + module doc
}

// ─── bundle constraints ──────────────────────────────────────────────────

#[test]
fn cell_01_bundle_template_required_unless_any_descriptor() {
    // Empty form state → --template Required.
    let empty = FormState::default();
    assert_eq!(
        vis_of(&run_conditional("bundle", &empty), "--template"),
        Visibility::Required
    );
    // Populating --descriptor relaxes the requirement.
    let with_desc = FormState::from_pairs(vec![(
        "--descriptor",
        FlagValue::Text("wpkh(@0/**)".into()),
    )]);
    assert_eq!(
        vis_of(&run_conditional("bundle", &with_desc), "--template"),
        Visibility::Visible
    );
    // Populating --descriptor-file likewise.
    let with_desc_file = FormState::from_pairs(vec![(
        "--descriptor-file",
        FlagValue::Path("/tmp/d.txt".into()),
    )]);
    assert_eq!(
        vis_of(&run_conditional("bundle", &with_desc_file), "--template"),
        Visibility::Visible
    );
}

#[test]
fn cell_02_bundle_descriptor_conflicts_descriptor_file() {
    let with_desc = FormState::from_pairs(vec![(
        "--descriptor",
        FlagValue::Text("wpkh(@0/**)".into()),
    )]);
    let vis = run_conditional("bundle", &with_desc);
    assert_eq!(vis_of(&vis, "--descriptor-file"), Visibility::Disabled);

    let with_desc_file = FormState::from_pairs(vec![(
        "--descriptor-file",
        FlagValue::Path("/tmp/d.txt".into()),
    )]);
    let vis = run_conditional("bundle", &with_desc_file);
    assert_eq!(vis_of(&vis, "--descriptor"), Visibility::Disabled);
}

// ─── verify-bundle constraints ───────────────────────────────────────────

#[test]
fn cell_03_verify_bundle_template_required_unless_any_descriptor() {
    let empty = FormState::default();
    assert_eq!(
        vis_of(&run_conditional("verify-bundle", &empty), "--template"),
        Visibility::Required
    );
    let with_desc = FormState::from_pairs(vec![(
        "--descriptor",
        FlagValue::Text("wpkh(@0/**)".into()),
    )]);
    assert_eq!(
        vis_of(&run_conditional("verify-bundle", &with_desc), "--template"),
        Visibility::Visible
    );
}

#[test]
fn cell_04_verify_bundle_descriptor_conflicts_descriptor_file() {
    let with_desc = FormState::from_pairs(vec![(
        "--descriptor",
        FlagValue::Text("wpkh(@0/**)".into()),
    )]);
    let vis = run_conditional("verify-bundle", &with_desc);
    assert_eq!(vis_of(&vis, "--descriptor-file"), Visibility::Disabled);

    let with_desc_file = FormState::from_pairs(vec![(
        "--descriptor-file",
        FlagValue::Path("/tmp/d.txt".into()),
    )]);
    let vis = run_conditional("verify-bundle", &with_desc_file);
    assert_eq!(vis_of(&vis, "--descriptor"), Visibility::Disabled);
}

#[test]
fn cell_05_verify_bundle_ms1_conflicts_bundle_json() {
    let with_bundle_json = FormState::from_pairs(vec![(
        "--bundle-json",
        FlagValue::Path("/tmp/bundle.json".into()),
    )]);
    let vis = run_conditional("verify-bundle", &with_bundle_json);
    assert_eq!(vis_of(&vis, "--ms1"), Visibility::Disabled);

    // Reverse: with --ms1 set, --bundle-json must be disabled.
    let with_ms1 = FormState::from_pairs(vec![("--ms1", FlagValue::Text("ms1xyz...".into()))]);
    let vis = run_conditional("verify-bundle", &with_ms1);
    assert_eq!(vis_of(&vis, "--bundle-json"), Visibility::Disabled);
}

#[test]
fn cell_06_verify_bundle_mk1_required_unless_bundle_json() {
    // With --bundle-json absent → --mk1 Required.
    let empty = FormState::default();
    assert_eq!(
        vis_of(&run_conditional("verify-bundle", &empty), "--mk1"),
        Visibility::Required
    );
    // With --bundle-json present → --mk1 not Required (Disabled per cell_07).
    let with_bundle_json = FormState::from_pairs(vec![(
        "--bundle-json",
        FlagValue::Path("/tmp/bundle.json".into()),
    )]);
    let vis = run_conditional("verify-bundle", &with_bundle_json);
    assert_ne!(vis_of(&vis, "--mk1"), Visibility::Required);
}

#[test]
fn cell_07_verify_bundle_mk1_conflicts_bundle_json() {
    let with_bundle_json = FormState::from_pairs(vec![(
        "--bundle-json",
        FlagValue::Path("/tmp/bundle.json".into()),
    )]);
    assert_eq!(
        vis_of(&run_conditional("verify-bundle", &with_bundle_json), "--mk1"),
        Visibility::Disabled
    );
    let with_mk1 = FormState::from_pairs(vec![("--mk1", FlagValue::Text("mk1xyz...".into()))]);
    assert_eq!(
        vis_of(&run_conditional("verify-bundle", &with_mk1), "--bundle-json"),
        Visibility::Disabled
    );
}

#[test]
fn cell_08_verify_bundle_md1_required_unless_bundle_json() {
    let empty = FormState::default();
    assert_eq!(
        vis_of(&run_conditional("verify-bundle", &empty), "--md1"),
        Visibility::Required
    );
    let with_bundle_json = FormState::from_pairs(vec![(
        "--bundle-json",
        FlagValue::Path("/tmp/bundle.json".into()),
    )]);
    assert_ne!(
        vis_of(&run_conditional("verify-bundle", &with_bundle_json), "--md1"),
        Visibility::Required
    );
}

#[test]
fn cell_09_verify_bundle_md1_conflicts_bundle_json() {
    let with_bundle_json = FormState::from_pairs(vec![(
        "--bundle-json",
        FlagValue::Path("/tmp/bundle.json".into()),
    )]);
    assert_eq!(
        vis_of(&run_conditional("verify-bundle", &with_bundle_json), "--md1"),
        Visibility::Disabled
    );
    let with_md1 = FormState::from_pairs(vec![("--md1", FlagValue::Text("md1xyz...".into()))]);
    assert_eq!(
        vis_of(&run_conditional("verify-bundle", &with_md1), "--bundle-json"),
        Visibility::Disabled
    );
}

// ─── convert constraints ─────────────────────────────────────────────────

#[test]
fn cell_10_convert_passphrase_stdin_conflicts_passphrase() {
    let with_pass = FormState::from_pairs(vec![("--passphrase", FlagValue::Text("p".into()))]);
    assert_eq!(
        vis_of(&run_conditional("convert", &with_pass), "--passphrase-stdin"),
        Visibility::Disabled
    );
    let with_stdin = FormState::from_pairs(vec![("--passphrase-stdin", FlagValue::Boolean(true))]);
    assert_eq!(
        vis_of(&run_conditional("convert", &with_stdin), "--passphrase"),
        Visibility::Disabled
    );
}

// ─── export-wallet constraints ───────────────────────────────────────────

#[test]
fn cell_11_export_wallet_template_conflicts_descriptor() {
    let with_template = FormState::from_pairs(vec![(
        "--template",
        FlagValue::Dropdown("bip84".into()),
    )]);
    assert_eq!(
        vis_of(&run_conditional("export-wallet", &with_template), "--descriptor"),
        Visibility::Disabled
    );
    let with_desc = FormState::from_pairs(vec![(
        "--descriptor",
        FlagValue::Text("wpkh(@0/**)".into()),
    )]);
    assert_eq!(
        vis_of(&run_conditional("export-wallet", &with_desc), "--template"),
        Visibility::Disabled
    );
}

/// Phase 5 R1 I-1 fold: export-wallet runtime pre-check at
/// `export_wallet.rs:215-219` rejects with BadInput when neither
/// `--template` nor `--descriptor` is supplied. The GUI conditional marks
/// both as Required to surface this pre-Run rather than after a
/// surprise non-zero exit. This is the only runtime-pre-check constraint
/// folded into Phase 5; if more similar pre-checks surface in future
/// upstream versions, model them via the same pattern.
#[test]
fn cell_12_export_wallet_template_or_descriptor_required_when_neither_set() {
    let empty = FormState::default();
    let vis = run_conditional("export-wallet", &empty);
    assert_eq!(vis_of(&vis, "--template"), Visibility::Required);
    assert_eq!(vis_of(&vis, "--descriptor"), Visibility::Required);

    // With either populated, neither is Required (the runtime check passes).
    let with_template = FormState::from_pairs(vec![(
        "--template",
        FlagValue::Dropdown("bip84".into()),
    )]);
    let vis = run_conditional("export-wallet", &with_template);
    assert_ne!(vis_of(&vis, "--template"), Visibility::Required);
    // --descriptor is Disabled (via cell_11 path), not Required.
    assert_eq!(vis_of(&vis, "--descriptor"), Visibility::Disabled);
}

// ─── coverage guard ──────────────────────────────────────────────────────

#[test]
fn coverage_all_constrained_subcommands_have_conditional_fn() {
    // The four subcommands with clap-level constraints must carry a
    // conditional fn pointer; derive-child has none and stays None.
    for name in ["bundle", "verify-bundle", "convert", "export-wallet"] {
        let sub = subcommand(name);
        assert!(
            sub.conditional.is_some(),
            "subcommand {} must have a conditional fn",
            name
        );
    }
    assert!(subcommand("derive-child").conditional.is_none());
}
