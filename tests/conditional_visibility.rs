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

/// v0.2 D.2: dispatch by CLI name for the ms / mk schemas. Existing
/// mnemonic-CLI tests keep using `run_conditional` above; new D.2 cells
/// route through this helper.
fn run_conditional_for_cli(
    name: &str,
    state: &FormState,
    cli: &str,
) -> Vec<(&'static str, Visibility)> {
    let schema_for_cli: &schema::Schema = match cli {
        "mnemonic" => &schema::mnemonic::SCHEMA,
        "ms" => &schema::ms::SCHEMA,
        "mk" => &schema::mk::SCHEMA,
        "md" => &schema::md::SCHEMA,
        other => panic!("unknown cli {}", other),
    };
    let sub = schema_for_cli
        .subcommands
        .iter()
        .find(|s| s.name == name)
        .unwrap_or_else(|| panic!("subcommand {} not in {} schema", name, cli));
    sub.conditional
        .unwrap_or_else(|| panic!("subcommand {} ({}) has no conditional fn", name, cli))(state)
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
    // All clap-constrained mnemonic subcommands carry a conditional fn.
    // v0.3: derive-child flipped None → Some (gained --passphrase-stdin
    // conflicts_with passphrase at toolkit v0.13.0). slip39-split /
    // slip39-combine added in v0.3. final-word + seed-xor-{split,combine}
    // have no clap conflicts → stay None.
    for name in [
        "bundle",
        "verify-bundle",
        "convert",
        "export-wallet",
        "derive-child",
        "slip39-split",
        "slip39-combine",
    ] {
        let sub = subcommand(name);
        assert!(
            sub.conditional.is_some(),
            "subcommand {} must have a conditional fn",
            name
        );
    }
    for name in ["final-word", "seed-xor-split", "seed-xor-combine"] {
        let sub = subcommand(name);
        assert!(
            sub.conditional.is_none(),
            "subcommand {} should have no conditional fn (no clap conflicts)",
            name
        );
    }

    // v0.2 D.2 + D.3: ms/mk/md new constrained subcommands also carry conditionals.
    for (cli, name) in [
        ("ms", "encode"),
        ("mk", "encode"),
        ("md", "encode"),
        ("md", "compile"),
        ("md", "address"),
    ] {
        let schema_for_cli: &schema::Schema = match cli {
            "ms" => &schema::ms::SCHEMA,
            "mk" => &schema::mk::SCHEMA,
            "md" => &schema::md::SCHEMA,
            _ => unreachable!(),
        };
        let sub = schema_for_cli
            .subcommands
            .iter()
            .find(|s| s.name == name)
            .unwrap_or_else(|| panic!("{} {} not in schema", cli, name));
        assert!(
            sub.conditional.is_some(),
            "{} {} must have a conditional fn",
            cli,
            name
        );
    }
}

// ─── v0.2 D.2: ms encode + mk encode constraints ─────────────────────────

#[test]
fn cell_d2_ms_encode_phrase_disables_hex() {
    let state = FormState::from_pairs(vec![(
        "--phrase",
        FlagValue::Text("abandon abandon ...".into()),
    )]);
    assert_eq!(
        vis_of(&run_conditional_for_cli("encode", &state, "ms"), "--hex"),
        Visibility::Disabled
    );
}

#[test]
fn cell_d2_ms_encode_hex_disables_phrase_and_hides_language() {
    let state = FormState::from_pairs(vec![("--hex", FlagValue::Text("00112233...".into()))]);
    let vis = run_conditional_for_cli("encode", &state, "ms");
    assert_eq!(vis_of(&vis, "--phrase"), Visibility::Disabled);
    assert_eq!(vis_of(&vis, "--language"), Visibility::Hidden);
}

#[test]
fn cell_d2_ms_encode_both_required_when_neither_set() {
    let state = FormState::default();
    let vis = run_conditional_for_cli("encode", &state, "ms");
    assert_eq!(vis_of(&vis, "--phrase"), Visibility::Required);
    assert_eq!(vis_of(&vis, "--hex"), Visibility::Required);
}

#[test]
fn cell_d2_mk_encode_origin_fingerprint_conflicts_privacy_preserving() {
    let state = FormState::from_pairs(vec![(
        "--origin-fingerprint",
        FlagValue::Text("12345678".into()),
    )]);
    assert_eq!(
        vis_of(
            &run_conditional_for_cli("encode", &state, "mk"),
            "--privacy-preserving"
        ),
        Visibility::Disabled
    );
    let state = FormState::from_pairs(vec![(
        "--privacy-preserving",
        FlagValue::Boolean(true),
    )]);
    assert_eq!(
        vis_of(
            &run_conditional_for_cli("encode", &state, "mk"),
            "--origin-fingerprint"
        ),
        Visibility::Disabled
    );
}

