//! `xpub-search address-of-seed` (toolkit PR #230; toolkit FOLLOWUP
//! `address-of-seed-gui-lockstep`): the secret surfaces of the new form.
//!
//! - the six seed/passphrase flags are `secret: true` (gui-schema ground
//!   truth) and `--from` is a repeating composite whose every node is
//!   argv-secret;
//! - the argv mask covers `--phrase`, `--passphrase` and every `--from`
//!   value, and each is a planner source;
//! - Run and Copy fail closed (`no-table-entry`) on every OS until the
//!   channel table is re-measured against a toolkit release that ships the
//!   verb (FOLLOWUP `address-of-seed-gui-lockstep`);
//! - no seed, ms1, entropy, SeedQR digits or passphrase typed into the form
//!   reaches `state.json`.

use mnemonic_gui::form::channels::copy::{copy_commands, CopyState};
use mnemonic_gui::form::channels::plan;
use mnemonic_gui::form::invocation::assemble_argv_with_sources;
use mnemonic_gui::form::secret_widget::SecretLineEdit;
use mnemonic_gui::persistence::{load, save, PersistedState, SCHEMA_VERSION};
use mnemonic_gui::schema::{self, FlagKind, FlagValue, FormState};
use mnemonic_gui::secrets::{flag_is_secret, node_type_is_argv_secret};

const SUB: &str = "xpub-search-address-of-seed";
const KEY: &str = "mnemonic:xpub-search-address-of-seed";

const PHRASE: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const PHRASE2: &str = "legal winner thank year wave sausage worth useful legal winner thank yellow";
const MS1: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq";
const ENTROPY: &str = "7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f";
const SEEDQR: &str = "000000000000000000000000000000000000000000000003";
const PASSPHRASE: &str = "correct-horse-aos-passphrase";
const TARGET: &str = "bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu";
const COSIGNER: &str = "@1=xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V";

fn sub() -> &'static schema::SubcommandSchema {
    schema::mnemonic::SCHEMA
        .subcommands
        .iter()
        .find(|s| s.name == SUB)
        .expect("xpub-search-address-of-seed in the mnemonic schema")
}

fn flag(name: &str) -> &'static schema::FlagSchema {
    sub()
        .flags
        .iter()
        .find(|f| f.name == name)
        .unwrap_or_else(|| panic!("{name} on {SUB}"))
}

fn composite(node: &str, value: &str) -> (String, FlagValue) {
    (
        "--from".into(),
        FlagValue::NodeValueComposite {
            node: node.into(),
            value: value.into(),
        },
    )
}

/// The form as the live widgets hold it: secret Text flags in
/// `secret_widgets`, `--from` rows as composites in `values`.
fn filled_form() -> FormState {
    let mut state = FormState::default();
    state
        .values
        .push(("--target-address".into(), FlagValue::Text(TARGET.into())));
    state
        .values
        .push(("--cosigner".into(), FlagValue::Text(COSIGNER.into())));
    state.values.push(composite("phrase", PHRASE2));
    state.values.push(composite("ms1", MS1));
    state.values.push(composite("entropy", ENTROPY));
    state.values.push(composite("seedqr", SEEDQR));
    state
        .values
        .push(("--phrase-stdin".into(), FlagValue::Boolean(true)));
    state
        .values
        .push(("--passphrase-stdin".into(), FlagValue::Boolean(true)));
    state
        .secret_widgets
        .insert("--phrase".into(), vec![SecretLineEdit::from_text(PHRASE)]);
    state.secret_widgets.insert(
        "--passphrase".into(),
        vec![SecretLineEdit::from_text(PASSPHRASE)],
    );
    state
        .secret_widgets
        .insert("--ms1".into(), vec![SecretLineEdit::from_text(MS1)]);
    state
}

