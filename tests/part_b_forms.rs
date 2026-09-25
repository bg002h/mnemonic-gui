//! DESIGN secret channels Part B — the five unsurfaced forms (§B1–§B6).
//!
//! Pure legs: each form is mirrored with the design table's flags and kinds;
//! each conditional rule; `ms hashlock`'s `--kind` ("(choose)" emits nothing
//! and blocks Run; "all kinds" emits nothing, unblocks Run and shows the
//! banner), source exclusivity, `--random` → `--out`, the phrase-only flags,
//! the `--json` notice, and the phrase's byte-verbatim private channel; the
//! B2–B4 `*` rule (a pasted xprv is masked and never persisted).
//!
//! Real-binary legs (`MNEMONIC_BIN` required): each new form, filled with a
//! valid state, runs through the GUI's own plan + runner at exit 0.

mod secret_channels_common;

use mnemonic_gui::form::channels;
use mnemonic_gui::form::conditional::{form_notices, run_blocker};
use mnemonic_gui::form::invocation::{assemble_argv, assemble_argv_with_secret_mask};
use mnemonic_gui::form::secret_widget::SecretLineEdit;
use mnemonic_gui::persistence::redact_for_persistence;
use mnemonic_gui::schema::{
    self, dropdown_unset_label, FlagKind, FlagValue, FormState, Schema, SubcommandSchema,
    Visibility,
};

const X: &str = "xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V";
const XPRV: &str = "xprv9s21ZrQH143K3QTDL4LXw2F7HEK3wJUD2nW2nRk4stbPy6cq3jPPqjiChkVvvNKmPGJxWUtg6LNnd5vjUBGRTb5rYs2PwEAaRS9CBYg5ZEDc";

fn sub(schema: &'static Schema, name: &str) -> &'static SubcommandSchema {
    schema
        .subcommands
        .iter()
        .find(|s| s.name == name)
        .unwrap_or_else(|| panic!("{} {name} is not mirrored", schema.cli_name))
}

fn vis(sub: &SubcommandSchema, st: &FormState, name: &str) -> Visibility {
    (sub.conditional.unwrap())(st)
        .into_iter()
        .find(|(k, _)| *k == name)
        .map(|(_, v)| v)
        .unwrap_or(Visibility::Visible)
}

fn flag_kind(sub: &SubcommandSchema, name: &str) -> &'static FlagKind {
    &sub.flags
        .iter()
        .find(|f| f.name == name)
        .unwrap_or_else(|| panic!("{name}"))
        .kind
}

#[test]
fn b_all_five_forms_are_mirrored_with_the_design_tables_kinds() {
    let md = &schema::md::SCHEMA;
    let c = sub(md, "compose");
    assert!(
        matches!(flag_kind(c, "--wrapper"), FlagKind::Dropdown(o) if *o == ["tr", "wsh", "sh-wsh", "sh"])
    );
    assert!(
        c.flags
            .iter()
            .find(|f| f.name == "--wrapper")
            .unwrap()
            .required
    );
    assert!(
        c.flags
            .iter()
            .find(|f| f.name == "--path")
            .unwrap()
            .repeating
    );
    assert!(
        matches!(flag_kind(c, "--unspendable"), FlagKind::Dropdown(o) if *o == ["", "nums", "liana"])
    );
    let s = sub(md, "shape-key");
    assert!(s.positional_args[0].repeating && !s.positional_args[0].secret);
    let d = sub(md, "descriptor");
    for f in [
        "--template",
        "--key",
        "--fingerprint",
        "--path",
        "--from-mk1",
        "--from-mk1-file",
        "--seat",
        "--network",
        "--chain",
        "--change",
        "--emit",
        "--out",
        "--group-size",
        "--separator",
        "--json",
        "--verify-against",
        "--experimental",
    ] {
        assert!(d.flags.iter().any(|x| x.name == f), "descriptor {f}");
    }
    let dc = sub(md, "decompose");
    assert!(
        matches!(flag_kind(dc, "--emit"), FlagKind::Dropdown(o) if *o == ["all", "template", "keys", "fingerprints", "descriptor", "commands"])
    );
    assert!(!dc.positional_args[0].repeating, "exactly one descriptor");
    let h = sub(&schema::ms::SCHEMA, "hashlock");
    for (f, secret) in [
        ("--hashlock-phrase", true),
        ("--hex", true),
        ("--in", false),
        ("--random", false),
    ] {
        assert_eq!(
            h.flags.iter().find(|x| x.name == f).unwrap().secret,
            secret,
            "{f}"
        );
    }
    assert!(
        h.positional_args[0].secret,
        "the <ms1> plate is hand-marked secret"
    );
    let FlagKind::Dropdown(kinds) = flag_kind(h, "--kind") else {
        panic!()
    };
    assert_eq!(
        *kinds,
        [
            "",
            "sha256",
            "hash256",
            "ripemd160",
            "hash160",
            schema::ms::HASHLOCK_KIND_ALL
        ]
    );
    assert_eq!(dropdown_unset_label(kinds), "(choose)");
    assert_eq!(dropdown_unset_label(&["", "nums"]), "(none)");
}

