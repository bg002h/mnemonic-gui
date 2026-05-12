//! Byte-exact argv-assembly tests (SPEC §6 + §6.7).
//!
//! One cell per `FlagKind` variant against the `mnemonic` schema. Slot
//! tests live in `tests/argv_assembler_slot.rs` (Phase 3).

use mnemonic_gui::form::invocation::assemble_argv;
use mnemonic_gui::schema::{
    self, FlagValue, FormState, TaggedOrIndexedValue, TimestampValue,
};

fn subcommand(name: &str) -> &'static schema::SubcommandSchema {
    schema::mnemonic::SCHEMA
        .subcommands
        .iter()
        .find(|s| s.name == name)
        .unwrap_or_else(|| panic!("subcommand {} not in schema", name))
}

#[test]
fn cell_1_bundle_phrase_minimal_argv() {
    // Covers Text + Number + Dropdown + Boolean emission rules. Boolean
    // `false` must NOT emit; absent fields must NOT emit.
    let state = FormState::from_pairs(vec![
        ("--network", FlagValue::Dropdown("mainnet".into())),
        ("--template", FlagValue::Dropdown("bip84".into())),
        ("--passphrase", FlagValue::Text("hunter2".into())),
        ("--account", FlagValue::Number(0)),
        ("--json", FlagValue::Boolean(true)),
        ("--privacy-preserving", FlagValue::Boolean(false)), // omit
    ]);
    let argv = assemble_argv(&schema::mnemonic::SCHEMA, subcommand("bundle"), &state);
    assert_eq!(
        argv,
        vec![
            "mnemonic", "bundle", "--network", "mainnet", "--template", "bip84",
            "--passphrase", "hunter2", "--account", "0", "--json",
        ]
    );
}

#[test]
fn cell_2_convert_from_to_argv() {
    // Covers NodeValueComposite (--from) + repeating Dropdown (--to).
    // Schema declares --to as repeating; emit one --to per FormState entry
    // in form-state order.
    let state = FormState::from_pairs(vec![
        (
            "--from",
            FlagValue::NodeValueComposite {
                node: "phrase".into(),
                value: "abandon abandon abandon".into(),
            },
        ),
        ("--to", FlagValue::Dropdown("xpub".into())),
        ("--to", FlagValue::Dropdown("address".into())),
        ("--network", FlagValue::Dropdown("mainnet".into())),
    ]);
    let argv = assemble_argv(&schema::mnemonic::SCHEMA, subcommand("convert"), &state);
    assert_eq!(
        argv,
        vec![
            "mnemonic",
            "convert",
            "--from",
            "phrase=abandon abandon abandon",
            "--to",
            "xpub",
            "--to",
            "address",
            "--network",
            "mainnet",
        ]
    );
}

#[test]
fn cell_3_export_wallet_range_timestamp_argv() {
    // Covers Range + Timestamp (both Now and Unix variants reachable in
    // separate fixtures; this fixture pins Range + Timestamp::Unix).
    let state = FormState::from_pairs(vec![
        ("--template", FlagValue::Dropdown("bip84".into())),
        ("--range", FlagValue::Range(0, 999)),
        ("--timestamp", FlagValue::Timestamp(TimestampValue::Unix(1_700_000_000))),
        ("--format", FlagValue::Dropdown("bitcoin-core".into())),
    ]);
    let argv = assemble_argv(
        &schema::mnemonic::SCHEMA,
        subcommand("export-wallet"),
        &state,
    );
    // Schema declares: --template, ..., --format, --output, --range,
    // --timestamp, ... — so --format precedes --range / --timestamp per
    // SPEC §6.3.
    assert_eq!(
        argv,
        vec![
            "mnemonic",
            "export-wallet",
            "--template",
            "bip84",
            "--format",
            "bitcoin-core",
            "--range",
            "0,999",
            "--timestamp",
            "1700000000",
        ]
    );
}

#[test]
fn cell_3b_export_wallet_timestamp_now_argv() {
    // Targeted: Timestamp::Now → literal "now" token.
    let state = FormState::from_pairs(vec![(
        "--timestamp",
        FlagValue::Timestamp(TimestampValue::Now),
    )]);
    let argv = assemble_argv(
        &schema::mnemonic::SCHEMA,
        subcommand("export-wallet"),
        &state,
    );
    assert_eq!(argv, vec!["mnemonic", "export-wallet", "--timestamp", "now"]);
}

