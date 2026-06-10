//! v0.31.1 (repeating-secrets SPEC §5) — per-row secret widget cells.
//!
//! THE bug (FOLLOWUP `repeating-secret-flags-never-reach-argv`): secret +
//! repeating + Text flags (`--ms1` on verify-bundle/import-wallet, `--share`
//! on slip39-combine/ms-shares-combine) rendered ONE `SecretLineEdit` into
//! `state.secret_widgets` while the assembler's repeating-secret arm read
//! rows from `state.values` — a source the widget never wrote — so LIVE
//! forms emitted NOTHING for these flags. The pre-v0.31.1 cells masked the
//! bug by synthesizing `state.values` entries directly (assembler-half
//! only). The cells here drive the REAL widget (`render_with_dispatch`)
//! through `egui_kittest` and pin the render→assemble seam the bug lived in.
//!
//! Typing mechanism (R0-r1 I2): kittest `Node::focus()` + `type_text()`
//! into the masked TextEdits — first in-repo use; masking is display-only,
//! so the routed `egui::Event::Text` lands in the focused row's buffer.
//! (The `SecretLineEdit::from_text` fallback was NOT needed.) NOTE: egui
//! registers password TextEdits as `Role::PasswordInput`, not
//! `Role::TextInput` — query by that role.

use eframe::egui;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;

use mnemonic_gui::app::CliTab;
use mnemonic_gui::form::invocation::assemble_argv;
use mnemonic_gui::form::widget;
use mnemonic_gui::persistence::redact_for_persistence;
use mnemonic_gui::schema::{self, FlagValue, FormState};
use mnemonic_gui::secrets::{schema_secret_flag_names, SECRET_FLAG_NAMES};

fn subcommand(name: &str) -> &'static schema::SubcommandSchema {
    schema::mnemonic::SCHEMA
        .subcommands
        .iter()
        .find(|s| s.name == name)
        .unwrap_or_else(|| panic!("subcommand {} not in schema", name))
}

fn flag_named(sub_name: &str, flag_name: &str) -> &'static schema::FlagSchema {
    subcommand(sub_name)
        .flags
        .iter()
        .find(|f| f.name == flag_name)
        .unwrap_or_else(|| panic!("{} has no flag {}", sub_name, flag_name))
}

/// Occurrences of `flag` (with its following value token) in argv, in order.
fn occurrences(argv: &[String], flag: &str) -> Vec<String> {
    argv.windows(2)
        .filter(|w| w[0] == flag)
        .map(|w| w[1].clone())
        .collect()
}

/// Harness rendering exactly ONE flag through the real
/// `render_with_dispatch` — the same dispatch the live form runs.
fn single_flag_harness(
    sub_name: &'static str,
    flag_name: &'static str,
) -> Harness<'static, FormState> {
    let flag = flag_named(sub_name, flag_name);
    Harness::new_ui_state(
        move |ui, state: &mut FormState| {
            widget::render_with_dispatch(ui, CliTab::Mnemonic, sub_name, flag, state, &[]);
        },
        FormState::default(),
    )
}

// ─── THE bug cell: live-path emission (previously impossible) ────────────

#[test]
fn cell_live_path_import_wallet_two_typed_ms1_rows_reach_argv_in_row_order() {
    // Drive the REAL widget for import-wallet --ms1 (optional + repeating
    // + secret): two rows added via the header's "+ add", text typed into
    // each masked TextEdit via kittest focus()+type_text(). Pre-v0.31.1
    // this exact flow emitted NOTHING (the live bug).
    let mut harness = single_flag_harness("import-wallet", "--ms1");
    harness.run();

    // Optional repeating secret: the default form seeds ZERO rows.
    assert!(
        harness
            .query_by_role(egui::accesskit::Role::PasswordInput)
            .is_none(),
        "optional --ms1 must seed no rows in the default form"
    );

    // Add two rows.
    harness.get_by_label("+ add").click();
    harness.run();
    harness.get_by_label("+ add").click();
    harness.run();

    // Type into row 0, then row 1 (re-query per frame; type_text routes
    // focus + egui::Event::Text into the masked TextEdit).
    {
        let rows: Vec<_> = harness
            .get_all_by_role(egui::accesskit::Role::PasswordInput)
            .collect();
        assert_eq!(rows.len(), 2, "two secret rows after two + add clicks");
        rows[0].type_text("ms1-row-one");
    }
    harness.run();
    {
        let rows: Vec<_> = harness
            .get_all_by_role(egui::accesskit::Role::PasswordInput)
            .collect();
        rows[1].type_text("ms1-row-two");
    }
    harness.run();
    harness.run(); // settle: buffer write-back happens at frame end

    let state = harness.state();
    // Secrets never enter state.values (the inverted fix direction).
    assert!(
        !state.values.iter().any(|(k, _)| k == "--ms1"),
        "secret rows must NOT enter state.values: {:?}",
        state.values
    );

    let argv = assemble_argv(
        &schema::mnemonic::SCHEMA,
        subcommand("import-wallet"),
        state,
    );
    assert_eq!(
        occurrences(&argv, "--ms1"),
        ["ms1-row-one", "ms1-row-two"],
        "both typed --ms1 rows must reach argv in row order: {argv:?}"
    );
}

