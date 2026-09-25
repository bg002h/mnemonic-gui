//! F-679 — `--allow-argv-secret` is GUI-managed.
//!
//! `mnemonic` (toolkit v0.104.0) and `ms` (v0.19.0) refuse secret material on
//! argv unless `--allow-argv-secret` is present. The GUI mirrors the flag for
//! schema parity but never renders it and never emits it from form state; the
//! RUN path (`admit_argv_secret_for_run` / `assemble_argv_for_run`) adds it
//! only when the argv carries secret material, and the COPY path
//! (`assemble_argv`) never carries it.

use mnemonic_gui::form::invocation::{
    admit_argv_secret_for_run, assemble_argv, assemble_argv_for_run,
    assemble_argv_with_secret_mask, is_gui_managed_flag, ALLOW_ARGV_SECRET, END_OF_OPTIONS,
};
use mnemonic_gui::form::secret_widget::SecretLineEdit;
use mnemonic_gui::schema::{self, FlagValue, FormState, Schema, SubcommandSchema};

fn sub(schema: &'static Schema, name: &str) -> &'static SubcommandSchema {
    schema
        .subcommands
        .iter()
        .find(|s| s.name == name)
        .unwrap_or_else(|| panic!("{} {name}", schema.cli_name))
}

const MS1: &str = "ms10testsxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx";

fn ms_decode_state() -> FormState {
    let mut state = FormState::default();
    state.secret_widgets.insert(
        "positional:ms1".into(),
        vec![SecretLineEdit::from_text(MS1)],
    );
    state
}

#[test]
fn run_argv_admits_right_after_the_subcommand_with_a_false_mask_bit() {
    let s = sub(&schema::ms::SCHEMA, "decode");
    let (argv, mask) = assemble_argv_with_secret_mask(&schema::ms::SCHEMA, s, &ms_decode_state());
    assert!(
        !argv.iter().any(|t| t == ALLOW_ARGV_SECRET),
        "copy argv carries it: {argv:?}"
    );
    let (run, run_mask) = admit_argv_secret_for_run(s, argv.clone(), mask.clone());
    assert_eq!(run[..2], ["ms", "decode"]);
    assert_eq!(run[2], ALLOW_ARGV_SECRET);
    assert!(!run_mask[2], "the opt-in token is not secret");
    assert_eq!(run.len(), argv.len() + 1);
    assert_eq!(run_mask.len(), run.len());
    // Everything else is unchanged and in order.
    let mut rest = run.clone();
    rest.remove(2);
    assert_eq!(rest, argv);
    // The secret value is still masked at its (shifted) position.
    let at = run.iter().position(|t| t == MS1).expect("ms1 in argv");
    assert!(run_mask[at]);
}

#[test]
fn nested_subcommand_gets_the_flag_after_the_child_token() {
    let s = sub(&schema::mnemonic::SCHEMA, "xpub-search-passphrase-of-xpub");
    let argv = vec![
        "mnemonic".to_string(),
        "xpub-search".into(),
        "passphrase-of-xpub".into(),
        "--passphrase".into(),
        "p".into(),
    ];
    let mask = vec![false, false, false, false, true];
    let (run, _) = admit_argv_secret_for_run(s, argv, mask);
    assert_eq!(
        run[..4],
        [
            "mnemonic",
            "xpub-search",
            "passphrase-of-xpub",
            ALLOW_ARGV_SECRET
        ]
    );
}

#[test]
fn no_secret_means_no_flag() {
    let s = sub(&schema::ms::SCHEMA, "decode");
    let state = FormState::default().with_positionals(["-"]);
    let run = assemble_argv_for_run(&schema::ms::SCHEMA, s, &state);
    assert!(!run.iter().any(|t| t == ALLOW_ARGV_SECRET), "{run:?}");
}

#[test]
fn a_cli_that_does_not_declare_it_never_gets_it() {
    // md has no secret surface and no --allow-argv-secret; even a (spurious)
    // secret mask bit must not add a flag md would reject.
    let s = sub(&schema::md::SCHEMA, "decode");
    let argv = vec!["md".to_string(), "decode".into(), "md1xyz".into()];
    let (run, _) = admit_argv_secret_for_run(s, argv.clone(), vec![false, false, true]);
    assert_eq!(run, argv);
}

#[test]
fn restore_from_a_secret_node_is_admitted_but_a_private_channel_is_not() {
    // `restore --from` is a plain Text flag (secret: false upstream AND here),
    // so the mask never marks it; the node-value classifier must.
    let s = sub(&schema::mnemonic::SCHEMA, "restore");
    let with = |v: &str| {
        let state = FormState::from_pairs(vec![("--from", FlagValue::Text(v.into()))]);
        assemble_argv_for_run(&schema::mnemonic::SCHEMA, s, &state)
    };
    assert!(with(&format!("ms1={MS1}"))
        .iter()
        .any(|t| t == ALLOW_ARGV_SECRET));
    for private in ["ms1=-", "ms1=@env:SEED"] {
        assert!(
            !with(private).iter().any(|t| t == ALLOW_ARGV_SECRET),
            "{private} is a private channel and needs no opt-in"
        );
    }
    // Watch-only material is not refused by the toolkit and needs nothing.
    assert!(!with("xpub=xpub6abc").iter().any(|t| t == ALLOW_ARGV_SECRET));
}

#[test]
fn form_state_cannot_emit_it_even_if_set() {
    // A stale persisted `true` (or any state write) must not reach argv: the
    // assembler skips GUI-managed flags outright.
    let s = sub(&schema::mnemonic::SCHEMA, "convert");
    let state = FormState::from_pairs(vec![(ALLOW_ARGV_SECRET, FlagValue::Boolean(true))]);
    let argv = assemble_argv(&schema::mnemonic::SCHEMA, s, &state);
    assert!(!argv.iter().any(|t| t == ALLOW_ARGV_SECRET), "{argv:?}");
}

#[test]
fn admission_is_idempotent() {
    let s = sub(&schema::ms::SCHEMA, "decode");
    let (argv, mask) = assemble_argv_with_secret_mask(&schema::ms::SCHEMA, s, &ms_decode_state());
    let (once, once_mask) = admit_argv_secret_for_run(s, argv, mask);
    let (twice, _) = admit_argv_secret_for_run(s, once.clone(), once_mask);
    assert_eq!(once, twice);
}

#[test]
fn every_declaring_subcommand_is_mnemonic_or_ms_and_the_flag_is_gui_managed() {
    assert!(is_gui_managed_flag(ALLOW_ARGV_SECRET));
    let mut declaring = 0;
    for schema in [
        &schema::mnemonic::SCHEMA,
        &schema::md::SCHEMA,
        &schema::ms::SCHEMA,
        &schema::mk::SCHEMA,
    ] {
        for s in schema.subcommands {
            if let Some(f) = s.flags.iter().find(|f| f.name == ALLOW_ARGV_SECRET) {
                assert!(
                    matches!(schema.cli_name, "mnemonic" | "ms"),
                    "{}",
                    schema.cli_name
                );
                assert!(!f.secret && !f.required && f.default_value.is_none());
                declaring += 1;
            }
        }
    }
    // 32 mnemonic subcommands + the 8 ms material verbs this GUI mirrors.
    assert_eq!(declaring, 40);
}

// ─── real-CLI cells (EARLY-RETURN-SKIP when the pinned binary env is unset,
//     the `ui_harness_i4_realcli` convention) ─────────────────────────────────

fn pinned_bin(env_var: &str) -> Option<String> {
    match std::env::var(env_var) {
        Ok(p) if !p.trim().is_empty() => Some(p),
        _ => {
            eprintln!("f679: {env_var} unset — skipping real-CLI cell (NOT a failure)");
            None
        }
    }
}

const ABANDON: &str = "abandon abandon abandon abandon abandon abandon \
                       abandon abandon abandon abandon abandon about";
/// `ms encode` of the all-zero BIP-39 vector ("abandon … about"), unbroken.
const MS1_ALL_ZERO: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";

fn run_for(
    schema: &'static Schema,
    sub_name: &str,
    bin: String,
    state: &FormState,
) -> mnemonic_gui::runner::RunResult {
    let mut argv = assemble_argv_for_run(schema, sub(schema, sub_name), state);
    argv[0] = bin;
    mnemonic_gui::runner::run(argv).expect("spawn")
}

/// `ms encode` with the phrase in the SECRET widget: the Run path admits it
/// (without the opt-in ms 0.19.0 refuses at exit 1), and the card is right.
#[test]
fn real_ms_encode_phrase_widget_runs_through_the_admission() {
    let Some(bin) = pinned_bin("MS_BIN") else {
        return;
    };
    let mut state = FormState::from_pairs(vec![
        ("--group-size", FlagValue::Number(0)),
        ("--no-engraving-card", FlagValue::Boolean(true)),
    ]);
    state
        .secret_widgets
        .insert("--phrase".into(), vec![SecretLineEdit::from_text(ABANDON)]);
    let r = run_for(&schema::ms::SCHEMA, "encode", bin, &state);
    assert!(r.argv.iter().any(|t| t == ALLOW_ARGV_SECRET));
    assert_eq!(
        r.exit_code,
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&r.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&r.stdout).trim(), MS1_ALL_ZERO);
}