#[test]
fn cell_4_export_wallet_tr_multi_a_argv() {
    // Covers TaggedOrIndexed (Tag arm + Indexed arm in two cells).
    let tag_state = FormState::from_pairs(vec![(
        "--taproot-internal-key",
        FlagValue::TaggedOrIndexed(TaggedOrIndexedValue::Tag("nums".into())),
    )]);
    let tag_argv = assemble_argv(
        &schema::mnemonic::SCHEMA,
        subcommand("export-wallet"),
        &tag_state,
    );
    assert_eq!(
        tag_argv,
        vec!["mnemonic", "export-wallet", "--taproot-internal-key", "nums"]
    );

    let indexed_state = FormState::from_pairs(vec![(
        "--taproot-internal-key",
        FlagValue::TaggedOrIndexed(TaggedOrIndexedValue::Indexed(2)),
    )]);
    let indexed_argv = assemble_argv(
        &schema::mnemonic::SCHEMA,
        subcommand("export-wallet"),
        &indexed_state,
    );
    assert_eq!(
        indexed_argv,
        vec!["mnemonic", "export-wallet", "--taproot-internal-key", "@2"]
    );
}

#[test]
fn cell_5_bundle_descriptor_file_argv() {
    // Covers Path { stdio_sentinel: false } — regular path emission.
    let state = FormState::from_pairs(vec![
        ("--network", FlagValue::Dropdown("mainnet".into())),
        (
            "--descriptor-file",
            FlagValue::Path("/tmp/my-descriptor.txt".into()),
        ),
    ]);
    let argv = assemble_argv(&schema::mnemonic::SCHEMA, subcommand("bundle"), &state);
    assert_eq!(
        argv,
        vec![
            "mnemonic",
            "bundle",
            "--network",
            "mainnet",
            "--descriptor-file",
            "/tmp/my-descriptor.txt",
        ]
    );
}

#[test]
fn cell_5b_export_wallet_output_stdio_sentinel_argv() {
    // Covers Path { stdio_sentinel: true } — `-` is a real argv token.
    let state = FormState::from_pairs(vec![("--output", FlagValue::Path("-".into()))]);
    let argv = assemble_argv(
        &schema::mnemonic::SCHEMA,
        subcommand("export-wallet"),
        &state,
    );
    assert_eq!(
        argv,
        vec!["mnemonic", "export-wallet", "--output", "-"]
    );
}

#[test]
fn cell_6_node_value_composite_empty_value_omitted() {
    // SPEC §6.7 R3 I-3 fold: empty value → omit (matches upstream
    // `parse_from_input` rejection of empty-after-`=`).
    let state = FormState::from_pairs(vec![(
        "--from",
        FlagValue::NodeValueComposite {
            node: "phrase".into(),
            value: "".into(),
        },
    )]);
    let argv = assemble_argv(&schema::mnemonic::SCHEMA, subcommand("convert"), &state);
    assert_eq!(argv, vec!["mnemonic", "convert"]); // bare argv, no --from
}

#[test]
fn cell_7_emission_order_follows_schema_declaration() {
    // SPEC §6.3 + R1 L-1: argv flag order matches the schema's declared
    // flag order, NOT the form-state insertion order.
    let state = FormState::from_pairs(vec![
        // Insert in REVERSE schema order.
        ("--account", FlagValue::Number(0)),
        ("--passphrase", FlagValue::Text("p".into())),
        ("--template", FlagValue::Dropdown("bip84".into())),
        ("--network", FlagValue::Dropdown("signet".into())),
    ]);
    let argv = assemble_argv(&schema::mnemonic::SCHEMA, subcommand("bundle"), &state);
    assert_eq!(
        argv,
        vec![
            "mnemonic", "bundle",
            // Schema order: --network, --template, ..., --passphrase, --account
            "--network", "signet",
            "--template", "bip84",
            "--passphrase", "p",
            "--account", "0",
        ]
    );
}

#[test]
fn cell_8_argv_zero_is_unqualified_binary_name() {
    // SPEC §6.1: argv[0] is the CLI binary name with NO absolute path.
    let state = FormState::default();
    let argv = assemble_argv(
        &schema::mnemonic::SCHEMA,
        subcommand("derive-child"),
        &state,
    );
    assert_eq!(argv[0], "mnemonic"); // not /usr/local/bin/mnemonic
    assert_eq!(argv[1], "derive-child");
    assert_eq!(argv.len(), 2); // no required flags populated → bare subcommand
}