// ─── v0.2 D.3: md encode / compile / address constraints ─────────────────

#[test]
fn cell_d3_md_encode_positional_template_disables_from_policy() {
    let state = FormState::default().with_positionals(vec!["wpkh(@0/**)"]);
    let vis = run_conditional_for_cli("encode", &state, "md");
    assert_eq!(vis_of(&vis, "--from-policy"), Visibility::Disabled);
    assert_eq!(vis_of(&vis, "--context"), Visibility::Hidden);
}

#[test]
fn cell_d3_md_encode_from_policy_requires_context() {
    let state = FormState::from_pairs(vec![
        ("--from-policy", FlagValue::Text("pk(@0)".into())),
    ]);
    let vis = run_conditional_for_cli("encode", &state, "md");
    assert_eq!(vis_of(&vis, "--context"), Visibility::Required);
}

#[test]
fn cell_d3_md_encode_unspendable_key_disabled_by_segwitv0() {
    let state = FormState::from_pairs(vec![
        ("--from-policy", FlagValue::Text("pk(@0)".into())),
        ("--context", FlagValue::Dropdown("segwitv0".into())),
    ]);
    let vis = run_conditional_for_cli("encode", &state, "md");
    assert_eq!(vis_of(&vis, "--unspendable-key"), Visibility::Disabled);
}

#[test]
fn cell_d3_md_compile_unspendable_key_disabled_by_segwitv0() {
    let state = FormState::from_pairs(vec![
        ("--context", FlagValue::Dropdown("segwitv0".into())),
    ]);
    let vis = run_conditional_for_cli("compile", &state, "md");
    assert_eq!(vis_of(&vis, "--unspendable-key"), Visibility::Disabled);
}

#[test]
fn cell_d3_md_address_phrases_disables_template_and_substitutions() {
    let state = FormState::default().with_positionals(vec!["md1abcd..."]);
    let vis = run_conditional_for_cli("address", &state, "md");
    assert_eq!(vis_of(&vis, "--template"), Visibility::Disabled);
    assert_eq!(vis_of(&vis, "--key"), Visibility::Disabled);
    assert_eq!(vis_of(&vis, "--fingerprint"), Visibility::Disabled);
}

// ─── v0.3: drift-fix conditionals (4 XOR pairs × 2 dirs = 8 cells) ──────
// v0.10..v0.13 toolkit cycles added `--passphrase-stdin` /
// `--bip38-passphrase-stdin` to bundle / verify-bundle / convert /
// derive-child; GUI now models the clap `conflicts_with` XOR.

#[test]
fn cell_v0_3_bundle_passphrase_disables_passphrase_stdin() {
    let state = FormState::from_pairs(vec![
        ("--descriptor", FlagValue::Text("wpkh(@0/**)".into())),
        ("--passphrase", FlagValue::Text("secret".into())),
    ]);
    let vis = run_conditional("bundle", &state);
    assert_eq!(vis_of(&vis, "--passphrase-stdin"), Visibility::Disabled);
}

#[test]
fn cell_v0_3_bundle_passphrase_stdin_disables_passphrase() {
    let state = FormState::from_pairs(vec![
        ("--descriptor", FlagValue::Text("wpkh(@0/**)".into())),
        ("--passphrase-stdin", FlagValue::Boolean(true)),
    ]);
    let vis = run_conditional("bundle", &state);
    assert_eq!(vis_of(&vis, "--passphrase"), Visibility::Disabled);
}

#[test]
fn cell_v0_3_verify_bundle_passphrase_disables_passphrase_stdin() {
    let state = FormState::from_pairs(vec![
        ("--descriptor", FlagValue::Text("wpkh(@0/**)".into())),
        ("--passphrase", FlagValue::Text("secret".into())),
    ]);
    let vis = run_conditional("verify-bundle", &state);
    assert_eq!(vis_of(&vis, "--passphrase-stdin"), Visibility::Disabled);
}

#[test]
fn cell_v0_3_verify_bundle_passphrase_stdin_disables_passphrase() {
    let state = FormState::from_pairs(vec![
        ("--descriptor", FlagValue::Text("wpkh(@0/**)".into())),
        ("--passphrase-stdin", FlagValue::Boolean(true)),
    ]);
    let vis = run_conditional("verify-bundle", &state);
    assert_eq!(vis_of(&vis, "--passphrase"), Visibility::Disabled);
}

