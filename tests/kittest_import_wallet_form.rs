//! v0.11.0 (toolkit v0.26.0): kittest cells for the `mnemonic import-wallet`
//! form surface. Mirrors the `cell_v0_3_*_argv_assembles` pattern from
//! `tests/widget_interaction.rs`: an `egui_kittest::Harness` drives a probe-
//! button closure that mutates `FormState`; the final assertion calls
//! `assemble_argv` on the harness state. Probe values need not be
//! cryptographically valid — the cells assert argv assembly, not toolkit
//! semantics.
//!
//! Coverage (6 cells):
//!   1. `cell_import_wallet_in_subcommands_set` — schema sanity-check: the
//!      new SubcommandSchema entry is present and discoverable by `name`.
//!   2. `cell_import_wallet_blob_path_argv` — `--blob /path/to/blob`
//!      assembles when `FormState.values["--blob"]` is `Path(...)`.
//!   3. `cell_import_wallet_blob_stdio_sentinel_argv` — `--blob -` emits
//!      the `-` token verbatim (Path with stdio_sentinel: true).
//!   4. `cell_import_wallet_repeating_ms1_argv` — two `--ms1` rows in
//!      `state.secret_widgets` (the v0.31.1 per-row vec source) produce two
//!      `--ms1 <value>` pairs in argv (positional cosigner-index ordering).
//!   5. `cell_import_wallet_select_descriptor_default_suppressed` — the
//!      schema-declared `default_value: "all"` suppresses `--select-descriptor all`
//!      from argv (D33 default-suppression).
//!   6. `cell_import_wallet_format_dropdown_argv` — picking
//!      `--format bsms` emits `--format bsms` in argv.
//!   7. `cell_import_wallet_slot_phrase_argv` — `--slot @0.phrase=<v>`
//!      via SlotEditor produces the per-slot argv pair.
//!   8. `cell_import_wallet_env_sentinel_literal_emission` — when the user
//!      types the `@env:VAR` sentinel verbatim into `--ms1`, the literal
//!      sentinel is forwarded to argv. Toolkit-side resolution does the
//!      `std::env::var` lookup; the GUI is sentinel-transparent.
//!
//! ### Env-var seed channel — current state + FOLLOWUP gap
//!
//! The toolkit (`crates/mnemonic-toolkit/src/cmd/import_wallet.rs:127-135`)
//! resolves `@env:<VAR>` sentinels on `--ms1` + secret-bearing `--slot`
//! subkeys at parse time. The GUI's argv assembler emits the value the
//! user typed into the secret widget VERBATIM into argv today; if the user
//! typed `@env:MY_SEED`, the toolkit will resolve it. If the user typed
//! a literal BIP-39 phrase, the literal phrase appears in argv (subject to
//! the v0.12.0+ FOLLOWUP `run-confirm-modal-argv-redaction` per the memory
//! note `feedback_run_confirm_modal_renders_argv_verbatim`).
//!
//! Argv-leak-protection via an env-var-channel injection (GUI rewrites
//! user-typed seeds to `@env:MNEMONIC_MS1_N` + sets `MNEMONIC_MS1_N` in
//! the spawned child env) is FILED as a v0.12.0 FOLLOWUP at
//! `design/FOLLOWUPS.md::gui-import-wallet-env-var-secret-channel` — the
//! v0.11.0 lockstep cycle ships the schema + literal emission path only.

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;

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

// ─── Cell 1: schema entry present ───────────────────────────────────────

#[test]
fn cell_import_wallet_in_subcommands_set() {
    let sub = schema::mnemonic::SCHEMA
        .subcommands
        .iter()
        .find(|s| s.name == "import-wallet");
    assert!(
        sub.is_some(),
        "import-wallet must appear in mnemonic SUBCOMMANDS"
    );
    let sub = sub.unwrap();
    assert!(sub.allows_slots, "import-wallet must opt into the slot editor");
    let flag_names: Vec<&str> = sub.flags.iter().map(|f| f.name).collect();
    // Spot-check the v0.26.0 surface.
    for required in [
        "--blob",
        "--format",
        "--select-descriptor",
        "--ms1",
        "--slot",
        "--json",
        "--no-auto-repair",
    ] {
        assert!(
            flag_names.contains(&required),
            "import-wallet missing required flag `{required}`; flags = {flag_names:?}"
        );
    }
}

// ─── Cell 2: `--blob <path>` argv ──────────────────────────────────────