#[test]
fn b1_compose_path_xor_preset_and_unspendable_only_with_tr() {
    let c = sub(&schema::md::SCHEMA, "compose");
    let st = FormState::from_pairs(vec![("--wrapper", FlagValue::Dropdown("wsh".into()))]);
    assert_eq!(vis(c, &st, "--path"), Visibility::Required);
    assert_eq!(vis(c, &st, "--preset"), Visibility::Required);
    assert_eq!(vis(c, &st, "--unspendable"), Visibility::Disabled);
    let st = FormState::from_pairs(vec![
        ("--wrapper", FlagValue::Dropdown("tr".into())),
        ("--path", FlagValue::Text("2of3".into())),
        ("--path", FlagValue::Text("1of1,older=100".into())),
        ("--unspendable", FlagValue::Dropdown("liana".into())),
    ]);
    assert_eq!(vis(c, &st, "--preset"), Visibility::Disabled);
    assert_eq!(vis(c, &st, "--unspendable"), Visibility::Visible);
    let argv = assemble_argv(&schema::md::SCHEMA, c, &st);
    let joined = argv.join(" ");
    assert!(
        joined.contains("--path 2of3 --path 1of1,older=100"),
        "path order kept: {joined}"
    );
    assert!(joined.contains("--unspendable liana"));
    let st = FormState::from_pairs(vec![(
        "--preset",
        FlagValue::Text("kofn-recovery,2of3".into()),
    )]);
    assert_eq!(vis(c, &st, "--path"), Visibility::Disabled);
    // the (none) sentinel never reaches argv
    let st = FormState::from_pairs(vec![
        ("--wrapper", FlagValue::Dropdown("tr".into())),
        ("--unspendable", FlagValue::Dropdown(String::new())),
    ]);
    assert!(!assemble_argv(&schema::md::SCHEMA, c, &st)
        .iter()
        .any(|t| t == "--unspendable"));
}

#[test]
fn b2_b4_positional_xor_flag() {
    let s = sub(&schema::md::SCHEMA, "shape-key");
    let st = FormState::default().with_positionals(["md1abc"]);
    assert_eq!(vis(s, &st, "--descriptor"), Visibility::Disabled);
    assert_eq!(
        vis(s, &FormState::default(), "--descriptor"),
        Visibility::Required
    );
    let dc = sub(&schema::md::SCHEMA, "decompose");
    let st = FormState::default().with_positionals(["wpkh(...)"]);
    assert_eq!(vis(dc, &st, "--in"), Visibility::Disabled);
    assert_eq!(vis(dc, &FormState::default(), "--in"), Visibility::Required);
}

