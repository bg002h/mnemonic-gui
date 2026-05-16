//! v0.16.0 P5 — default form-state smoke test.
//!
//! Mirrors `main.rs::default()`'s bundle-form seed and asserts the assembled
//! argv does NOT contain the v0.16.0 motivating-bug flags
//! (`--multisig-path-family` / `--threshold`) when the default template
//! (`bip84`, single-sig) is selected. Pre-v0.16.0 the seeded
//! `--multisig-path-family = bip87` value emitted into argv and triggered
//! the CLI's PATH_FAMILY_WITHOUT_MULTISIG byte-exact rejection.
//!
//! If `MNEMONIC_BIN` is set, also shells out to the toolkit binary and
//! asserts the bundle subcommand accepts the default argv (with a
//! canonical test-vector phrase supplied via `--slot @0.phrase=...`).
//! Skipped when the binary is not resolvable.

use mnemonic_gui::form::invocation::assemble_argv;
use mnemonic_gui::form::slot_editor::{SlotRow, SlotState, SlotSubkey};
use mnemonic_gui::schema::{self, FlagValue, FormState};

fn bundle_subcommand() -> &'static schema::SubcommandSchema {
    schema::mnemonic::SCHEMA
        .subcommands
        .iter()
        .find(|s| s.name == "bundle")
        .expect("bundle subcommand")
}

/// Build the FormState that `main.rs::default()` seeds for the bundle tab,
/// post-v0.16.0 P5 cleanup. Adding a canonical-test-vector phrase to
/// slot 0 so the smoke test exercises a full happy path.
fn default_bundle_form_state_with_phrase() -> FormState {
    FormState::from_pairs(vec![
        ("--network", FlagValue::Dropdown("mainnet".into())),
        ("--template", FlagValue::Dropdown("bip84".into())),
        ("--account", FlagValue::Number(0)),
    ])
    .with_slots(SlotState {
        rows: vec![SlotRow {
            index: 0,
            subkey: SlotSubkey::Phrase,
            value: "abandon abandon abandon abandon abandon abandon \
                    abandon abandon abandon abandon abandon about"
                .into(),
        }],
    })
}

#[test]
fn default_bundle_form_state_argv_omits_multisig_path_family() {
    let state = default_bundle_form_state_with_phrase();
    let argv = assemble_argv(&schema::mnemonic::SCHEMA, bundle_subcommand(), &state);
    assert!(
        !argv.iter().any(|a| a == "--multisig-path-family"),
        "default form-state argv MUST NOT contain --multisig-path-family \
         (was the v0.16.0 motivating bug); got {argv:?}"
    );
    assert!(
        !argv.iter().any(|a| a == "--threshold"),
        "default form-state argv MUST NOT contain --threshold; got {argv:?}"
    );
}

#[test]
fn default_bundle_form_state_argv_has_required_minimum() {
    let state = default_bundle_form_state_with_phrase();
    let argv = assemble_argv(&schema::mnemonic::SCHEMA, bundle_subcommand(), &state);
    // Sanity: argv contains argv[0]/[1] and the four required-ish flags.
    assert_eq!(&argv[0], "mnemonic");
    assert_eq!(&argv[1], "bundle");
    assert!(argv.iter().any(|a| a == "--network"));
    assert!(argv.iter().any(|a| a == "--template"));
    // Slot is emitted via SlotState; assert the slot=@0.phrase line is there.
    assert!(
        argv.iter().any(|a| a.starts_with("@0.phrase=")),
        "expected slot argv; got {argv:?}"
    );
}

#[test]
fn default_bundle_form_state_cli_accepts() {
    // Skip when MNEMONIC_BIN is unset (offline dev-laptop runs).
    let mnemonic_bin = match std::env::var("MNEMONIC_BIN") {
        Ok(v) => v,
        Err(_) => {
            eprintln!("MNEMONIC_BIN unset; skipping CLI exit-0 assertion");
            return;
        }
    };
    let state = default_bundle_form_state_with_phrase();
    let mut argv = assemble_argv(&schema::mnemonic::SCHEMA, bundle_subcommand(), &state);
    // Add --self-check so the CLI does not actually print 100+ lines of
    // cards; just runs the full pre-check + synthesis pipeline.
    argv.push("--self-check".into());
    let bin = std::path::PathBuf::from(&mnemonic_bin);
    let output = std::process::Command::new(&bin)
        .args(&argv[1..]) // skip argv[0] "mnemonic" since the binary IS mnemonic
        .output()
        .expect("MNEMONIC_BIN exec failed");
    assert!(
        output.status.success(),
        "default form-state argv must produce CLI exit 0; got status \
         {:?}\nstderr: {}\nargv: {argv:?}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
}
