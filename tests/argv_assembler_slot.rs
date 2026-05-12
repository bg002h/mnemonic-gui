//! SlotEditor argv-emission tests (SPEC §B.4 + §6.4 + Phase 3 IMPL_PLAN).
//!
//! Three cells per the IMPL_PLAN:
//!   1. 3-slot phrase bundle — mixed subkeys, all three rows present in
//!      the emitted argv at the `--slot` position, in slot-index order.
//!   2. Slot-index sort under reverse-insertion form state — @2 added
//!      before @0 in state.slots, argv emission is @0, @1, @2.
//!   3. Secret-class exclusion from persistable rows.

use mnemonic_gui::form::invocation::assemble_argv;
use mnemonic_gui::form::slot_editor::{SlotRow, SlotState, SlotSubkey};
use mnemonic_gui::schema::{self, FlagValue, FormState};

fn subcommand(name: &str) -> &'static schema::SubcommandSchema {
    schema::mnemonic::SCHEMA
        .subcommands
        .iter()
        .find(|s| s.name == name)
        .unwrap_or_else(|| panic!("subcommand {} not in schema", name))
}

#[test]
fn cell_1_bundle_3_slot_phrase_argv() {
    let slots = SlotState {
        rows: vec![
            SlotRow {
                index: 0,
                subkey: SlotSubkey::Phrase,
                value: "abandon abandon abandon".into(),
            },
            SlotRow {
                index: 1,
                subkey: SlotSubkey::Xpub,
                value: "xpub6BosfCnifzxcF...".into(),
            },
            SlotRow {
                index: 2,
                subkey: SlotSubkey::Fingerprint,
                value: "00000000".into(),
            },
        ],
    };
    let state = FormState::from_pairs(vec![
        ("--network", FlagValue::Dropdown("mainnet".into())),
        ("--template", FlagValue::Dropdown("wsh-sortedmulti".into())),
        ("--threshold", FlagValue::Number(2)),
    ])
    .with_slots(slots);
    let argv = assemble_argv(&schema::mnemonic::SCHEMA, subcommand("bundle"), &state);
    assert_eq!(
        argv,
        vec![
            "mnemonic",
            "bundle",
            "--network",
            "mainnet",
            "--template",
            "wsh-sortedmulti",
            "--threshold",
            "2",
            "--slot",
            "@0.phrase=abandon abandon abandon",
            "--slot",
            "@1.xpub=xpub6BosfCnifzxcF...",
            "--slot",
            "@2.fingerprint=00000000",
        ]
    );
}

#[test]
fn cell_2_verify_bundle_slot_index_sort_argv() {
    // Insert in REVERSE index order (@2, @0, @1); emitter sorts by index
    // ascending per SPEC §6.4 + cmd::bundle::resolve_slots BTreeMap parity.
    let slots = SlotState {
        rows: vec![
            SlotRow {
                index: 2,
                subkey: SlotSubkey::Xpub,
                value: "xpub_two".into(),
            },
            SlotRow {
                index: 0,
                subkey: SlotSubkey::Xpub,
                value: "xpub_zero".into(),
            },
            SlotRow {
                index: 1,
                subkey: SlotSubkey::Xpub,
                value: "xpub_one".into(),
            },
        ],
    };
    let state = FormState::from_pairs(vec![(
        "--network",
        FlagValue::Dropdown("mainnet".into()),
    )])
    .with_slots(slots);
    let argv = assemble_argv(
        &schema::mnemonic::SCHEMA,
        subcommand("verify-bundle"),
        &state,
    );
    // Slot pairs must be in @0, @1, @2 order regardless of insertion.
    let slot_pairs: Vec<&str> = argv
        .iter()
        .filter(|s| s.starts_with("@"))
        .map(String::as_str)
        .collect();
    assert_eq!(
        slot_pairs,
        vec![
            "@0.xpub=xpub_zero",
            "@1.xpub=xpub_one",
            "@2.xpub=xpub_two",
        ]
    );
}