#[test]
fn b3_descriptor_three_input_modes() {
    let d = sub(&schema::md::SCHEMA, "descriptor");
    // A: phrases → --template/--key/--fingerprint greyed
    let st = FormState::default().with_positionals(["md1abc"]);
    for f in ["--template", "--key", "--fingerprint"] {
        assert_eq!(vis(d, &st, f), Visibility::Disabled, "{f}");
    }
    // B: --template needs ≥1 --key; key cards greyed
    let st = FormState::from_pairs(vec![(
        "--template",
        FlagValue::Text("wpkh(@0/<0;1>/*)".into()),
    )]);
    assert_eq!(vis(d, &st, "--key"), Visibility::Required);
    assert_eq!(vis(d, &st, "--from-mk1"), Visibility::Disabled);
    // --emit md1 only in mode C; --out/--group-size/--separator only with it
    assert_eq!(vis(d, &st, "--emit"), Visibility::Disabled);
    assert_eq!(vis(d, &st, "--out"), Visibility::Disabled);
    let st = FormState::from_pairs(vec![
        ("--from-mk1", FlagValue::Text("mk1x".into())),
        ("--emit", FlagValue::Dropdown("md1".into())),
    ]);
    assert_eq!(vis(d, &st, "--emit"), Visibility::Visible);
    assert_eq!(vis(d, &st, "--out"), Visibility::Visible);
    assert_eq!(vis(d, &st, "--template"), Visibility::Visible);
    assert_eq!(vis(d, &st, "positional:phrases"), Visibility::Required);
    // --chain XOR --change
    let st = FormState::from_pairs(vec![("--change", FlagValue::Boolean(true))]);
    assert_eq!(vis(d, &st, "--chain"), Visibility::Disabled);
}

fn hashlock() -> &'static SubcommandSchema {
    sub(&schema::ms::SCHEMA, "hashlock")
}

fn hl(kind: &str) -> FormState {
    let mut st = FormState::from_pairs(vec![("--kind", FlagValue::Dropdown(kind.into()))]);
    st.secret_widgets.insert(
        "--hashlock-phrase".into(),
        vec![SecretLineEdit::from_text("  pad  ")],
    );
    st
}

#[test]
fn b5_kind_choose_blocks_run_and_all_kinds_emits_nothing_with_a_banner() {
    let h = hashlock();
    // default: "(choose)" — no --kind token, Run disabled
    let st = hl("");
    assert!(!assemble_argv(&schema::ms::SCHEMA, h, &st)
        .iter()
        .any(|t| t == "--kind"));
    assert!(run_blocker("ms", "hashlock", &st).is_some());
    // all kinds — lookup only: no --kind token, Run allowed, the banner shows
    let st = hl(schema::ms::HASHLOCK_KIND_ALL);
    let argv = assemble_argv(&schema::ms::SCHEMA, h, &st);
    assert!(
        !argv
            .iter()
            .any(|t| t == "--kind" || t.contains("all kinds")),
        "{argv:?}"
    );
    assert!(run_blocker("ms", "hashlock", &st).is_none());
    assert!(form_notices("ms", "hashlock", &st)
        .contains(&"stdout is the sha256 record; your wallet's kind may differ"));
    // a real kind emits
    let st = hl("sha256");
    assert!(assemble_argv(&schema::ms::SCHEMA, h, &st)
        .join(" ")
        .contains("--kind sha256"));
    assert!(run_blocker("ms", "hashlock", &st).is_none());
    assert!(form_notices("ms", "hashlock", &st).is_empty());
    // other forms are never blocked
    assert!(run_blocker("ms", "encode", &FormState::default()).is_none());
}

#[test]
fn b5_one_source_random_needs_out_and_phrase_only_flags() {
    let h = hashlock();
    let st = hl("sha256");
    for f in ["--hex", "--in", "--random"] {
        assert_eq!(vis(h, &st, f), Visibility::Disabled, "phrase wins over {f}");
    }
    assert_eq!(vis(h, &st, "--method"), Visibility::Visible);
    assert_eq!(vis(h, &st, "--emit-record"), Visibility::Visible);
    let mut st = FormState::from_pairs(vec![("--random", FlagValue::Boolean(true))]);
    assert_eq!(vis(h, &st, "--out"), Visibility::Required);
    assert_eq!(vis(h, &st, "--method"), Visibility::Disabled);
    assert_eq!(vis(h, &st, "--emit-record"), Visibility::Disabled);
    st.values
        .push(("--out".into(), FlagValue::Path("/tmp/x".into())));
    assert_eq!(vis(h, &st, "--out"), Visibility::Visible);
    // the plate positional wins over everything
    let mut st = hl("sha256");
    st.secret_widgets.insert(
        "positional:ms1".into(),
        vec![SecretLineEdit::from_text("ms10x")],
    );
    assert_eq!(vis(h, &st, "--hashlock-phrase"), Visibility::Disabled);
    // nothing filled: every source Required
    let st = FormState::default();
    assert_eq!(vis(h, &st, "--hashlock-phrase"), Visibility::Required);
    // --json notice
    let mut st = hl("sha256");
    st.values.push(("--json".into(), FlagValue::Boolean(true)));
    assert!(form_notices("ms", "hashlock", &st).contains(&"--json: stdout then carries the secret"));
}