#[test]
fn aos_secret_flags_are_classified() {
    for name in [
        "--phrase",
        "--phrase-stdin",
        "--ms1",
        "--ms1-stdin",
        "--passphrase",
        "--passphrase-stdin",
    ] {
        assert!(flag(name).secret, "{SUB} {name} must be secret: true");
        assert!(flag_is_secret(flag(name)));
    }
    let from = flag("--from");
    assert!(from.repeating, "--from is repeatable (one row per seed)");
    let FlagKind::NodeValueComposite(nodes) = from.kind else {
        panic!("--from must be a node=value composite");
    };
    assert_eq!(nodes, &["phrase", "ms1", "entropy", "seedqr"]);
    for n in nodes {
        assert!(
            node_type_is_argv_secret(n),
            "every --from node of {SUB} is seed material; `{n}` must be argv-secret"
        );
    }
    let target = flag("--target-address");
    assert!(target.required && target.repeating);
}

#[test]
fn aos_argv_masks_every_secret_and_lists_it_as_a_source() {
    let state = filled_form();
    let (argv, mask, sites, _) =
        assemble_argv_with_sources(&schema::mnemonic::SCHEMA, sub(), &state);
    assert_eq!(&argv[1..3], ["xpub-search", "address-of-seed"]);
    for secret in [
        PHRASE.to_string(),
        PASSPHRASE.to_string(),
        MS1.to_string(),
        format!("phrase={PHRASE2}"),
        format!("ms1={MS1}"),
        format!("entropy={ENTROPY}"),
        format!("seedqr={SEEDQR}"),
    ] {
        let i = argv
            .iter()
            .position(|t| *t == secret)
            .unwrap_or_else(|| panic!("{secret:?} emitted; argv {argv:?}"));
        assert!(mask[i], "{secret:?} must be masked");
        assert!(
            sites.iter().any(|s| s.value_at.contains(&i)),
            "{secret:?} must be a planner source"
        );
    }
    for public in [TARGET, COSIGNER] {
        let i = argv
            .iter()
            .position(|t| t == public)
            .expect("public emitted");
        assert!(!mask[i], "{public} is public and stays readable");
    }
    // Secret stdin toggles never emit (the plan owns stdin).
    assert!(!argv
        .iter()
        .any(|t| t == "--phrase-stdin" || t == "--passphrase-stdin"));
}

#[test]
fn aos_run_and_copy_fail_closed_until_measured() {
    // The channel table is measured against the pinned toolkit, which has
    // no address-of-seed; a secret-bearing Run must refuse on every OS
    // rather than guess a channel (DESIGN §A0).
    let none = |_: &str| None;
    for os in ["linux", "macos", "windows"] {
        for state in [
            filled_form(),
            FormState::from_pairs(vec![
                ("--target-address", FlagValue::Text(TARGET.into())),
                (
                    "--from",
                    FlagValue::NodeValueComposite {
                        node: "phrase".into(),
                        value: PHRASE.into(),
                    },
                ),
            ]),
        ] {
            let r = plan(&schema::mnemonic::SCHEMA, sub(), &state, &none, os);
            assert_eq!(
                r.err().map(|e| e.code),
                Some("no-table-entry"),
                "{os}: a secret-bearing address-of-seed run must refuse until the \
                 channel table measures the verb"
            );
        }
    }
    let (posix, windows) = copy_commands(&schema::mnemonic::SCHEMA, sub(), &filled_form(), &none);
    for c in [posix, windows] {
        assert!(
            matches!(c, CopyState::Disabled(_)),
            "Copy must be disabled: {c:?}"
        );
    }
}

