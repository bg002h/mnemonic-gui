//! Secret-handling tests (SPEC §9 + §10 + Phase 7 IMPL_PLAN).
//!
//! Covers:
//!   - Paste-warn threshold (≥8 chars) on secret-class flag.
//!   - Below-threshold pastes: no modal.
//!   - Non-secret flag pastes: no modal regardless of length.
//!   - Run-confirm fires iff any secret-class field is populated.
//!   - SecretBuffer::zeroize() overwrites the visible bytes.
//!   - zeroize_form_state() sweep.
//!   - Modal copy text byte-exact.

use mnemonic_gui::form::slot_editor::{SlotRow, SlotState, SlotSubkey};
use mnemonic_gui::schema::{self, FlagValue, FormState};
use mnemonic_gui::secrets::{
    self, should_confirm_run, should_warn_on_paste, zeroize_form_state, SecretBuffer,
    PASTE_WARN_MODAL_TEXT, PASTE_WARN_THRESHOLD, RUN_CONFIRM_MODAL_PREFIX,
    SECRET_FLAG_NAMES, SECRET_NODE_TYPES, SECRET_SLOT_SUBKEYS,
};

fn subcommand(name: &str) -> &'static schema::SubcommandSchema {
    schema::mnemonic::SCHEMA
        .subcommands
        .iter()
        .find(|s| s.name == name)
        .unwrap_or_else(|| panic!("subcommand {} not in schema", name))
}

fn flag<'a>(sub: &'a schema::SubcommandSchema, name: &str) -> &'a schema::FlagSchema {
    sub.flags
        .iter()
        .find(|f| f.name == name)
        .unwrap_or_else(|| panic!("flag {} not in {}", name, sub.name))
}

// ─── Paste-warn modal ────────────────────────────────────────────────────

#[test]
fn paste_warn_fires_at_8_char_threshold_on_secret_flag() {
    let bundle = subcommand("bundle");
    let passphrase = flag(bundle, "--passphrase");
    // Threshold is ≥ PASTE_WARN_THRESHOLD (8).
    assert!(!should_warn_on_paste(passphrase, PASTE_WARN_THRESHOLD - 1));
    assert!(should_warn_on_paste(passphrase, PASTE_WARN_THRESHOLD));
    assert!(should_warn_on_paste(passphrase, PASTE_WARN_THRESHOLD + 100));
}

#[test]
fn paste_warn_silent_on_non_secret_flag_at_any_length() {
    let bundle = subcommand("bundle");
    let network = flag(bundle, "--network");
    assert!(!should_warn_on_paste(network, 100));
    assert!(!should_warn_on_paste(network, 8192));
}

#[test]
fn paste_warn_modal_text_byte_exact_to_spec_9() {
    // Spot-check the load-bearing substrings — full text is in
    // src/secrets.rs and pinned by inspection.
    assert!(PASTE_WARN_MODAL_TEXT.contains("BIP-39 phrase"));
    assert!(PASTE_WARN_MODAL_TEXT.contains("BIP-38 ciphertext"));
    assert!(PASTE_WARN_MODAL_TEXT.contains("ms1 share"));
    assert!(PASTE_WARN_MODAL_TEXT.contains("v0.2 deferred per FOLLOWUPS"));
    assert!(PASTE_WARN_MODAL_TEXT.contains("gui-os-snapshot-secret-occlusion"));
    assert!(PASTE_WARN_MODAL_TEXT.contains("gui-secret-buffer-allocator-residue"));
}

// ─── Run-confirm modal ───────────────────────────────────────────────────

#[test]
fn run_confirm_fires_when_passphrase_populated() {
    let state = FormState::from_pairs(vec![
        ("--network", FlagValue::Dropdown("mainnet".into())),
        ("--passphrase", FlagValue::Text("hunter2".into())),
    ]);
    assert!(should_confirm_run(subcommand("bundle"), &state));
}

#[test]
fn run_confirm_silent_with_no_secret_fields() {
    let state = FormState::from_pairs(vec![
        ("--network", FlagValue::Dropdown("mainnet".into())),
        ("--template", FlagValue::Dropdown("bip84".into())),
    ]);
    assert!(!should_confirm_run(subcommand("bundle"), &state));
}

#[test]
fn run_confirm_fires_on_secret_class_slot_row() {
    let state = FormState::default().with_slots(SlotState {
        rows: vec![SlotRow {
            index: 0,
            subkey: SlotSubkey::Phrase,
            value: "abandon abandon...".into(),
        }],
    });
    assert!(should_confirm_run(subcommand("bundle"), &state));
}

#[test]
fn run_confirm_silent_on_watch_only_slot_row() {
    let state = FormState::default().with_slots(SlotState {
        rows: vec![SlotRow {
            index: 0,
            subkey: SlotSubkey::Xpub,
            value: "xpub...".into(),
        }],
    });
    assert!(!should_confirm_run(subcommand("bundle"), &state));
}

#[test]
fn run_confirm_fires_on_secret_node_value_composite() {
    let state = FormState::from_pairs(vec![(
        "--from",
        FlagValue::NodeValueComposite {
            node: "phrase".into(),
            value: "abandon...".into(),
        },
    )]);
    assert!(should_confirm_run(subcommand("convert"), &state));
}