#[test]
fn b5_phrase_is_byte_verbatim_over_its_only_private_channel() {
    let none = |_: &str| None;
    let p = channels::plan(
        &schema::ms::SCHEMA,
        hashlock(),
        &hl("sha256"),
        &none,
        "linux",
    )
    .unwrap();
    assert!(
        p.argv.iter().any(|t| t == "--hashlock-phrase-stdin"),
        "{:?}",
        p.argv
    );
    assert!(!p.argv.iter().any(|t| t.contains("pad")));
    assert_eq!(
        p.stdin.as_deref().map(|b| b.to_vec()),
        Some(b"  pad  \r\n".to_vec()),
        "no trim"
    );
    // the interim path: the exact bytes on argv, separate word
    let p = channels::plan(
        &schema::ms::SCHEMA,
        hashlock(),
        &hl("sha256"),
        &none,
        "macos",
    )
    .unwrap();
    let at = p
        .argv
        .iter()
        .position(|t| t == "  pad  ")
        .expect("verbatim on interim argv");
    assert_eq!(p.argv[at - 1], "--hashlock-phrase");
    assert!(p.mask[at]);
}

#[test]
fn b2_b4_a_pasted_xprv_is_masked_and_never_persisted() {
    let md = &schema::md::SCHEMA;
    // descriptor --key @0=xprv…
    let d = sub(md, "descriptor");
    let st = FormState::from_pairs(vec![
        ("--template", FlagValue::Text("wpkh(@0/<0;1>/*)".into())),
        (
            "--key",
            FlagValue::Text(format!("@0=[73c5da0a/84'/0'/0']{XPRV}")),
        ),
    ]);
    let (argv, mask) = assemble_argv_with_secret_mask(md, d, &st);
    let at = argv.iter().position(|t| t.contains(XPRV)).unwrap();
    assert!(mask[at], "a pasted xprv is masked in every display");
    let plan = channels::plan(md, d, &st, &|_: &str| None, "linux").unwrap();
    let at = plan.argv.iter().position(|t| t.contains(XPRV)).unwrap();
    assert!(
        plan.mask[at],
        "the Preview (the plan's argv) keeps the mask"
    );
    assert!(
        !redact_for_persistence(&st)
            .values
            .iter()
            .any(|(k, _)| k == "--key"),
        "never persisted"
    );
    // a public xpub is left alone
    let st = FormState::from_pairs(vec![
        ("--template", FlagValue::Text("wpkh(@0/<0;1>/*)".into())),
        ("--key", FlagValue::Text(format!("@0={X}"))),
    ]);
    let (argv, mask) = assemble_argv_with_secret_mask(md, d, &st);
    let at = argv.iter().position(|t| t.contains(X)).unwrap();
    assert!(!mask[at]);
    assert!(redact_for_persistence(&st)
        .values
        .iter()
        .any(|(k, _)| k == "--key"));
    // shape-key --descriptor and the decompose positional
    let s = sub(md, "shape-key");
    let st = FormState::from_pairs(vec![(
        "--descriptor",
        FlagValue::Text(format!("wpkh({XPRV}/<0;1>/*)")),
    )]);
    let (argv, mask) = assemble_argv_with_secret_mask(md, s, &st);
    assert!(mask[argv.iter().position(|t| t.contains(XPRV)).unwrap()]);
    let dc = sub(md, "decompose");
    let st = FormState::default().with_positionals([format!("wsh(multi(2,{X},{XPRV}))")]);
    let (argv, mask) = assemble_argv_with_secret_mask(md, dc, &st);
    assert!(mask[argv.iter().position(|t| t.contains(XPRV)).unwrap()]);
    // the same text in a field NOT listed is not masked by this rule
    let st = FormState::default().with_positionals([format!("wpkh({XPRV}/<0;1>/*)")]);
    let (_, mask) = assemble_argv_with_secret_mask(md, sub(md, "shape-key"), &st);
    assert!(!mask.iter().any(|&m| m));
}