#[test]
fn cell_3_export_wallet_persist_secret_exclusion() {
    // SPEC §10 R1 I-4: never-persist set = SlotSubkey::is_secret_bearing()
    // true variants (Phrase, Entropy, Wif, Xprv). Watch-only subkeys
    // (Xpub, MasterXpub, Fingerprint, Path) ARE persistable.
    let slots = SlotState {
        rows: vec![
            // Secret-class rows that must be excluded:
            SlotRow {
                index: 0,
                subkey: SlotSubkey::Phrase,
                value: "abandon...".into(),
            },
            SlotRow {
                index: 1,
                subkey: SlotSubkey::Entropy,
                value: "deadbeef".into(),
            },
            SlotRow {
                index: 2,
                subkey: SlotSubkey::Wif,
                value: "Kxxxxx...".into(),
            },
            SlotRow {
                index: 3,
                subkey: SlotSubkey::Xprv,
                value: "xprv...".into(),
            },
            // Watch-only rows that MUST persist:
            SlotRow {
                index: 4,
                subkey: SlotSubkey::Xpub,
                value: "xpub...".into(),
            },
            SlotRow {
                index: 5,
                subkey: SlotSubkey::MasterXpub,
                value: "xpub_master...".into(),
            },
            SlotRow {
                index: 6,
                subkey: SlotSubkey::Fingerprint,
                value: "00000000".into(),
            },
            SlotRow {
                index: 7,
                subkey: SlotSubkey::Path,
                value: "84'/0'/0'".into(),
            },
        ],
    };

    let persistable_subkeys: Vec<SlotSubkey> =
        slots.persistable_rows().map(|r| r.subkey).collect();

    // Exactly the 4 watch-only subkeys must remain.
    assert_eq!(
        persistable_subkeys,
        vec![
            SlotSubkey::Xpub,
            SlotSubkey::MasterXpub,
            SlotSubkey::Fingerprint,
            SlotSubkey::Path,
        ]
    );

    // None of the secret-class subkeys appear in the persistable set.
    for sk in [
        SlotSubkey::Phrase,
        SlotSubkey::Entropy,
        SlotSubkey::Wif,
        SlotSubkey::Xprv,
    ] {
        assert!(sk.is_secret_bearing(), "expected {:?} secret-bearing", sk);
        assert!(
            !persistable_subkeys.contains(&sk),
            "secret-class {:?} leaked into persistable set",
            sk
        );
    }

    // Cross-check: each is_secret_bearing() variant is also not in the
    // is-watch-only set. (R3 I-3 fold parity with upstream.)
    for sk in SlotSubkey::ALL {
        let secret = sk.is_secret_bearing();
        let persistable = !secret;
        assert_eq!(
            persistable,
            persistable_subkeys.contains(sk),
            "subkey {:?} persistability mismatch",
            sk
        );
    }
}

#[test]
fn cell_4_empty_slot_value_omitted() {
    // SPEC §6.7 empty-value omission rule applies to slot rows too.
    let slots = SlotState {
        rows: vec![
            SlotRow {
                index: 0,
                subkey: SlotSubkey::Xpub,
                value: "xpub_zero".into(),
            },
            SlotRow {
                index: 1,
                subkey: SlotSubkey::Xpub,
                value: "".into(), // empty — must omit
            },
            SlotRow {
                index: 2,
                subkey: SlotSubkey::Xpub,
                value: "xpub_two".into(),
            },
        ],
    };
    let state = FormState::from_pairs(vec![(
        "--network",
        FlagValue::Dropdown("mainnet".into()),
    )])
    .with_slots(slots);
    let argv = assemble_argv(&schema::mnemonic::SCHEMA, subcommand("bundle"), &state);
    let slot_pairs: Vec<&str> = argv
        .iter()
        .filter(|s| s.starts_with("@"))
        .map(String::as_str)
        .collect();
    assert_eq!(
        slot_pairs,
        vec!["@0.xpub=xpub_zero", "@2.xpub=xpub_two"]
    );
}

#[test]
fn cell_5_subcommand_without_allows_slots_ignores_slot_state() {
    // `convert` has allows_slots = false. Slot rows in the form state
    // must NOT leak into argv (the schema doesn't carry --slot for it).
    let slots = SlotState {
        rows: vec![SlotRow {
            index: 0,
            subkey: SlotSubkey::Xpub,
            value: "xpub_leak".into(),
        }],
    };
    let state = FormState::from_pairs(vec![(
        "--from",
        FlagValue::NodeValueComposite {
            node: "xpub".into(),
            value: "xpub...".into(),
        },
    )])
    .with_slots(slots);
    let argv = assemble_argv(&schema::mnemonic::SCHEMA, subcommand("convert"), &state);
    for tok in &argv {
        assert!(
            !tok.contains("xpub_leak"),
            "slot row leaked into convert argv: {:?}",
            argv
        );
        assert!(
            !tok.starts_with("@"),
            "slot @-token leaked into convert argv: {:?}",
            argv
        );
    }
}