#[test]
fn cell_live_path_slip39_share_required_seed_typed_reaches_argv() {
    // Twin for slip39-combine --share (REQUIRED + repeating + secret): the
    // seeded row appears WITHOUT clicking "+ add" (the per-frame
    // required-zero-rows seed), and typing into it reaches argv.
    let mut harness = single_flag_harness("slip39-combine", "--share");
    harness.run();
    harness.run(); // seed lands during frame 1; row renders frame 2

    {
        let rows: Vec<_> = harness
            .get_all_by_role(egui::accesskit::Role::PasswordInput)
            .collect();
        assert_eq!(
            rows.len(),
            1,
            "required --share must seed exactly one row without + add"
        );
        rows[0].type_text("share-words-typed");
    }
    harness.run();
    harness.run();

    let argv = assemble_argv(
        &schema::mnemonic::SCHEMA,
        subcommand("slip39-combine"),
        harness.state(),
    );
    assert_eq!(
        occurrences(&argv, "--share"),
        ["share-words-typed"],
        "the typed required --share row must reach argv: {argv:?}"
    );
}

// ─── Required-seed respawn ────────────────────────────────────────────────

#[test]
fn cell_required_share_last_row_remove_respawns_next_frame() {
    // Removing the LAST row of a required repeating secret flag respawns
    // it next frame (the per-frame seed re-fires) — intended (SPEC §1).
    let mut harness = single_flag_harness("slip39-combine", "--share");
    harness.run();
    harness.run();
    assert_eq!(
        harness
            .state()
            .secret_widgets
            .get("--share")
            .map(|rows| rows.len()),
        Some(1),
        "required --share seeds one row"
    );

    harness.get_by_label("✕").click();
    harness.run(); // apply the remove; the seed re-fires same/next frame
    harness.run();
    assert_eq!(
        harness
            .state()
            .secret_widgets
            .get("--share")
            .map(|rows| rows.len()),
        Some(1),
        "removing the last required row must respawn it (per-frame seed)"
    );
}

// ─── Empty-row suppression ────────────────────────────────────────────────

#[test]
fn cell_added_blank_secret_rows_emit_nothing() {
    // An added-but-blank row is inert (assembler-side guard): with one
    // filled row and one blank row, only the filled one emits; with all
    // rows blank, nothing emits.
    use mnemonic_gui::form::secret_widget::SecretLineEdit;

    let mut state = FormState::default();
    state.secret_widgets.insert(
        "--ms1".into(),
        vec![
            SecretLineEdit::from_text("ms1-filled"),
            SecretLineEdit::new(),
        ],
    );
    let argv = assemble_argv(
        &schema::mnemonic::SCHEMA,
        subcommand("import-wallet"),
        &state,
    );
    assert_eq!(
        occurrences(&argv, "--ms1"),
        ["ms1-filled"],
        "only the filled row emits; the blank row is inert: {argv:?}"
    );

    let mut all_blank = FormState::default();
    all_blank
        .secret_widgets
        .insert("--ms1".into(), vec![SecretLineEdit::new(), SecretLineEdit::new()]);
    let argv = assemble_argv(
        &schema::mnemonic::SCHEMA,
        subcommand("import-wallet"),
        &all_blank,
    );
    assert!(
        !argv.contains(&"--ms1".to_string()),
        "all-blank rows must emit nothing: {argv:?}"
    );
}

// ─── Dead path: values-synthesized repeating Text secrets ────────────────