/// `ms encode --in FILE --out FILE` — the PRIVATE channel the new surface
/// exposes: nothing secret on argv, so NO opt-in, and the artifact lands in the
/// 0600 file instead of stdout.
#[test]
fn real_ms_encode_in_and_out_files_need_no_opt_in() {
    let Some(bin) = pinned_bin("MS_BIN") else {
        return;
    };
    let dir = tempfile::tempdir().unwrap();
    let inp = dir.path().join("phrase.txt");
    let out = dir.path().join("card.ms1");
    std::fs::write(&inp, ABANDON).unwrap();
    let state = FormState::from_pairs(vec![
        ("--in", FlagValue::Path(inp.to_string_lossy().into())),
        ("--out", FlagValue::Path(out.to_string_lossy().into())),
        ("--group-size", FlagValue::Number(0)),
        ("--no-engraving-card", FlagValue::Boolean(true)),
    ]);
    let r = run_for(&schema::ms::SCHEMA, "encode", bin, &state);
    assert!(
        !r.argv.iter().any(|t| t == ALLOW_ARGV_SECRET),
        "{:?}",
        r.argv
    );
    assert_eq!(
        r.exit_code,
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&r.stderr)
    );
    let written = std::fs::read_to_string(&out).unwrap();
    assert_eq!(written.trim(), MS1_ALL_ZERO);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&out).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "ms --out is owner-only");
    }
}

