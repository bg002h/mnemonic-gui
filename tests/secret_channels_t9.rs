//! T9 — `ms hashlock` phrase fidelity (DESIGN §A9, §B5), against the PINNED
//! `ms` (`MNEMONIC_BIN` required — never a skip).
//!
//! The phrase is byte-verbatim (no trim): through the GUI's own form, planner
//! and runner, `"  pad  "` must give a different digest from `"pad"`, and the
//! planned run must equal the oracle — the known bytes on argv with
//! `--allow-argv-secret` (`ms hashlock --hashlock-phrase` has no CLI `@env:`).
//!
//! `"pad\n"` and `"pad\r\n"` end in a newline: since fold 5 the design REFUSES
//! a target ending in CR or LF on every path (§A0.1, `value-ends-in-newline`)
//! and "a refusal is always safe" (§A9 T3′). Those legs assert the refusal;
//! they run nothing.

mod secret_channels_common;

use mnemonic_gui::form::channels;
use mnemonic_gui::form::secret_widget::SecretLineEdit;
use mnemonic_gui::schema::{self, FlagValue, FormState};
use secret_channels_common::bin_dir;

fn hashlock() -> &'static schema::SubcommandSchema {
    schema::ms::SCHEMA
        .subcommands
        .iter()
        .find(|s| s.name == "hashlock")
        .expect("ms hashlock is mirrored")
}

fn form(phrase: &str) -> FormState {
    let mut st = FormState::from_pairs(vec![
        ("--kind", FlagValue::Dropdown("sha256".into())),
        ("--no-engraving-card", FlagValue::Boolean(true)),
    ]);
    st.secret_widgets.insert(
        "--hashlock-phrase".into(),
        vec![SecretLineEdit::from_text(phrase)],
    );
    st
}

fn ms() -> String {
    bin_dir().join("ms").to_string_lossy().into_owned()
}

/// The GUI's Run: plan on Linux (the private path) and run it.
fn gui_run(phrase: &str) -> Result<(Option<i32>, String), &'static str> {
    let mut plan = channels::plan(
        &schema::ms::SCHEMA,
        hashlock(),
        &form(phrase),
        &|_: &str| None,
        "linux",
    )
    .map_err(|r| r.code)?;
    assert!(
        !plan.argv.iter().any(|t| t.contains("pad")),
        "the phrase on argv: {:?}",
        plan.argv
    );
    assert!(
        plan.argv.iter().any(|t| t == "--hashlock-phrase-stdin"),
        "{:?}",
        plan.argv
    );
    plan.argv[0] = ms();
    let r = mnemonic_gui::runner::run_plan(&plan).expect("runner");
    Ok((r.exit_code, String::from_utf8_lossy(&r.stdout).into_owned()))
}

/// The oracle: the known bytes on argv.
fn oracle(phrase: &str) -> (Option<i32>, String) {
    let o = std::process::Command::new(ms())
        .args([
            "hashlock",
            "--allow-argv-secret",
            "--kind",
            "sha256",
            "--no-engraving-card",
            "--hashlock-phrase",
            phrase,
        ])
        .output()
        .unwrap();
    (
        o.status.code(),
        String::from_utf8_lossy(&o.stdout).into_owned(),
    )
}

#[test]
fn t9_hashlock_phrase_is_byte_verbatim_through_the_gui() {
    let padded = gui_run("  pad  ").expect("plans");
    let bare = gui_run("pad").expect("plans");
    assert_eq!(padded.0, Some(0));
    assert_eq!(bare.0, Some(0));
    assert_ne!(
        padded.1, bare.1,
        "\"  pad  \" must not hash as \"pad\" (no trim)"
    );
    assert_eq!(padded, oracle("  pad  "), "planned run == oracle");
    assert_eq!(bare, oracle("pad"), "planned run == oracle");
    for v in ["pad\n", "pad\r\n"] {
        assert_eq!(
            gui_run(v).err(),
            Some("value-ends-in-newline"),
            "{v:?} is refused (§A0.1), never sent"
        );
    }
}