#[test]
fn aos_no_secret_reaches_state_json() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("state.json");
    let mut map = std::collections::BTreeMap::new();
    map.insert(KEY.to_string(), filled_form());
    let state = PersistedState {
        schema_version: SCHEMA_VERSION,
        last_cli_tab: "mnemonic".into(),
        last_subcommand_per_tab: std::collections::BTreeMap::new(),
        window_size: None,
        window_position: None,
        show_cmdline: true,
        show_stdout: true,
        show_stderr: true,
        form_state_per_subcommand: map,
    };
    save(&state, &path).unwrap();
    let raw = std::fs::read_to_string(&path).unwrap();
    for secret in [PHRASE, PHRASE2, MS1, ENTROPY, SEEDQR, PASSPHRASE] {
        assert!(
            !raw.contains(secret),
            "{secret:?} leaked into state.json:\n{raw}"
        );
    }
    for word in ["abandon", "sausage"] {
        assert!(
            !raw.contains(word),
            "seed word {word:?} leaked into state.json"
        );
    }
    // The public inputs persist (they are cheap to keep and not secret).
    let loaded = load(&path).expect("load");
    let form = loaded
        .form_state_per_subcommand
        .get(KEY)
        .expect("form kept");
    assert!(form
        .values
        .iter()
        .any(|(k, v)| k == "--target-address" && *v == FlagValue::Text(TARGET.into())));
    assert!(!form.values.iter().any(|(k, _)| k == "--from"));
    assert!(form.secret_widgets.is_empty());
}

const XPRV: &str = "xprv9s21ZrQH143K3QTDL4LXw2F7HEK3wJUD2nW2nRk4stbPy6cq3jPPqjiChkVvvNKmPGJxWUtg6LnF5kejMRNNU3TGtRBeJgk33yuGBxrMPHi";
const TPRV: &str = "tprv8ZgxMBicQKsPeDgjzdC36fs6bMjGApWDNLR9erAXMs5skhMv36j9MV5ecvfavji5khqjWaWSFhN3YcCUUdiKH6isR4Pwy3U5y5egddBr16m";
/// One private key (BIP-0013 test key 0C28FC…AA1D) in all four WIF forms:
/// mainnet/testnet × uncompressed (33-byte payload) / compressed (34).
const WIFS: [&str; 4] = [
    "5HueCGU8rMjxEXxiPuD5BDku4MkFqeZyd4dZ1jvhTVqvbTLvyTJ",
    "KwdMAjGmerYanjeui5SHS7JkmpZvVipYvB2LJGU1ZxJwYvP98617",
    "91gGn1HgSap6CbU12F6z3pJri26xzp7Ay1VW6NHCoEayNXwRpu2",
    "cMzLdeGd5vEqxB8B6VFQoRopQ3sLAAvEzDAoQgvX54xwofSWj1fx",
];

fn private_keys() -> Vec<&'static str> {
    let mut v = vec![XPRV, TPRV];
    v.extend(WIFS);
    v
}

#[test]
fn text_holds_private_key_recognises_wif_without_false_positives() {
    use mnemonic_gui::secrets::text_holds_private_key;
    for k in private_keys() {
        assert!(text_holds_private_key(k), "{k} is a private key");
        assert!(
            text_holds_private_key(&format!("wpkh({k})")),
            "{k} inside a descriptor"
        );
        assert!(text_holds_private_key(&format!("@1={k}")), "{k} as @N=");
    }
    for public in [
        "xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V",
        "tpubD6NzVbkrYhZ4XgiXtGrdW5XDAPFCL9h7we1vwNCpn8tGbBcgfVYjXyhWo4E1xkh56hjod1RhGjxbaTLV3X4FyWuejifB9jusQ46QzG87VKp",
        TARGET,
        "1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2",
        "3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy",
        "tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx",
        "mipcBbFg9gMiCh81Kj8tqqdgoZub1ZJRfn",
        "md1qqpqqxqxkceprx7rap4t",
        "mk1qpzry9x8gf2tvdw0s3jn54khce6mua7l",
        "wpkh([d34db33f/84h/0h/0h]@0/<0;1>/*)",
    ] {
        assert!(!text_holds_private_key(public), "{public} is not a private key");
    }
}