/// `md decode --in FILE` (new md 0.20.3 surface): md1 read from a file, no
/// positional. md declares no opt-in and must never get one.
#[test]
fn real_md_decode_in_file() {
    let Some(bin) = pinned_bin("MD_BIN") else {
        return;
    };
    let dir = tempfile::tempdir().unwrap();
    let inp = dir.path().join("card.md1");
    std::fs::write(&inp, "md1yqpqqxqq8xtwhw4xwn4qh\n").unwrap();
    let state = FormState::from_pairs(vec![
        ("--in", FlagValue::Path(inp.to_string_lossy().into())),
        ("--json", FlagValue::Boolean(true)),
    ]);
    let r = run_for(&schema::md::SCHEMA, "decode", bin, &state);
    assert!(!r.argv.iter().any(|t| t == ALLOW_ARGV_SECRET));
    assert_eq!(
        r.exit_code,
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&r.stderr)
    );
    let v: serde_json::Value = serde_json::from_slice(&r.stdout).expect("json");
    assert_eq!(v["descriptor"]["tree"]["tag"].as_str(), Some("Wpkh"));
}

/// Every separator the GUI still offers is one the pinned CLI accepts — the
/// drift gate cannot see this (all four report `--separator` as `text`).
#[test]
fn real_every_offered_separator_is_accepted() {
    use mnemonic_gui::schema::FlagKind;
    let cases: [(&'static Schema, &str, &str, Vec<String>); 4] = [
        (&schema::mnemonic::SCHEMA, "bundle", "MNEMONIC_BIN", vec![]),
        (
            &schema::md::SCHEMA,
            "encode",
            "MD_BIN",
            vec!["wpkh(@0/<0;1>/*)".into()],
        ),
        (&schema::mk::SCHEMA, "encode", "MK_BIN", vec![]),
        (&schema::ms::SCHEMA, "encode", "MS_BIN", vec![]),
    ];
    for (schema, sub_name, env, positionals) in cases {
        let Some(bin) = pinned_bin(env) else { continue };
        let s = sub(schema, sub_name);
        let flag = s.flags.iter().find(|f| f.name == "--separator").unwrap();
        let FlagKind::Dropdown(opts) = flag.kind else {
            panic!("dropdown")
        };
        for sep in opts {
            // `--help` short-circuits nothing about value parsing: clap parses
            // the whole argv first, so a refused `--separator` value still
            // fails, while an accepted one prints help and exits 0.
            let out = std::process::Command::new(&bin)
                .args([sub_name, "--separator", sep, "--help"])
                .args(&positionals)
                .output()
                .unwrap();
            assert!(
                out.status.success(),
                "{} {sub_name} --separator {sep} refused: {}",
                schema.cli_name,
                String::from_utf8_lossy(&out.stderr)
            );
        }
    }
}

// ─── F-679 fold 1 (review I2): `--` before positionals ─────────────────────

/// A keyless wpkh policy card (md 0.20.3) and the two mk1 cards of the BIP-84
/// account-0 xpub of the published "abandon … about" vector (mk 0.13.0,
/// `--policy-id-stub 00000000`). Watch-only; the composed first address is the
/// well-known `bc1qcr8te4…`.
const MD1_KEYLESS_WPKH: &str = "md1yq802gggqpsqwgtua24e7ssf3";
const MK1_ACCT0_A: &str = "mk1qpe9m4pqqsqsqqqqqpeutks2qvzg3vs70mejhk622ws2kgdemj2cd8zwj2skzx2wq0qw70l4q99vdyh5x0z8v4yslsp8qjt8k8r9fgxmzmht";
const MK1_ACCT0_B: &str =
    "mk1qpe9m4pp0f30mtxzd65mvwcur9usdatwuqvq6z70r9nwrgk6xn6l8gy6n0yhwh6mr79jfqallmwff";
const ACCT0_FIRST_ADDRESS: &str = "bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu";

fn md_address_from_mk1_state() -> FormState {
    FormState::from_pairs(vec![
        ("--from-mk1", FlagValue::Text(MK1_ACCT0_A.into())),
        ("--from-mk1", FlagValue::Text(MK1_ACCT0_B.into())),
    ])
    .with_positionals([MD1_KEYLESS_WPKH])
}

#[test]
fn positionals_follow_an_end_of_options_marker() {
    let s = sub(&schema::md::SCHEMA, "address");
    let argv = assemble_argv(&schema::md::SCHEMA, s, &md_address_from_mk1_state());
    let at = argv
        .iter()
        .position(|t| t == MD1_KEYLESS_WPKH)
        .expect("md1 in argv");
    assert_eq!(argv[at - 1], END_OF_OPTIONS, "{argv:?}");
    assert_eq!(argv.iter().filter(|t| *t == END_OF_OPTIONS).count(), 1);
    // No positionals → no marker.
    let bare = assemble_argv(&schema::md::SCHEMA, s, &FormState::default());
    assert!(!bare.iter().any(|t| t == END_OF_OPTIONS), "{bare:?}");
    // A secret positional keeps its mask bit after the marker.
    let d = sub(&schema::ms::SCHEMA, "decode");
    let (argv, mask) = assemble_argv_with_secret_mask(&schema::ms::SCHEMA, d, &ms_decode_state());
    let at = argv.iter().position(|t| t == MS1).unwrap();
    assert_eq!(argv[at - 1], END_OF_OPTIONS);
    assert!(mask[at] && !mask[at - 1]);
}

/// The review's I2 reproduction, through the GUI's own assembler: before the
/// fold the md1 policy card landed after `--from-mk1 <STRING>...` and md
/// refused it as a third key card (exit 1). Copy, Preview and Run agree here
/// (md declares no opt-in), so this argv is also what the user copies.
#[test]
fn real_md_address_from_mk1_with_the_policy_positional() {
    let Some(bin) = pinned_bin("MD_BIN") else {
        return;
    };
    let s = sub(&schema::md::SCHEMA, "address");
    let state = md_address_from_mk1_state();
    let copy = assemble_argv(&schema::md::SCHEMA, s, &state);
    let run_argv = assemble_argv_for_run(&schema::md::SCHEMA, s, &state);
    assert_eq!(copy, run_argv, "md has no opt-in: copy == run");
    let r = run_for(&schema::md::SCHEMA, "address", bin, &state);
    assert_eq!(
        r.exit_code,
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&r.stderr)
    );
    assert!(
        String::from_utf8_lossy(&r.stdout).contains(ACCT0_FIRST_ADDRESS),
        "stdout: {}",
        String::from_utf8_lossy(&r.stdout)
    );
}