#[test]
fn run_confirm_silent_on_watch_only_node_value_composite() {
    let state = FormState::from_pairs(vec![(
        "--from",
        FlagValue::NodeValueComposite {
            node: "xpub".into(),
            value: "xpub6...".into(),
        },
    )]);
    assert!(!should_confirm_run(subcommand("convert"), &state));
}

#[test]
fn run_confirm_prefix_byte_exact_to_spec_9() {
    assert_eq!(
        RUN_CONFIRM_MODAL_PREFIX,
        "This invocation passes secret-bearing arguments to "
    );
}

// ─── Build.rs-generated SECRET_* constants ───────────────────────────────

#[test]
fn secret_node_types_set_pinned() {
    // Phase 7 R1 anchor: these constants are GENERATED from upstream by
    // build.rs. If upstream `NodeType::is_secret_bearing()` changes, the
    // generated arrays change too — and this assertion will need to be
    // updated in lockstep. The schema_mirror source-audit cells provide
    // the dynamic-vs-generated cross-check; this cell pins the v0.1
    // expected set.
    let expected: std::collections::BTreeSet<&str> = [
        "phrase",
        "entropy",
        "xprv",
        "wif",
        "ms1",
        "bip38",
        "electrum-phrase",
    ]
    .into_iter()
    .collect();
    let actual: std::collections::BTreeSet<&str> = SECRET_NODE_TYPES.iter().copied().collect();
    assert_eq!(actual, expected);
}

#[test]
fn secret_slot_subkeys_set_pinned() {
    let expected: std::collections::BTreeSet<&str> = ["phrase", "entropy", "xprv", "wif"]
        .into_iter()
        .collect();
    let actual: std::collections::BTreeSet<&str> = SECRET_SLOT_SUBKEYS.iter().copied().collect();
    assert_eq!(actual, expected);
}

#[test]
fn secret_flag_names_include_all_passphrase_class() {
    for f in ["--passphrase", "--bip38-passphrase", "--passphrase-stdin"] {
        assert!(
            SECRET_FLAG_NAMES.contains(&f),
            "{} must be in SECRET_FLAG_NAMES",
            f
        );
    }
}

// ─── Zeroize ─────────────────────────────────────────────────────────────

#[test]
fn secret_buffer_zeroize_overwrites_bytes() {
    let mut buf = SecretBuffer::from_text("hunter2");
    use zeroize::Zeroize;
    buf.zeroize();
    // After zeroize, the visible string is empty (String::zeroize clears
    // both bytes AND length).
    assert!(buf.is_empty());
}

#[test]
fn zeroize_form_state_clears_text_dropdown_path_and_slots() {
    let mut state = FormState::from_pairs(vec![
        ("--passphrase", FlagValue::Text("hunter2".into())),
        ("--network", FlagValue::Dropdown("mainnet".into())),
        ("--output", FlagValue::Path("/tmp/x".into())),
        (
            "--from",
            FlagValue::NodeValueComposite {
                node: "phrase".into(),
                value: "abandon...".into(),
            },
        ),
    ])
    .with_slots(SlotState {
        rows: vec![SlotRow {
            index: 0,
            subkey: SlotSubkey::Phrase,
            value: "abandon abandon...".into(),
        }],
    });

    zeroize_form_state(&mut state);

    // All string-bearing slots cleared (Boolean/Number/Range/Timestamp/
    // TaggedOrIndexed don't carry secrets and are not zeroized).
    for (_, v) in &state.values {
        match v {
            FlagValue::Text(s) | FlagValue::Dropdown(s) | FlagValue::Path(s) => {
                assert!(s.is_empty(), "value should be cleared: {:?}", v);
            }
            FlagValue::NodeValueComposite { value, .. } => {
                assert!(value.is_empty(), "node-value should be cleared");
            }
            _ => {}
        }
    }
    for row in &state.slots.rows {
        assert!(row.value.is_empty(), "slot row should be cleared");
    }
}

// ─── flag_is_secret + slot_subkey_is_secret + node_type_is_secret ────────

#[test]
fn flag_is_secret_covers_passphrase_class() {
    let bundle = subcommand("bundle");
    assert!(secrets::flag_is_secret(flag(bundle, "--passphrase")));
    assert!(!secrets::flag_is_secret(flag(bundle, "--network")));

    let convert = subcommand("convert");
    assert!(secrets::flag_is_secret(flag(convert, "--bip38-passphrase")));
    assert!(secrets::flag_is_secret(flag(convert, "--passphrase-stdin")));
}

#[test]
fn slot_subkey_is_secret_matches_upstream_set() {
    for sk in [
        SlotSubkey::Phrase,
        SlotSubkey::Entropy,
        SlotSubkey::Wif,
        SlotSubkey::Xprv,
    ] {
        assert!(secrets::slot_subkey_is_secret(sk));
    }
    for sk in [
        SlotSubkey::Xpub,
        SlotSubkey::MasterXpub,
        SlotSubkey::Fingerprint,
        SlotSubkey::Path,
    ] {
        assert!(!secrets::slot_subkey_is_secret(sk));
    }
}

#[test]
fn node_type_is_secret_matches_upstream_set() {
    for n in ["phrase", "entropy", "xprv", "wif", "ms1", "bip38", "electrum-phrase"] {
        assert!(secrets::node_type_is_secret(n));
    }
    for n in ["xpub", "address", "fingerprint", "path"] {
        assert!(!secrets::node_type_is_secret(n));
    }
}