// ── real binaries ─────────────────────────────────────────────────────────

fn run(schema: &'static Schema, subname: &str, st: &FormState) -> (Option<i32>, String, String) {
    let mut p = channels::plan_for_run(schema, sub(schema, subname), st)
        .unwrap_or_else(|r| panic!("{subname}: {r}"));
    p.argv[0] = secret_channels_common::bin_dir()
        .join(schema.cli_name)
        .to_string_lossy()
        .into_owned();
    let r = mnemonic_gui::runner::run_plan(&p).unwrap();
    (
        r.exit_code,
        String::from_utf8_lossy(&r.stdout).into_owned(),
        String::from_utf8_lossy(&r.stderr).into_owned(),
    )
}

#[test]
fn b_real_every_new_form_runs_at_exit_zero() {
    let md = &schema::md::SCHEMA;
    let desc = format!("wpkh([73c5da0a/84'/0'/0']{X}/<0;1>/*)");
    let cases: Vec<(&'static Schema, &str, FormState, &str)> = vec![
        (
            md,
            "compose",
            FormState::from_pairs(vec![
                ("--wrapper", FlagValue::Dropdown("wsh".into())),
                ("--path", FlagValue::Text("2of3".into())),
            ]),
            "sortedmulti(2,",
        ),
        (
            md,
            "shape-key",
            FormState::from_pairs(vec![("--descriptor", FlagValue::Text(desc.clone()))]),
            "wpkh(@0/<0;1>/*)",
        ),
        (
            md,
            "descriptor",
            FormState::from_pairs(vec![
                ("--template", FlagValue::Text("wpkh(@0/<0;1>/*)".into())),
                (
                    "--key",
                    FlagValue::Text(format!("@0=[73c5da0a/84'/0'/0']{X}")),
                ),
            ]),
            "wpkh([73c5da0a/84'/0'/0']",
        ),
        (
            md,
            "decompose",
            FormState::from_pairs(vec![("--emit", FlagValue::Dropdown("template".into()))])
                .with_positionals([desc.clone()]),
            "wpkh(@0/84'/0'/0'/<0;1>/*)",
        ),
    ];
    for (schema, name, st, want) in cases {
        let (code, out, err) = run(schema, name, &st);
        assert_eq!(code, Some(0), "{name}: {err}");
        assert!(out.contains(want), "{name}: {out}");
    }
    // ms hashlock from a 32-byte --hex (a secret: over its private channel)
    let mut st = FormState::from_pairs(vec![
        ("--kind", FlagValue::Dropdown("sha256".into())),
        ("--no-engraving-card", FlagValue::Boolean(true)),
    ]);
    st.secret_widgets.insert(
        "--hex".into(),
        vec![SecretLineEdit::from_text(&"11".repeat(32))],
    );
    let (code, out, err) = run(&schema::ms::SCHEMA, "hashlock", &st);
    assert_eq!(code, Some(0), "hashlock: {err}");
    assert!(!out.is_empty());
    // all kinds — lookup only: runs, and lists every kind on stderr
    let mut st = FormState::from_pairs(vec![
        (
            "--kind",
            FlagValue::Dropdown(schema::ms::HASHLOCK_KIND_ALL.into()),
        ),
        ("--no-engraving-card", FlagValue::Boolean(true)),
    ]);
    st.secret_widgets.insert(
        "--hex".into(),
        vec![SecretLineEdit::from_text(&"11".repeat(32))],
    );
    let (code, _, err) = run(&schema::ms::SCHEMA, "hashlock", &st);
    assert_eq!(code, Some(0), "{err}");
    assert!(
        err.contains("ripemd160") && err.contains("hash160"),
        "{err}"
    );
}