/// Final review I1: a WIF is classified by SHAPE, as the toolkit does
/// (`[5KLc9]` + 50-51 base58), not by checksum — a mistyped WIF is still a
/// private key. A base58 run of 36+ characters starting `[5KLc9]` is a WIF
/// being typed (no legacy address is longer than 35 characters, and none
/// starts with those characters). Glued and `0`-bounded keys count too.
#[test]
fn text_holds_private_key_classifies_wif_by_shape() {
    use mnemonic_gui::secrets::text_holds_private_key;
    let wif = WIFS[1];
    let mut shaped: Vec<String> = vec![
        // last character mistyped (checksum broken)
        "KwdMAjGmerYanjeui5SHS7JkmpZvVipYvB2LJGU1ZxJwYvP98618".into(),
        // WIF length and prefix, another version byte
        "L6Cyjtdfq1TimFfgG9qBEHQF2FmaL96H5ceTrip74S4eqFapbcDJ".into(),
        // the first 50 of 52 characters, and a 36-character prefix
        wif[..50].into(),
        wif[..36].into(),
        // glued to a non-base58 character, and two glued together
        format!("{wif}0"),
        format!("0{wif}"),
        format!("{wif}{wif}"),
    ];
    for w in WIFS {
        shaped.push(w[..36].into());
    }
    for v in &shaped {
        assert!(text_holds_private_key(v), "{v} is WIF-shaped");
        assert!(text_holds_private_key(&format!("@1={v}")), "@1={v}");
        assert!(text_holds_private_key(&format!("wpkh({v})")), "wpkh({v})");
    }
    // A 35-character prefix is below the threshold.
    assert!(!text_holds_private_key(&wif[..35]));
}

/// Final review I1: no false positive on any public value these fields take,
/// including base58 addresses of every legacy kind and length, bech32 and
/// bech32m (upper and lower case, whose `0`/`l`-split chunks can start with
/// `5`/`9`/`c`/`K`/`L`), raw hex keys, md1/mk1 cards and xpub forms.
#[test]
fn text_holds_private_key_has_no_false_positive_on_public_values() {
    use mnemonic_gui::secrets::text_holds_private_key;
    let public = [
        // P2PKH mainnet (34 and 33 characters), P2SH (34), testnet m/n/2
        "1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2",
        "1111111111111111111114oLvT2",
        "16UwLL9Risc3QfPqBUvKofHmBQ7wMtjvM",
        "3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy",
        "mipcBbFg9gMiCh81Kj8tqqdgoZub1ZJRfn",
        "n3GNqMveyvaPvUbH469vDRadqpJMPc84JA",
        "2MzQwSSnBHWHqSAqtTVQ6v47XtaisrJa1Vc",
        // bech32 / bech32m, lower and upper case, P2WPKH / P2WSH / P2TR
        "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4",
        "BC1QW508D6QEJXTDG4Y5R3ZARVARY0C5XW7KV8F3T4",
        "bc1qrp33g0q5c5txsp9arysrx4k6zdkfs4nce4xj0gdcccefvpysxf3qccfmv3",
        "BC1QRP33G0Q5C5TXSP9ARYSRX4K6ZDKFS4NCE4XJ0GDCCCEFVPYSXF3QCCFMV3",
        "tb1qrp33g0q5c5txsp9arysrx4k6zdkfs4nce4xj0gdcccefvpysxf3q0sl5k7",
        "bc1p5d7rjq7g6rdk2yhzks9smlaqtedr4dekq08ge8ztwac72sfr9rusxg3297",
        "BC1P5D7RJQ7G6RDK2YHZKS9SMLAQTEDR4DEKQ08GE8ZTWAC72SFR9RUSXG3297",
        "bc1pqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqsyjer9e",
        // a chunk after `0` that starts with `9`/`c`/`5` and runs 51+ characters
        "bc1p09ccccccccccccccccccccccccccccccccccccccccccccccccccccccccc5",
        "md10c99999999999999999999999999999999999999999999999999999999999999999",
        "mk1l5cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
        // raw hex keys (33-byte compressed, x-only) whose `0`-split runs are long
        "0299999999999999999999999999999999999999999999999999999999999999c9",
        "c999999999999999999999999999999999999999999999999999999999999999",
        "tr(c999999999999999999999999999999999999999999999999999999999999999)",
        // xpub forms
        "xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V",
        "[d34db33f/48h/0h/0h/2h]xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V/<0;1>/*",
        // paths and templates
        "m/84'/0'/account'",
        "84h/0h/0h/0/5",
        "md1=md1qqpqqxqxkceprx7rap4t",
    ];
    for v in public {
        assert!(!text_holds_private_key(v), "{v} is public");
    }
}

