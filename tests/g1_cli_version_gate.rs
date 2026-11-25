//! Bug hunt 2026-11-25 G1: no runtime check of the spawned CLI's version.
//!
//! The planner sends a secret as `@env:MNEMONIC_GUI_S<i>` (or `-`, or
//! `/dev/fd/N`) on Linux, measured against the pinned binaries only. An older
//! CLI first on `$PATH` can take that reference literally as the secret. The
//! gate (`cli_version::gate_secret_run`, called by the app's
//! `spawn_and_capture` before every run) refuses a secret-bearing run unless
//! the binary reports a version at least the pinned one, and never alters a
//! plan it admits.

use mnemonic_gui::cli_version::{gate_secret_run, pinned_version_for};
use mnemonic_gui::form::channels;
use mnemonic_gui::schema::{self, FlagValue, FormState, SubcommandSchema};

const MS1: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";

fn restore() -> &'static SubcommandSchema {
    schema::mnemonic::SCHEMA
        .subcommands
        .iter()
        .find(|s| s.name == "restore")
        .expect("restore subcommand")
}

fn env_ref_plan() -> channels::RunPlan {
    let st = FormState::from_pairs(vec![("--from", FlagValue::Text("ms1=@env:SEED".into()))]);
    let env = |k: &str| (k == "SEED").then(|| MS1.to_string());
    let p =
        channels::plan(&schema::mnemonic::SCHEMA, restore(), &st, &env, "linux").expect("planned");
    assert!(
        p.argv.iter().any(|t| t == "ms1=@env:MNEMONIC_GUI_S0"),
        "precondition: the private path sends an @env: reference: {:?}",
        p.argv
    );
    p
}

#[test]
fn g1_env_reference_to_an_older_cli_is_refused() {
    let p = env_ref_plan();
    let mut probed = None;
    let r = gate_secret_run(&p, |bin| {
        probed = Some(bin.to_string());
        Ok("mnemonic 0.25.0\n".to_string())
    });
    assert_eq!(probed.as_deref(), Some("mnemonic"));
    let err = r.expect_err("an older CLI must be refused");
    assert!(err.contains("older than the pinned mnemonic "), "{err}");
}

#[test]
fn g1_a_different_program_named_like_the_cli_is_refused() {
    let p = env_ref_plan();
    assert!(gate_secret_run(&p, |_| Ok("something-else 3.0.0\n".into())).is_err());
    assert!(gate_secret_run(&p, |_| Err("exit 2".into())).is_err());
}

#[test]
fn g1_pinned_or_newer_cli_passes_and_the_plan_is_untouched() {
    let p = env_ref_plan();
    let before = (
        p.argv.clone(),
        p.mask.clone(),
        p.env[0].1.as_str().to_string(),
    );
    let pinned = pinned_version_for("mnemonic").unwrap().to_string();
    assert_eq!(gate_secret_run(&p, |_| Ok(format!("{pinned}\n"))), Ok(()));
    assert_eq!(
        gate_secret_run(&p, |_| Ok("mnemonic 9.0.0\n".into())),
        Ok(())
    );
    let after = (
        p.argv.clone(),
        p.mask.clone(),
        p.env[0].1.as_str().to_string(),
    );
    assert_eq!(before, after);
    assert_eq!(
        after.2, MS1,
        "the secret reaching a correct CLI is unchanged"
    );
}

#[test]
fn g1_public_run_is_never_probed() {
    let st = FormState::from_pairs(vec![(
        "--from",
        FlagValue::Text("xpub=xpub661MyMwAqRbcFtXgS5sYJABqqG9YLmC4Q1Rdap9gSE8NqtwybGhePY2gZ29ESFjqJoCu1Rupje8YtGqsefD265TMg7usUDFdp6W1EGMcet".into()),
    )]);
    let none = |_: &str| None;
    let p = channels::plan(&schema::mnemonic::SCHEMA, restore(), &st, &none, "linux").unwrap();
    assert!(!p.has_secrets());
    assert_eq!(gate_secret_run(&p, |_| panic!("must not probe")), Ok(()));
}

/// The app wires the gate into `spawn_and_capture`, the single choke point
/// every Run (confirmed or not) passes through, before `runner::run_plan`.
#[test]
fn g1_spawn_and_capture_calls_the_gate_before_spawning() {
    let src = include_str!("../src/app_window.rs");
    let f = src
        .find("fn spawn_and_capture(")
        .expect("spawn_and_capture exists");
    let body = &src[f..];
    let gate = body
        .find("cli_version::gate_secret_run(")
        .expect("spawn_and_capture must call the version gate");
    let spawn = body.find("runner::run_plan(").expect("spawn site");
    assert!(gate < spawn, "the gate must run before the spawn");
}