#[test]
fn cell_import_wallet_blob_path_argv() {
    let initial = FormState::default();
    let mut harness = Harness::new_ui_state(
        |ui, state: &mut FormState| {
            if ui.button("set-blob-path").clicked() {
                state.values.push((
                    "--blob".into(),
                    FlagValue::Path("/tmp/wallet.bsms".into()),
                ));
            }
        },
        initial,
    );
    harness.get_by_label("set-blob-path").click();
    harness.run();

    let argv = assemble_argv(
        &schema::mnemonic::SCHEMA,
        subcommand("import-wallet"),
        harness.state(),
    );
    assert_eq!(
        argv,
        vec![
            "mnemonic",
            "import-wallet",
            "--blob",
            "/tmp/wallet.bsms",
        ]
    );
}

// ─── Cell 3: `--blob -` stdio sentinel ─────────────────────────────────

#[test]
fn cell_import_wallet_blob_stdio_sentinel_argv() {
    // Schema-declared `stdio_sentinel: true` lets `-` flow through to argv
    // verbatim. (No default_value: Some("-") suppression applies — the
    // schema declares `default_value: None` for --blob, so the `-` is
    // emitted as-typed.)
    let mut state = FormState::default();
    state
        .values
        .push(("--blob".into(), FlagValue::Path("-".into())));

    let argv = assemble_argv(
        &schema::mnemonic::SCHEMA,
        subcommand("import-wallet"),
        &state,
    );
    assert_eq!(
        argv,
        vec!["mnemonic", "import-wallet", "--blob", "-"]
    );
}

// ─── Cell 4: repeating `--ms1` (literal seed emission) ─────────────────

#[test]
fn cell_import_wallet_repeating_ms1_argv() {
    // `--ms1` is `repeating: true + secret: true`. v0.31.1 (SPEC §5,
    // R0-r1 I4 migration): repeating secrets live as per-row
    // `SecretLineEdit`s in `state.secret_widgets[flag.name]` (the vec
    // source) — the v0.3 values-routing this cell used to synthesize was
    // a dead source the widget never wrote. The literal value flows to
    // argv verbatim — see module-level doc for the env-var-channel
    // FOLLOWUP that closes the argv-leak gap.
    use mnemonic_gui::form::secret_widget::SecretLineEdit;
    let initial = FormState::default();
    let mut harness = Harness::new_ui_state(
        |ui, state: &mut FormState| {
            if ui.button("set-blob").clicked() {
                state.values.push((
                    "--blob".into(),
                    FlagValue::Path("/tmp/wallet.bsms".into()),
                ));
            }
            if ui.button("set-ms1-0").clicked() {
                state
                    .secret_widgets
                    .entry("--ms1".into())
                    .or_default()
                    .push(SecretLineEdit::from_text("ms1-cosigner-0"));
            }
            if ui.button("set-ms1-1").clicked() {
                state
                    .secret_widgets
                    .entry("--ms1".into())
                    .or_default()
                    .push(SecretLineEdit::from_text("ms1-cosigner-1"));
            }
        },
        initial,
    );
    harness.get_by_label("set-blob").click();
    harness.run();
    harness.get_by_label("set-ms1-0").click();
    harness.run();
    harness.get_by_label("set-ms1-1").click();
    harness.run();

    let argv = assemble_argv(
        &schema::mnemonic::SCHEMA,
        subcommand("import-wallet"),
        harness.state(),
    );

    // Two `--ms1` pairs in form-state-insert order (which is positional
    // cosigner-index order per SPEC §8.3).
    let ms1_indices: Vec<_> = argv
        .iter()
        .enumerate()
        .filter(|(_, s)| *s == "--ms1")
        .map(|(i, _)| i)
        .collect();
    assert_eq!(
        ms1_indices.len(),
        2,
        "expected 2 --ms1 argv tokens; got argv = {argv:?}"
    );
    assert_eq!(argv[ms1_indices[0] + 1], "ms1-cosigner-0");
    assert_eq!(argv[ms1_indices[1] + 1], "ms1-cosigner-1");
}

// ─── Cell 5: `--select-descriptor` default suppression ─────────────────