/// Final review I2: an ms1 card (any case) in a public key field is secret
/// material, bare or as `@N=`.
#[test]
fn text_holds_private_key_flags_an_ms1_card() {
    use mnemonic_gui::secrets::text_holds_private_key;
    let upper = MS1.to_ascii_uppercase();
    for v in [MS1, upper.as_str(), "ms1", "MS1q"] {
        assert!(text_holds_private_key(v), "{v}");
        assert!(text_holds_private_key(&format!("@1={v}")), "@1={v}");
        assert!(text_holds_private_key(&format!("wpkh({v})")), "wpkh({v})");
    }
    assert!(!text_holds_private_key(
        "mk1qpzry9x8gf2tvdw0s3jn54khce6mua7l"
    ));
    assert!(!text_holds_private_key("md1qqpqqxqxkceprx7rap4t"));
}

/// Final review I1/I2 and M2: a shaped WIF, an ms1 card or an xprv in
/// `--cosigner`, `--descriptor`, `--target-address`, `--add-path` or
/// `--descriptor-from` is masked, Copy says it reveals a secret, it is never
/// saved, and Run is refused (M1: the toolkit takes these only on argv and
/// refuses them at intake, so the GUI does not send them).
#[test]
fn aos_shaped_secret_in_any_public_field_is_masked_unsaved_and_run_refused() {
    use mnemonic_gui::form::channels::copy::copy_reveals_secret;
    use mnemonic_gui::form::invocation::assemble_argv_with_secret_mask;
    use mnemonic_gui::persistence::redact_for_persistence;
    let none = |_: &str| None;
    let bad_wif = "KwdMAjGmerYanjeui5SHS7JkmpZvVipYvB2LJGU1ZxJwYvP98618";
    for (field, value) in [
        ("--cosigner", format!("@1={bad_wif}")),
        ("--cosigner", MS1.to_string()),
        ("--cosigner", format!("@1={}", MS1.to_ascii_uppercase())),
        ("--descriptor", format!("wpkh({bad_wif})")),
        ("--target-address", XPRV.to_string()),
        ("--target-address", WIFS[0].to_string()),
        ("--target-address", MS1.to_string()),
        ("--add-path", WIFS[1].to_string()),
        ("--descriptor-from", format!("md1={XPRV}")),
    ] {
        let mut pairs = vec![(field, FlagValue::Text(value.clone()))];
        if field != "--target-address" {
            pairs.push(("--target-address", FlagValue::Text(TARGET.into())));
        }
        let state = FormState::from_pairs(pairs);
        let (argv, mask) = assemble_argv_with_secret_mask(&schema::mnemonic::SCHEMA, sub(), &state);
        let i = argv.iter().position(|t| *t == value).expect("emitted");
        assert!(mask[i], "{field} {value} must be masked");
        assert!(
            copy_reveals_secret(&schema::mnemonic::SCHEMA, sub(), &state, &none),
            "Copy must say it reveals {field} {value}"
        );
        let saved = redact_for_persistence(&state);
        assert!(
            !saved
                .values
                .iter()
                .any(|(n, v)| n == field && *v == FlagValue::Text(value.clone())),
            "{field} {value} must not persist"
        );
        for os in ["linux", "macos", "windows"] {
            let r = plan(&schema::mnemonic::SCHEMA, sub(), &state, &none, os);
            match r {
                Err(e) => assert_eq!(e.code, "private-key-in-public-field", "{field} {os}: {e}"),
                Ok(p) => panic!("{field} {value} on {os} must not run: {:?}", p.argv.len()),
            }
        }
    }
    // Public values in the same fields are untouched by the refusal.
    let state = FormState::from_pairs(vec![
        ("--target-address", FlagValue::Text(TARGET.into())),
        ("--cosigner", FlagValue::Text(COSIGNER.into())),
        ("--add-path", FlagValue::Text("m/84'/0'/account'".into())),
    ]);
    let (argv, mask) = assemble_argv_with_secret_mask(&schema::mnemonic::SCHEMA, sub(), &state);
    for v in [TARGET, COSIGNER, "m/84'/0'/account'"] {
        let i = argv.iter().position(|t| t == v).unwrap();
        assert!(!mask[i], "{v} stays readable");
    }
    if let Err(e) = plan(&schema::mnemonic::SCHEMA, sub(), &state, &none, "linux") {
        assert_ne!(e.code, "private-key-in-public-field", "{e}");
    }
}

