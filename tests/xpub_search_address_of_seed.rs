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