#[test]
fn cell_import_wallet_select_descriptor_default_suppressed() {
    // The schema declares `default_value: Some("all")` for
    // --select-descriptor. D33 default-suppression drops it from argv when
    // the user-typed value equals the default.
    let mut state = FormState::default();
    state.values.push((
        "--blob".into(),
        FlagValue::Path("/tmp/wallet.json".into()),
    ));
    state.values.push((
        "--select-descriptor".into(),
        FlagValue::Text("all".into()),
    ));

    let argv = assemble_argv(
        &schema::mnemonic::SCHEMA,
        subcommand("import-wallet"),
        &state,
    );
    assert!(
        !argv.iter().any(|s| s == "--select-descriptor"),
        "--select-descriptor all is the schema default — must be suppressed; got argv = {argv:?}"
    );

    // Non-default value emits normally.
    let mut state2 = FormState::default();
    state2.values.push((
        "--blob".into(),
        FlagValue::Path("/tmp/wallet.json".into()),
    ));
    state2.values.push((
        "--select-descriptor".into(),
        FlagValue::Text("active-receive".into()),
    ));
    let argv2 = assemble_argv(
        &schema::mnemonic::SCHEMA,
        subcommand("import-wallet"),
        &state2,
    );
    let idx = argv2.iter().position(|s| s == "--select-descriptor")
        .expect("non-default --select-descriptor must emit");
    assert_eq!(argv2[idx + 1], "active-receive");
}

// ─── Cell 6: `--format` dropdown ───────────────────────────────────────

#[test]
fn cell_import_wallet_format_dropdown_argv() {
    let mut state = FormState::default();
    state.values.push((
        "--blob".into(),
        FlagValue::Path("/tmp/wallet.bsms".into()),
    ));
    state
        .values
        .push(("--format".into(), FlagValue::Dropdown("bsms".into())));

    let argv = assemble_argv(
        &schema::mnemonic::SCHEMA,
        subcommand("import-wallet"),
        &state,
    );
    let idx = argv
        .iter()
        .position(|s| s == "--format")
        .expect("--format must appear in argv when set");
    assert_eq!(argv[idx + 1], "bsms");
}

// ─── Cell 7: `--slot @<i>.phrase=<v>` via SlotEditor ───────────────────

#[test]
fn cell_import_wallet_slot_phrase_argv() {
    // SlotEditor emits `--slot @<index>.<subkey>=<value>` pairs in
    // ascending-index order. Toolkit v0.26.0 only accepts the `phrase`
    // subkey on import-wallet; the GUI is permissive (allows the user to
    // pick any subkey via the SlotEditor) and the CLI rejection surfaces
    // in the output pane if the user picks the wrong one.
    let mut slots = SlotState::default();
    slots.rows.push(SlotRow {
        index: 0,
        subkey: SlotSubkey::Phrase,
        value: "abandon abandon abandon abandon abandon about".into(),
    });
    let state = FormState::default().with_slots(slots);

    let argv = assemble_argv(
        &schema::mnemonic::SCHEMA,
        subcommand("import-wallet"),
        &state,
    );
    let idx = argv
        .iter()
        .position(|s| s == "--slot")
        .expect("--slot must appear in argv");
    assert_eq!(
        argv[idx + 1],
        "@0.phrase=abandon abandon abandon abandon abandon about"
    );
}

// ─── Cell 8: `@env:VAR` sentinel literal pass-through ──────────────────

#[test]
fn cell_import_wallet_env_sentinel_literal_emission() {
    // Toolkit (`import_wallet.rs:127-135`) resolves `@env:<VAR>` on `--ms1`
    // at parse time. The GUI emits the value VERBATIM — so a user who
    // types `@env:MNEMONIC_MS1_0` into the --ms1 widget produces an argv
    // with `--ms1 @env:MNEMONIC_MS1_0`, and the toolkit substitutes the
    // env-var contents at run time. This cell pins that literal-pass-
    // through contract: any change to argv-rewriting (e.g., the v0.12.0
    // env-var-channel FOLLOWUP) must explicitly opt into a behavior break
    // here.
    //
    // v0.31.1 (SPEC §5, R0-r1 I4 migration): the `@env:` sentinel rides a
    // secret ROW — the vec source in `state.secret_widgets`, not a
    // synthesized `state.values` entry.
    use mnemonic_gui::form::secret_widget::SecretLineEdit;
    let mut state = FormState::default();
    state.values.push((
        "--blob".into(),
        FlagValue::Path("/tmp/wallet.bsms".into()),
    ));
    state.secret_widgets.insert(
        "--ms1".into(),
        vec![SecretLineEdit::from_text("@env:MNEMONIC_MS1_0")],
    );

    let argv = assemble_argv(
        &schema::mnemonic::SCHEMA,
        subcommand("import-wallet"),
        &state,
    );
    let idx = argv
        .iter()
        .position(|s| s == "--ms1")
        .expect("--ms1 must appear in argv");
    assert_eq!(
        argv[idx + 1],
        "@env:MNEMONIC_MS1_0",
        "@env: sentinel must flow verbatim to argv (toolkit-side resolution)"
    );
}