#[test]
fn cell_values_synthesized_repeating_text_secret_emits_nothing_and_redacts() {
    // SPEC §2: the Text arm of the secret branch `continue`s
    // UNCONDITIONALLY — a values-synthesized repeating Text-secret entry
    // (e.g. a crafted/stale state.json, or the pre-v0.31.1 test idiom) is
    // a DEAD path for emission, and SPEC §3's name-level net redacts it
    // at persist.
    let mut state = FormState::default();
    state
        .values
        .push(("--ms1".into(), FlagValue::Text("BOGUS-from-values".into())));
    state
        .values
        .push(("--json".into(), FlagValue::Boolean(true)));

    let argv = assemble_argv(
        &schema::mnemonic::SCHEMA,
        subcommand("import-wallet"),
        &state,
    );
    assert!(
        !argv.iter().any(|a| a == "--ms1" || a == "BOGUS-from-values"),
        "a values-synthesized Text-secret entry must emit NOTHING: {argv:?}"
    );
    assert!(
        argv.contains(&"--json".to_string()),
        "non-secret values keep emitting: {argv:?}"
    );

    // ... and the persistence redaction drops it (belt-and-suspenders).
    let redacted = redact_for_persistence(&state);
    assert!(
        !redacted.values.iter().any(|(k, _)| k == "--ms1"),
        "the secret-named values entry must be redacted at persist: {:?}",
        redacted.values
    );
    assert!(
        redacted.values.iter().any(|(k, _)| k == "--json"),
        "non-secret values survive redaction"
    );
}

// ─── SPEC §3 drift tests: schema_secret_flag_names ───────────────────────

#[test]
fn drift_union_covers_known_secret_names_and_is_never_empty() {
    // R0-r1 M4: union ⊇ {--ms1, --share} ∪ SECRET_FLAG_NAMES — an emptied
    // union fails loud.
    let union = schema_secret_flag_names();
    for name in ["--ms1", "--share"] {
        assert!(
            union.contains(name),
            "schema_secret_flag_names() must contain {name}"
        );
    }
    for name in SECRET_FLAG_NAMES {
        assert!(
            union.contains(name),
            "schema_secret_flag_names() must contain hand-maintained {name}"
        );
    }
}

#[test]
fn drift_every_schema_secret_true_flag_redacts_a_dummy_values_entry() {
    // SPEC §3 drift test, FIELD-extracted (R0-r3 m-NEW2: never a text grep
    // — comment lines in mnemonic.rs contain the literal `secret: true`).
    // For every flag declared `secret: true` in ANY of the 4 schemas:
    //   (a) its name is in the union;
    //   (b) a dummy values entry under that name is dropped at redact.
    let schemas: [&schema::Schema; 4] = [
        &schema::mnemonic::SCHEMA,
        &schema::md::SCHEMA,
        &schema::ms::SCHEMA,
        &schema::mk::SCHEMA,
    ];
    let union = schema_secret_flag_names();
    let mut checked = 0usize;
    for s in schemas {
        for sub in s.subcommands {
            for flag in sub.flags {
                if !flag.secret {
                    continue;
                }
                checked += 1;
                assert!(
                    union.contains(flag.name),
                    "{} {} {} is secret:true but missing from the union",
                    s.cli_name,
                    sub.name,
                    flag.name
                );
                let mut state = FormState::default();
                state
                    .values
                    .push((flag.name.to_string(), FlagValue::Text("dummy-secret".into())));
                let redacted = redact_for_persistence(&state);
                assert!(
                    redacted.values.is_empty(),
                    "dummy values entry under secret flag {} ({} {}) must be \
                     redacted at persist; got {:?}",
                    flag.name,
                    s.cli_name,
                    sub.name,
                    redacted.values
                );
            }
        }
    }
    assert!(
        checked > 0,
        "the field-extraction census found no secret:true flags — vacuous"
    );
}

// ─── Scalar regression (per-row vec, 1 element) ──────────────────────────

#[test]
fn cell_scalar_passphrase_typed_via_live_widget_still_emits() {
    // SPEC §1: scalar (non-repeating) secrets keep byte-identical behavior
    // — one widget = vec[0]. Drive the live widget for bundle --passphrase
    // and assert the typed value emits exactly once.
    let mut harness = single_flag_harness("bundle", "--passphrase");
    harness.run();

    harness
        .get_by_role(egui::accesskit::Role::PasswordInput)
        .type_text("hunter2");
    harness.run();
    harness.run();

    let argv = assemble_argv(&schema::mnemonic::SCHEMA, subcommand("bundle"), harness.state());
    assert_eq!(
        occurrences(&argv, "--passphrase"),
        ["hunter2"],
        "scalar secret behavior must be unchanged: {argv:?}"
    );
}