/// Final review M4: the rendered `--cosigner` field is in password mode
/// with an xprv, a shaped WIF or an ms1 card in it, and stays readable with
/// an xpub (the widget passes the schema name; the mask once never matched
/// a nested verb).
#[test]
fn aos_cosigner_widget_renders_a_private_key_masked() {
    use egui_kittest::kittest::Queryable;
    use egui_kittest::Harness;
    use mnemonic_gui::app::CliTab;
    use mnemonic_gui::form::widget;
    let harness = |initial: String| {
        Harness::new_ui_state(
            move |ui, value: &mut FlagValue| {
                let state = FormState::default();
                widget::render(
                    ui,
                    CliTab::Mnemonic,
                    SUB,
                    flag("--cosigner"),
                    value,
                    &state,
                    &[],
                );
            },
            FlagValue::Text(initial),
        )
    };
    for v in [
        format!("@1={XPRV}"),
        "@1=KwdMAjGmerYanjeui5SHS7JkmpZvVipYvB2LJGU1ZxJwYvP98618".to_string(),
        MS1.to_string(),
    ] {
        let mut h = harness(v.clone());
        h.run();
        assert!(
            h.query_by_role(egui::accesskit::Role::PasswordInput)
                .is_some(),
            "--cosigner {v} must render masked"
        );
    }
    let mut h = harness(COSIGNER.to_string());
    h.run();
    assert!(
        h.query_by_role(egui::accesskit::Role::PasswordInput)
            .is_none(),
        "a public cosigner stays readable"
    );
}

/// Review r1 I1: an xprv/tprv/WIF in `--cosigner` or `--descriptor` is
/// masked in the command, flagged on Copy and never persisted.
#[test]
fn aos_private_key_in_cosigner_or_descriptor_is_masked_flagged_and_not_saved() {
    use mnemonic_gui::form::channels::copy::copy_reveals_secret;
    use mnemonic_gui::form::invocation::assemble_argv_with_secret_mask;
    use mnemonic_gui::persistence::redact_for_persistence;
    use mnemonic_gui::secrets::field_masks_private_key_content;
    // The widget keys on the schema name, the argv on its tokens.
    for base in [
        "mnemonic xpub-search address-of-seed",
        "mnemonic xpub-search-address-of-seed",
    ] {
        for f in ["--descriptor", "--cosigner"] {
            assert!(field_masks_private_key_content(base, f), "{base} {f}");
        }
    }
    let none = |_: &str| None;
    for k in private_keys() {
        for (field, value) in [
            ("--cosigner", format!("@1={k}")),
            ("--descriptor", format!("wpkh({k}/<0;1>/*)")),
        ] {
            let state = FormState::from_pairs(vec![
                ("--target-address", FlagValue::Text(TARGET.into())),
                (field, FlagValue::Text(value.clone())),
            ]);
            let (argv, mask) =
                assemble_argv_with_secret_mask(&schema::mnemonic::SCHEMA, sub(), &state);
            let i = argv.iter().position(|t| *t == value).expect("emitted");
            assert!(mask[i], "{field} {value} must be masked");
            assert!(
                copy_reveals_secret(&schema::mnemonic::SCHEMA, sub(), &state, &none),
                "Copy must say it reveals {field} {value}"
            );
            let saved = redact_for_persistence(&state);
            assert!(
                !saved.values.iter().any(|(n, _)| n == field),
                "{field} {value} must not persist"
            );
            assert!(saved.values.iter().any(|(n, _)| n == "--target-address"));
        }
    }
    // A public cosigner stays readable and persists.
    let state = FormState::from_pairs(vec![("--cosigner", FlagValue::Text(COSIGNER.into()))]);
    let (argv, mask) = assemble_argv_with_secret_mask(&schema::mnemonic::SCHEMA, sub(), &state);
    let i = argv.iter().position(|t| t == COSIGNER).unwrap();
    assert!(!mask[i]);
    assert!(redact_for_persistence(&state)
        .values
        .iter()
        .any(|(n, _)| n == "--cosigner"));
}