#[test]
fn cell_v0_3_convert_bip38_passphrase_disables_bip38_passphrase_stdin() {
    let state = FormState::from_pairs(vec![
        ("--bip38-passphrase", FlagValue::Text("hunter2".into())),
    ]);
    let vis = run_conditional("convert", &state);
    assert_eq!(vis_of(&vis, "--bip38-passphrase-stdin"), Visibility::Disabled);
}

#[test]
fn cell_v0_3_convert_bip38_passphrase_stdin_disables_bip38_passphrase() {
    let state = FormState::from_pairs(vec![
        ("--bip38-passphrase-stdin", FlagValue::Boolean(true)),
    ]);
    let vis = run_conditional("convert", &state);
    assert_eq!(vis_of(&vis, "--bip38-passphrase"), Visibility::Disabled);
}

#[test]
fn cell_v0_3_derive_child_passphrase_disables_passphrase_stdin() {
    let state = FormState::from_pairs(vec![
        ("--passphrase", FlagValue::Text("secret".into())),
    ]);
    let vis = run_conditional("derive-child", &state);
    assert_eq!(vis_of(&vis, "--passphrase-stdin"), Visibility::Disabled);
}

#[test]
fn cell_v0_3_derive_child_passphrase_stdin_disables_passphrase() {
    let state = FormState::from_pairs(vec![
        ("--passphrase-stdin", FlagValue::Boolean(true)),
    ]);
    let vis = run_conditional("derive-child", &state);
    assert_eq!(vis_of(&vis, "--passphrase"), Visibility::Disabled);
}

// ─── v0.3: slip39-split conditionals (3 cells) ─────────────────────────

#[test]
fn cell_v0_3_slip39_split_passphrase_disables_passphrase_stdin() {
    let state = FormState::from_pairs(vec![
        ("--passphrase", FlagValue::Text("slip39pp".into())),
    ]);
    let vis = run_conditional("slip39-split", &state);
    assert_eq!(vis_of(&vis, "--passphrase-stdin"), Visibility::Disabled);
}

#[test]
fn cell_v0_3_slip39_split_passphrase_stdin_disables_passphrase() {
    let state = FormState::from_pairs(vec![
        ("--passphrase-stdin", FlagValue::Boolean(true)),
    ]);
    let vis = run_conditional("slip39-split", &state);
    assert_eq!(vis_of(&vis, "--passphrase"), Visibility::Disabled);
}

#[test]
fn cell_v0_3_slip39_split_language_hidden_when_from_entropy() {
    let state = FormState::from_pairs(vec![
        ("--from", FlagValue::NodeValueComposite {
            node: "entropy".into(),
            value: "deadbeef".into(),
        }),
    ]);
    let vis = run_conditional("slip39-split", &state);
    assert_eq!(vis_of(&vis, "--language"), Visibility::Hidden);
}

// ─── v0.3: slip39-combine conditionals (3 cells) ───────────────────────

#[test]
fn cell_v0_3_slip39_combine_passphrase_disables_passphrase_stdin() {
    let state = FormState::from_pairs(vec![
        ("--to", FlagValue::Dropdown("phrase".into())),
        ("--passphrase", FlagValue::Text("slip39pp".into())),
    ]);
    let vis = run_conditional("slip39-combine", &state);
    assert_eq!(vis_of(&vis, "--passphrase-stdin"), Visibility::Disabled);
}

#[test]
fn cell_v0_3_slip39_combine_passphrase_stdin_disables_passphrase() {
    let state = FormState::from_pairs(vec![
        ("--to", FlagValue::Dropdown("phrase".into())),
        ("--passphrase-stdin", FlagValue::Boolean(true)),
    ]);
    let vis = run_conditional("slip39-combine", &state);
    assert_eq!(vis_of(&vis, "--passphrase"), Visibility::Disabled);
}

#[test]
fn cell_v0_3_slip39_combine_language_hidden_when_to_entropy() {
    // --to == "entropy" (the toolkit default) → --language Hidden.
    let state = FormState::from_pairs(vec![
        ("--to", FlagValue::Dropdown("entropy".into())),
    ]);
    let vis = run_conditional("slip39-combine", &state);
    assert_eq!(vis_of(&vis, "--language"), Visibility::Hidden);
}
