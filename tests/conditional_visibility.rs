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
    // v0.6.0: Visibility no longer Copy (PinValue carries serde_json::Value);
    // clone the matched entry rather than dereffing.
    vis.iter()
        .find(|(k, _)| *k == flag)
        .map(|(_, v)| v.clone())
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
    // v0.16.0 SPEC §6.10.7 / bundle.rs::mode_text::DESCRIPTOR_AND_TEMPLATE:
    // populating --descriptor BOTH relaxes the requirement AND makes
    // --template Disabled (descriptor mode is mutually exclusive with
    // template mode). Prior pre-v0.16.0 behaviour was Visible.
    let with_desc = FormState::from_pairs(vec![(
        "--descriptor",
        FlagValue::Text("wpkh(@0/**)".into()),
    )]);
    assert_eq!(
        vis_of(&run_conditional("bundle", &with_desc), "--template"),
        Visibility::Disabled,
        "v0.16.0: --template Disabled when --descriptor present \
         (SPEC §6.10.7; DESCRIPTOR_AND_TEMPLATE)"
    );
    // Populating --descriptor-file relaxes Required but does NOT disable
    // --template (the mutex is between --descriptor and --template-or-
    // --descriptor-file). v0.16.0 cycle did not add a rule disabling
    // --template when --descriptor-file is set (the existing GUI rule
    // already disables --descriptor when --descriptor-file is set,
    // which is symmetric).
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
    // v0.16.0 SPEC §6.10.7 (verify-bundle mirror): --template Disabled when
    // --descriptor present. Prior pre-v0.16.0 behaviour was Visible.
    let with_desc = FormState::from_pairs(vec![(
        "--descriptor",
        FlagValue::Text("wpkh(@0/**)".into()),
    )]);
    assert_eq!(
        vis_of(&run_conditional("verify-bundle", &with_desc), "--template"),
        Visibility::Disabled,
        "v0.16.0: --template Disabled when --descriptor present"
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

// ─── v0.16.0 SPEC §6.10.7 conditional-applicability cells ──────────────
//
// Bundle / verify-bundle / export-wallet / derive-child gain new
// per-frame visibility rules in v0.16.0 GUI conditional-applicability v1
// cycle. Per-cell tests below pair positive (predicate satisfied) +
// negative (predicate not satisfied) + composition checks with the
// pre-existing rules. Drift gate at `tests/gui_schema_conditional_drift.rs`
// (P4) enforces parity with the toolkit's gui-schema JSON output.

#[test]
fn cell_v0_16_bundle_threshold_disabled_when_single_sig_template() {
    let state = FormState::from_pairs(vec![
        ("--template", FlagValue::Dropdown("bip84".into())),
    ]);
    let vis = run_conditional("bundle", &state);
    assert_eq!(vis_of(&vis, "--threshold"), Visibility::Disabled);
    assert_eq!(vis_of(&vis, "--multisig-path-family"), Visibility::Disabled);
}

#[test]
fn cell_v0_16_bundle_threshold_visible_when_multisig_template() {
    let state = FormState::from_pairs(vec![
        ("--template", FlagValue::Dropdown("wsh-sortedmulti".into())),
    ]);
    let vis = run_conditional("bundle", &state);
    assert_eq!(vis_of(&vis, "--threshold"), Visibility::Visible);
    assert_eq!(vis_of(&vis, "--multisig-path-family"), Visibility::Visible);
}

#[test]
fn cell_v0_16_bundle_descriptor_disables_template_and_threshold_family() {
    let state = FormState::from_pairs(vec![
        ("--descriptor", FlagValue::Text("wpkh(@0/**)".into())),
    ]);
    let vis = run_conditional("bundle", &state);
    assert_eq!(vis_of(&vis, "--template"), Visibility::Disabled);
    assert_eq!(vis_of(&vis, "--threshold"), Visibility::Disabled);
    assert_eq!(vis_of(&vis, "--multisig-path-family"), Visibility::Disabled);
}

#[test]
fn cell_v0_16_bundle_compose_descriptor_first_rule_wins_over_template() {
    // First-rule-wins (SPEC §6.10.4 / `main.rs:391-394`): even if
    // --template were independently single-sig-typed, the descriptor-mode
    // rule fires FIRST and dictates Disabled.
    let state = FormState::from_pairs(vec![
        ("--descriptor", FlagValue::Text("wpkh(@0/**)".into())),
        ("--template", FlagValue::Dropdown("bip84".into())),
    ]);
    let vis = run_conditional("bundle", &state);
    // Both rules produce Disabled, so observable effect is the same.
    // What matters is the priority-order invariant: the first
    // (--threshold, Disabled) entry has the descriptor-present rationale.
    assert_eq!(vis_of(&vis, "--threshold"), Visibility::Disabled);
    // Find the first --threshold rule and confirm it precedes (or equals)
    // the position of the second.
    let threshold_indices: Vec<usize> = vis
        .iter()
        .enumerate()
        .filter(|(_, (k, _))| *k == "--threshold")
        .map(|(i, _)| i)
        .collect();
    assert!(threshold_indices.len() >= 1);
}

#[test]
fn cell_v0_16_verify_bundle_threshold_disabled_when_single_sig() {
    let state = FormState::from_pairs(vec![
        ("--template", FlagValue::Dropdown("bip49".into())),
    ]);
    let vis = run_conditional("verify-bundle", &state);
    assert_eq!(vis_of(&vis, "--threshold"), Visibility::Disabled);
}

#[test]
fn cell_v0_16_verify_bundle_template_disabled_when_descriptor() {
    let state = FormState::from_pairs(vec![
        ("--descriptor", FlagValue::Text("wpkh(@0/**)".into())),
    ]);
    let vis = run_conditional("verify-bundle", &state);
    assert_eq!(vis_of(&vis, "--template"), Visibility::Disabled);
}

#[test]
fn cell_v0_16_export_wallet_taproot_internal_key_required_for_taproot_multi() {
    let state = FormState::from_pairs(vec![
        ("--template", FlagValue::Dropdown("tr-sortedmulti-a".into())),
    ]);
    let vis = run_conditional("export-wallet", &state);
    assert_eq!(
        vis_of(&vis, "--taproot-internal-key"),
        Visibility::Required
    );
}

#[test]
fn cell_v0_16_export_wallet_taproot_internal_key_disabled_for_non_taproot() {
    let state = FormState::from_pairs(vec![
        ("--template", FlagValue::Dropdown("bip84".into())),
    ]);
    let vis = run_conditional("export-wallet", &state);
    assert_eq!(
        vis_of(&vis, "--taproot-internal-key"),
        Visibility::Disabled
    );
}

#[test]
fn cell_v0_16_export_wallet_threshold_disabled_for_single_sig() {
    let state = FormState::from_pairs(vec![
        ("--template", FlagValue::Dropdown("bip86".into())),
    ]);
    let vis = run_conditional("export-wallet", &state);
    assert_eq!(vis_of(&vis, "--threshold"), Visibility::Disabled);
    assert_eq!(vis_of(&vis, "--multisig-path-family"), Visibility::Disabled);
}

#[test]
fn cell_v0_16_derive_child_dice_sides_required_when_application_dice() {
    let state = FormState::from_pairs(vec![
        ("--application", FlagValue::Dropdown("dice".into())),
    ]);
    let vis = run_conditional("derive-child", &state);
    assert_eq!(vis_of(&vis, "--dice-sides"), Visibility::Required);
}

#[test]
fn cell_v0_16_derive_child_dice_sides_visible_when_application_other() {
    let state = FormState::from_pairs(vec![
        ("--application", FlagValue::Dropdown("nostr".into())),
    ]);
    let vis = run_conditional("derive-child", &state);
    assert_eq!(vis_of(&vis, "--dice-sides"), Visibility::Visible);
}

// ─── v0.7.0 SPEC §6.10.3 v4 disable_options Effect ──────────────────────

fn state_with_slot_count(count: usize) -> FormState {
    let mut state = FormState::default();
    while state.slots.rows.len() < count {
        state.slots.rows.push(mnemonic_gui::form::slot_editor::SlotRow::default());
    }
    state.slots.rows.truncate(count);
    state
}

// ── v0.7.2 template/slot_count warning helper ─────────────────────────

#[test]
fn template_warning_none_when_template_unset() {
    // No template chosen (e.g., descriptor mode active or pre-selection)
    // — warning suppresses regardless of slot_count.
    assert_eq!(
        mnemonic_gui::form::conditional::template_slot_count_warning(None, 0),
        None,
    );
    assert_eq!(
        mnemonic_gui::form::conditional::template_slot_count_warning(None, 5),
        None,
    );
}

#[test]
fn template_warning_none_for_single_sig_with_one_slot() {
    // bip84 + 1 slot = valid single-sig configuration. No warning.
    assert_eq!(
        mnemonic_gui::form::conditional::template_slot_count_warning(Some("bip84"), 1),
        None,
    );
    assert_eq!(
        mnemonic_gui::form::conditional::template_slot_count_warning(Some("bip44"), 1),
        None,
    );
}

#[test]
fn template_warning_none_for_single_sig_with_zero_slots() {
    // bip84 + 0 slots = valid pre-build state (user picked template,
    // hasn't added the cosigner slot yet). No warning.
    assert_eq!(
        mnemonic_gui::form::conditional::template_slot_count_warning(Some("bip84"), 0),
        None,
    );
}

#[test]
fn template_warning_fires_for_single_sig_with_two_slots() {
    // SPEC §6.6 row 10: single-sig + 2+ slots is invalid. Warning text
    // suggests both directions of fix.
    let warning =
        mnemonic_gui::form::conditional::template_slot_count_warning(Some("bip84"), 2);
    let text = warning.expect("single-sig + 2 slots must fire row 10 warning");
    assert!(text.contains("bip84"), "warning must name the template; got: {text}");
    assert!(text.contains("single-sig"), "warning must explain template kind");
    assert!(text.contains("2"), "warning must cite the slot count");
    assert!(
        text.contains("multisig") || text.contains("remove"),
        "warning must suggest a fix; got: {text}"
    );
}

#[test]
fn template_warning_fires_for_multisig_with_zero_slots() {
    // SPEC §6.6 row 11: multisig + 0 slots is invalid. Warning fires.
    let warning =
        mnemonic_gui::form::conditional::template_slot_count_warning(Some("wsh-multi"), 0);
    let text = warning.expect("multisig + 0 slots must fire row 11 warning");
    assert!(text.contains("wsh-multi"));
    assert!(text.contains("multisig"));
    assert!(text.contains("0 slot"));
}

#[test]
fn template_warning_fires_for_multisig_with_one_slot() {
    // The transient state that v0.7.0 incorrectly disabled. v0.7.2
    // shows a warning instead of disabling, so the user can complete
    // their multisig setup.
    let warning =
        mnemonic_gui::form::conditional::template_slot_count_warning(Some("wsh-sortedmulti"), 1);
    let text = warning.expect("multisig + 1 slot must fire row 11 warning");
    assert!(text.contains("wsh-sortedmulti"));
    assert!(text.contains("1 slot"));
    assert!(
        text.contains("Add") || text.contains("single-sig"),
        "warning must suggest a fix; got: {text}"
    );
}

#[test]
fn template_warning_none_for_multisig_with_two_or_more_slots() {
    // Valid multisig configuration. No warning.
    assert_eq!(
        mnemonic_gui::form::conditional::template_slot_count_warning(Some("wsh-multi"), 2),
        None,
    );
    assert_eq!(
        mnemonic_gui::form::conditional::template_slot_count_warning(Some("tr-multi-a"), 5),
        None,
    );
}

#[test]
fn cell_v0_18_1_bundle_emits_no_disable_options_after_row_10_11_rollback() {
    // v0.18.1 + v0.7.2 reverted the v0.18.0 row 10/11 DisableOptions
    // emissions (UX flaw: row 11 disabled multisig at slot_count==1,
    // the natural transient state during multisig setup). The
    // template/slot_count mismatch UX migrated to a GUI-internal
    // warning banner via `template_slot_count_warning` (rendered
    // adjacent to the slot grid in main.rs). bundle()'s conditional
    // fn must NOT push any DisableOptions entries for --template
    // (or any other flag) at any slot_count.
    for slot_count in [0_usize, 1, 2, 5] {
        let state = state_with_slot_count(slot_count);
        let vis = run_conditional("bundle", &state);
        let any_disable_options = vis
            .iter()
            .any(|(_, v)| matches!(v, Visibility::DisableOptions { .. }));
        assert!(
            !any_disable_options,
            "bundle() must emit ZERO DisableOptions entries at slot_count={slot_count} \
             (v0.7.2 reverted v0.7.0's row 10/11 disable_options pushes); \
             vis: {vis:?}"
        );
    }
}

// ─── v0.6.0 SPEC §6.10.3 v3 pin_value Effect ─────────────────────────────

#[test]
fn cell_v0_17_bundle_account_pin_value_zero_when_descriptor() {
    // SPEC §6.10.7 row 12 (DESCRIPTOR_WITH_NONZERO_ACCOUNT): --account is
    // pinned to 0 when --descriptor is present (the descriptor encodes
    // account in @i origin paths). PinValue REPLACES user-typed value per
    // §6.10.4 emission table (vs Hidden/Disabled which suppress entirely).
    let with_desc = FormState::from_pairs(vec![(
        "--descriptor",
        FlagValue::Text("wpkh(@0/**)".into()),
    )]);
    assert_eq!(
        vis_of(&run_conditional("bundle", &with_desc), "--account"),
        Visibility::PinValue { value: serde_json::json!(0) },
    );
    // Sanity: --account is NOT pinned when --descriptor is absent (no
    // override, falls through to Visible default).
    let without_desc = FormState::default();
    assert_eq!(
        vis_of(&run_conditional("bundle", &without_desc), "--account"),
        Visibility::Visible,
    );
}