/// Review r1 I1, sibling verbs: the public key fields of the other
/// xpub-search verbs had the same gap.
#[test]
fn xpub_search_sibling_key_fields_mask_private_keys() {
    use mnemonic_gui::form::invocation::assemble_argv_with_secret_mask;
    use mnemonic_gui::persistence::redact_for_persistence;
    for (sub_name, field, wrap) in [
        (
            "xpub-search-account-of-descriptor",
            "--descriptor",
            "wpkh({})",
        ),
        ("xpub-search-address-of-xpub", "--xpub", "{}"),
        ("xpub-search-path-of-xpub", "--target-xpub", "{}"),
        ("xpub-search-passphrase-of-xpub", "--target-xpub", "{}"),
    ] {
        let s = schema::mnemonic::SCHEMA
            .subcommands
            .iter()
            .find(|s| s.name == sub_name)
            .unwrap();
        for k in private_keys() {
            let value = wrap.replace("{}", k);
            let state = FormState::from_pairs(vec![(field, FlagValue::Text(value.clone()))]);
            let (argv, mask) = assemble_argv_with_secret_mask(&schema::mnemonic::SCHEMA, s, &state);
            let i = argv.iter().position(|t| *t == value).expect("emitted");
            assert!(mask[i], "{sub_name} {field} {value} must be masked");
            assert!(
                !redact_for_persistence(&state)
                    .values
                    .iter()
                    .any(|(n, _)| n == field),
                "{sub_name} {field} {value} must not persist"
            );
        }
    }
}

/// Review r1 M1: every seed/passphrase source refuses on its own, in each
/// of its spellings (literal, `@env:` set, `-`).
#[test]
fn aos_each_source_alone_fails_closed() {
    let env = |n: &str| (n == "AOS_SET").then(|| "value".to_string());
    let mut states: Vec<(String, FormState)> = Vec::new();
    for (flag_name, literal) in [
        ("--phrase", PHRASE),
        ("--ms1", MS1),
        ("--passphrase", PASSPHRASE),
    ] {
        for v in [literal, "@env:AOS_SET", "-"] {
            let mut s =
                FormState::from_pairs(vec![("--target-address", FlagValue::Text(TARGET.into()))]);
            s.secret_widgets
                .insert(flag_name.into(), vec![SecretLineEdit::from_text(v)]);
            states.push((format!("{flag_name} {v}"), s));
        }
    }
    for (node, literal) in [
        ("phrase", PHRASE),
        ("ms1", MS1),
        ("entropy", ENTROPY),
        ("seedqr", SEEDQR),
    ] {
        for v in [literal, "@env:AOS_SET", "-"] {
            let mut s =
                FormState::from_pairs(vec![("--target-address", FlagValue::Text(TARGET.into()))]);
            s.values.push(composite(node, v));
            states.push((format!("--from {node}={v}"), s));
        }
    }
    for (label, state) in &states {
        for os in ["linux", "macos", "windows"] {
            let code = plan(&schema::mnemonic::SCHEMA, sub(), state, &env, os)
                .err()
                .map(|e| e.code);
            assert!(
                matches!(code, Some(c) if c == "no-table-entry" || c.starts_with("C1-")),
                "{os} {label}: must refuse, got {code:?}"
            );
        }
        let (posix, windows) = copy_commands(&schema::mnemonic::SCHEMA, sub(), state, &env);
        for c in [posix, windows] {
            assert!(matches!(c, CopyState::Disabled(_)), "{label}: Copy {c:?}");
        }
    }
}
