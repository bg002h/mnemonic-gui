//! Byte-exact argv-assembly tests (SPEC §6 + §6.7).
//!
//! One cell per `FlagKind` variant against the `mnemonic` schema. Slot
//! tests live in `tests/argv_assembler_slot.rs` (Phase 3).

use mnemonic_gui::form::invocation::assemble_argv;
use mnemonic_gui::form::secret_widget::SecretLineEdit;
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
    //
    // v0.2 Phase B.1: secret-class flags (e.g. --passphrase) route
    // through state.secret_widgets, NOT state.values, per SPEC §3
    // assemble_argv branch.
    //
    // v0.10.0 B.3 (D33): `--account` with value `0` is suppressed by the
    // default-suppression predicate (the toolkit v5 schema declares
    // `default_value: 0`); the toolkit picks up the same value from its
    // own defaults. To force emission, use a non-default value (`1` etc).
    let mut state = FormState::from_pairs(vec![
        ("--network", FlagValue::Dropdown("mainnet".into())),
        ("--template", FlagValue::Dropdown("bip84".into())),
        ("--account", FlagValue::Number(0)),
        ("--json", FlagValue::Boolean(true)),
        ("--privacy-preserving", FlagValue::Boolean(false)), // omit
    ]);
    state
        .secret_widgets
        .insert("--passphrase".into(), SecretLineEdit::from_text("hunter2"));
    let argv = assemble_argv(&schema::mnemonic::SCHEMA, subcommand("bundle"), &state);
    assert_eq!(
        argv,
        vec![
            "mnemonic", "bundle", "--network", "mainnet", "--template", "bip84",
            "--passphrase", "hunter2", "--json",
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
    //
    // v0.10.0 B.3 (D33): `--range 0,999` and `--format bitcoin-core` are
    // suppressed by the default-suppression predicate (toolkit v5 schema
    // declares those as the defaults). `--timestamp 1700000000` is an
    // Epoch value; the `is_at_default` Unix arm is always `false`, so it
    // emits regardless of the schema default (which is `0` since v0.28.0 /
    // toolkit v0.47.3).
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
    // SPEC §6.3. Default-suppression drops --range + --format.
    assert_eq!(
        argv,
        vec![
            "mnemonic",
            "export-wallet",
            "--template",
            "bip84",
            "--timestamp",
            "1700000000",
        ]
    );
}

#[test]
fn cell_3b_export_wallet_timestamp_now_argv() {
    // Targeted: Timestamp::Now → literal "now" token.
    //
    // v0.28.0 (toolkit v0.47.3): the export-wallet --timestamp schema default
    // is now "0" (genesis rescan), so an EXPLICIT Timestamp::Now is no longer
    // at-default and MUST emit `--timestamp now`. (Pre-v0.28.0 the schema
    // default was "now" and D33 suppressed it — which silently discarded the
    // user's explicit `now` once the toolkit default flipped to 0.)
    let state = FormState::from_pairs(vec![(
        "--timestamp",
        FlagValue::Timestamp(TimestampValue::Now),
    )]);
    let argv = assemble_argv(
        &schema::mnemonic::SCHEMA,
        subcommand("export-wallet"),
        &state,
    );
    assert_eq!(
        argv,
        vec!["mnemonic", "export-wallet", "--timestamp", "now"]
    );
}

#[test]
fn cell_4_export_wallet_tr_multi_a_argv() {
    // Covers TaggedOrIndexed (Tag arm + Indexed arm in two cells).
    // v0.16.0 SPEC §6.10.7: --taproot-internal-key is gated by template ∈
    // {tr-multi-a, tr-sortedmulti-a}; without a matching --template, the
    // new P3 visibility gate suppresses emission. The test now supplies
    // --template = tr-multi-a explicitly (matches the cell name).
    let tag_state = FormState::from_pairs(vec![
        ("--template", FlagValue::Dropdown("tr-multi-a".into())),
        (
            "--taproot-internal-key",
            FlagValue::TaggedOrIndexed(TaggedOrIndexedValue::Tag("nums".into())),
        ),
    ]);
    let tag_argv = assemble_argv(
        &schema::mnemonic::SCHEMA,
        subcommand("export-wallet"),
        &tag_state,
    );
    // Emission order = schema declaration order per SPEC §6 invariant 3.
    // --template precedes --taproot-internal-key in the schema.
    assert_eq!(
        tag_argv,
        vec![
            "mnemonic",
            "export-wallet",
            "--template",
            "tr-multi-a",
            "--taproot-internal-key",
            "nums",
        ]
    );

    let indexed_state = FormState::from_pairs(vec![
        ("--template", FlagValue::Dropdown("tr-sortedmulti-a".into())),
        (
            "--taproot-internal-key",
            FlagValue::TaggedOrIndexed(TaggedOrIndexedValue::Indexed(2)),
        ),
    ]);
    let indexed_argv = assemble_argv(
        &schema::mnemonic::SCHEMA,
        subcommand("export-wallet"),
        &indexed_state,
    );
    assert_eq!(
        indexed_argv,
        vec![
            "mnemonic",
            "export-wallet",
            "--template",
            "tr-sortedmulti-a",
            "--taproot-internal-key",
            "@2",
        ]
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
    //
    // v0.10.0 B.3 (D33): `--output -` is the toolkit v5 default;
    // default-suppression drops it from argv. To force emission, supply a
    // non-default path (`/tmp/foo` etc). Emission behavior for non-default
    // paths is covered by `cell_5_bundle_descriptor_file_argv`.
    let state = FormState::from_pairs(vec![("--output", FlagValue::Path("-".into()))]);
    let argv = assemble_argv(
        &schema::mnemonic::SCHEMA,
        subcommand("export-wallet"),
        &state,
    );
    assert_eq!(argv, vec!["mnemonic", "export-wallet"]);
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
    //
    // v0.2 Phase B.1: --passphrase lives in state.secret_widgets, not
    // state.values; the emission order still follows the schema.
    //
    // v0.10.0 B.3 (D33): `--account 0` is the schema default — suppressed.
    // Bump to `1` to force emission and still exercise the reverse-insert
    // → schema-order assertion.
    let mut state = FormState::from_pairs(vec![
        // Insert in REVERSE schema order.
        ("--account", FlagValue::Number(1)),
        ("--template", FlagValue::Dropdown("bip84".into())),
        ("--network", FlagValue::Dropdown("signet".into())),
    ]);
    state
        .secret_widgets
        .insert("--passphrase".into(), SecretLineEdit::from_text("p"));
    let argv = assemble_argv(&schema::mnemonic::SCHEMA, subcommand("bundle"), &state);
    assert_eq!(
        argv,
        vec![
            "mnemonic", "bundle",
            // Schema order: --network, --template, ..., --passphrase, --account
            "--network", "signet",
            "--template", "bip84",
            "--passphrase", "p",
            "--account", "1",
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

// ─── v0.2 Phase B.1: secret-flag emission routes through secret_widgets ───

#[test]
fn secret_class_flag_emitted_from_secret_widget_not_values_map() {
    // SPEC §3 (v0.2 B.1): the assemble_argv secret-flag branch reads
    // ONLY from state.secret_widgets for secret-class flags; state.values
    // is NOT consulted for them.
    let mut state = FormState::default();
    // Populate the secret-widget bucket with --passphrase.
    state
        .secret_widgets
        .insert("--passphrase".into(), SecretLineEdit::from_text("alpha"));
    // Populate state.values for a non-secret flag (--account) and also
    // intentionally for --passphrase (this entry must be IGNORED by the
    // secret-flag branch).
    state.values.push((
        "--passphrase".to_string(),
        FlagValue::Text("BOGUS — should not appear in argv".into()),
    ));
    state.values.push(("--account".to_string(), FlagValue::Number(7)));

    let argv = assemble_argv(&schema::mnemonic::SCHEMA, subcommand("bundle"), &state);

    // The widget's "alpha" must appear; the BOGUS state.values entry must NOT.
    let joined = argv.join(" ");
    assert!(
        joined.contains("--passphrase alpha"),
        "--passphrase value must come from secret_widgets: {}",
        joined
    );
    assert!(
        !joined.contains("BOGUS"),
        "state.values --passphrase entry must be ignored: {}",
        joined
    );
    // --account must still emit from state.values (non-secret path).
    assert!(joined.contains("--account 7"));
}

// ─── v0.10.0 B.3 (D33) default-suppression cells ─────────────────────────
//
// D33 contract: when a form-state value equals the schema-declared
// `default_value` for a flag, the argv assembler suppresses the flag from
// emission. The toolkit picks up the same value from its own clap-derive
// default at parse time, so explicit emission is noise. Per-FlagKind
// compare-predicate table in `src/form/invocation.rs::is_at_default`.
//
// Coverage strategy: one cell per FlagKind variant exercising the
// suppression (default-equal value suppressed) AND a paired negative-
// control cell asserting non-default values still emit normally.
//
// NodeValueComposite + Boolean trivially-suppressed cases are covered
// by pre-existing cells (`cell_6_node_value_composite_empty_value_omitted`
// + the Boolean(false) omission in `cell_1`).

mod d33_default_suppression {
    use super::*;

    #[test]
    fn d33_number_at_default_suppresses_account_zero() {
        // bundle's --account schema default is "0" (toolkit v5 schema).
        // Number(0) → suppressed.
        let state = FormState::from_pairs(vec![
            ("--network", FlagValue::Dropdown("mainnet".into())),
            ("--account", FlagValue::Number(0)),
        ]);
        let argv =
            assemble_argv(&schema::mnemonic::SCHEMA, subcommand("bundle"), &state);
        assert!(
            !argv.iter().any(|a| a == "--account"),
            "--account 0 (schema default) must be suppressed; got {argv:?}"
        );
        assert!(
            !argv.iter().any(|a| a == "0"),
            "the default value 0 must NOT leak as a positional; got {argv:?}"
        );
    }

    #[test]
    fn d33_number_non_default_emits_account_nonzero() {
        // Negative control: --account 1 is NOT the default → emits.
        let state = FormState::from_pairs(vec![
            ("--network", FlagValue::Dropdown("mainnet".into())),
            ("--account", FlagValue::Number(1)),
        ]);
        let argv =
            assemble_argv(&schema::mnemonic::SCHEMA, subcommand("bundle"), &state);
        assert!(
            argv.iter().zip(argv.iter().skip(1)).any(|(a, b)| a == "--account" && b == "1"),
            "--account 1 (non-default) must emit; got {argv:?}"
        );
    }

    #[test]
    fn d33_text_at_default_suppresses() {
        // No --network default exists for export-wallet (schema's default
        // is "mainnet" Dropdown). Text-default coverage uses Path below.
        // Empty text is already trivially suppressed by emit_one.
        // Synthetic case: bundle's --passphrase is Text-secret; secrets
        // bypass emit_one's is_at_default. Use the export-wallet --network
        // (Dropdown) below for the at-default path; here we anchor the
        // empty-text already-suppressed path as the D33 trivial baseline.
        let state = FormState::from_pairs(vec![(
            "--descriptor",
            FlagValue::Text("".into()),
        )]);
        let argv =
            assemble_argv(&schema::mnemonic::SCHEMA, subcommand("bundle"), &state);
        assert!(
            !argv.iter().any(|a| a == "--descriptor"),
            "empty Text already suppressed (pre-D33 baseline); got {argv:?}"
        );
    }

    #[test]
    fn d33_path_at_default_suppresses_output_dash() {
        // export-wallet --output schema default is "-" (toolkit v5).
        // Path("-") → suppressed.
        let state = FormState::from_pairs(vec![("--output", FlagValue::Path("-".into()))]);
        let argv = assemble_argv(
            &schema::mnemonic::SCHEMA,
            subcommand("export-wallet"),
            &state,
        );
        assert!(
            !argv.iter().any(|a| a == "--output"),
            "--output - (schema default) must be suppressed; got {argv:?}"
        );
    }

    #[test]
    fn d33_path_non_default_emits() {
        // Negative control: --output /tmp/foo is NOT the default → emits.
        let state = FormState::from_pairs(vec![(
            "--output",
            FlagValue::Path("/tmp/foo.json".into()),
        )]);
        let argv = assemble_argv(
            &schema::mnemonic::SCHEMA,
            subcommand("export-wallet"),
            &state,
        );
        assert!(
            argv.iter().any(|a| a == "/tmp/foo.json"),
            "--output /tmp/foo.json (non-default) must emit; got {argv:?}"
        );
    }

    #[test]
    fn d33_range_at_default_suppresses_zero_999() {
        // export-wallet --range schema default is "0,999" (toolkit v5).
        // Range(0, 999) → suppressed.
        let state = FormState::from_pairs(vec![("--range", FlagValue::Range(0, 999))]);
        let argv = assemble_argv(
            &schema::mnemonic::SCHEMA,
            subcommand("export-wallet"),
            &state,
        );
        assert!(
            !argv.iter().any(|a| a == "--range"),
            "--range 0,999 (schema default) must be suppressed; got {argv:?}"
        );
    }

    #[test]
    fn d33_range_non_default_emits() {
        // Negative control: --range 0,500 is NOT the default → emits.
        let state = FormState::from_pairs(vec![("--range", FlagValue::Range(0, 500))]);
        let argv = assemble_argv(
            &schema::mnemonic::SCHEMA,
            subcommand("export-wallet"),
            &state,
        );
        assert!(
            argv.iter().any(|a| a == "0,500"),
            "--range 0,500 (non-default) must emit; got {argv:?}"
        );
    }

    #[test]
    fn d33_timestamp_now_is_emitted_when_default_is_zero() {
        // v0.28.0 (toolkit v0.47.3): export-wallet --timestamp schema default
        // is "0", so an explicit Timestamp::Now is NOT at-default and MUST be
        // emitted. Regression guard for the D33 default-value-drift fix
        // (gui-timestamp-default-value-drift-v0.47.3): an explicit `now`
        // selection must reach the toolkit, not be silently dropped to the
        // toolkit's new `0` default.
        let state = FormState::from_pairs(vec![(
            "--timestamp",
            FlagValue::Timestamp(TimestampValue::Now),
        )]);
        let argv = assemble_argv(
            &schema::mnemonic::SCHEMA,
            subcommand("export-wallet"),
            &state,
        );
        assert!(
            argv.windows(2).any(|w| w == ["--timestamp", "now"]),
            "explicit --timestamp now must be EMITTED when the default is 0; got {argv:?}"
        );
    }

    #[test]
    fn d33_timestamp_epoch_never_matches_now_default() {
        // D33: "Timestamp(Epoch(n)) never matches `now` default; epoch
        // values always emit."
        let state = FormState::from_pairs(vec![(
            "--timestamp",
            FlagValue::Timestamp(TimestampValue::Unix(0)),
        )]);
        let argv = assemble_argv(
            &schema::mnemonic::SCHEMA,
            subcommand("export-wallet"),
            &state,
        );
        assert!(
            argv.iter().any(|a| a == "--timestamp"),
            "Epoch Timestamp(0) must emit (D33: epoch never matches `now`); got {argv:?}"
        );
        assert!(
            argv.iter().any(|a| a == "0"),
            "epoch value 0 must appear as the token; got {argv:?}"
        );
    }

    #[test]
    fn d33_dropdown_at_default_suppresses_format_bitcoin_core() {
        // export-wallet --format schema default is "bitcoin-core" (v5).
        let state = FormState::from_pairs(vec![(
            "--format",
            FlagValue::Dropdown("bitcoin-core".into()),
        )]);
        let argv = assemble_argv(
            &schema::mnemonic::SCHEMA,
            subcommand("export-wallet"),
            &state,
        );
        assert!(
            !argv.iter().any(|a| a == "--format"),
            "--format bitcoin-core (schema default) must be suppressed; got {argv:?}"
        );
    }

    #[test]
    fn d33_dropdown_non_default_emits() {
        // Negative control: --format coldcard is NOT the default → emits.
        let state = FormState::from_pairs(vec![(
            "--format",
            FlagValue::Dropdown("coldcard".into()),
        )]);
        let argv = assemble_argv(
            &schema::mnemonic::SCHEMA,
            subcommand("export-wallet"),
            &state,
        );
        assert!(
            argv.iter().zip(argv.iter().skip(1)).any(|(a, b)| a == "--format" && b == "coldcard"),
            "--format coldcard (non-default) must emit; got {argv:?}"
        );
    }

    #[test]
    fn d33_no_default_value_always_emits() {
        // For flags without a schema-declared default (most flags),
        // `is_at_default` returns false → emit normally. Anchors the
        // "fall-through" path of the predicate.
        let state = FormState::from_pairs(vec![(
            "--threshold",
            FlagValue::Number(2),
        )]);
        let argv = assemble_argv(
            &schema::mnemonic::SCHEMA,
            subcommand("export-wallet"),
            &state,
        );
        // --threshold has no schema default → emits.
        assert!(
            argv.iter().any(|a| a == "--threshold"),
            "flags without default_value must always emit when Set; got {argv:?}"
        );
    }
}