/// F-679 fold 1 (review N1): a secret field holding only a private-channel
/// sentinel gets no opt-in; a real value (even one containing `=`) does.
#[test]
fn private_channel_sentinels_in_secret_fields_need_no_opt_in() {
    let s = sub(&schema::mnemonic::SCHEMA, "convert");
    for (value, expect) in [
        ("@env:SEED", false),
        ("-", false),
        ("abandon abandon", true),
    ] {
        let state = FormState::from_pairs(vec![(
            "--from",
            FlagValue::NodeValueComposite {
                node: "phrase".into(),
                value: value.into(),
            },
        )]);
        let run = assemble_argv_for_run(&schema::mnemonic::SCHEMA, s, &state);
        assert_eq!(
            run.iter().any(|t| t == ALLOW_ARGV_SECRET),
            expect,
            "{value}: {run:?}"
        );
    }
    // A passphrase that merely contains `=` is still material.
    let x = sub(&schema::mnemonic::SCHEMA, "xpub-search-passphrase-of-xpub");
    let mut state = FormState::default();
    state.secret_widgets.insert(
        "--passphrase".into(),
        vec![SecretLineEdit::from_text("a=-")],
    );
    let run = assemble_argv_for_run(&schema::mnemonic::SCHEMA, x, &state);
    assert!(run.iter().any(|t| t == ALLOW_ARGV_SECRET), "{run:?}");
}
