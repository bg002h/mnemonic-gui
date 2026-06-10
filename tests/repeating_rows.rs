//! v0.30.0 SPEC §3 — generic repeating-row widget cells.
//!
//! The argv assembler has always emitted every matching `state.values` row
//! for a `repeating: true` flag (invocation.rs); pre-v0.30.0 the widget
//! layer rendered at most ONE row per flag name, capping every repeating
//! flag at a single value from the live form. v0.30.0 adds the multi-row
//! widget (`widget::render_repeating` behind `render_with_dispatch`):
//! header (label + `?` + required marker + "+ add") + one row per
//! `state.values` entry + per-row remove.
//!
//! Two layers of coverage (mirroring the repo's split):
//!   - state-level cells drive `FormState` + `assemble_argv` directly
//!     (row order, removal order, empty-row suppression);
//!   - UI-path kittest cells drive `widget::render_with_dispatch` through
//!     `egui_kittest::Harness` (the seed rule, "+ add" semantics, per-row
//!     remove, required-row respawn) — the seed rule is a RENDER-time
//!     behavior and cannot be pinned at the state level.

use eframe::egui;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;

use mnemonic_gui::app::CliTab;
use mnemonic_gui::form::invocation::assemble_argv;
use mnemonic_gui::form::widget;
use mnemonic_gui::schema::{self, FlagValue, FormState};

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

// ─── state-level cells ───────────────────────────────────────────────────

#[test]
fn cell_three_key_rows_emit_in_row_order_and_middle_remove_preserves_order() {
    // (SPEC §6 cell a) 3 --key rows → 3 occurrences in ROW order (argv
    // order is load-bearing for quorum order); remove the middle → 2,
    // order preserved.
    let mut state = FormState::from_pairs(vec![
        ("--key", FlagValue::Text("xpubAAA".into())),
        ("--key", FlagValue::Text("xpubBBB".into())),
        ("--key", FlagValue::Text("xpubCCC".into())),
        ("--threshold", FlagValue::Number(2)),
    ]);
    let argv = assemble_argv(&schema::mnemonic::SCHEMA, subcommand("build-descriptor"), &state);
    assert_eq!(
        occurrences(&argv, "--key"),
        ["xpubAAA", "xpubBBB", "xpubCCC"],
        "3 --key rows must emit in row order: {argv:?}"
    );

    // Remove the middle row (what the per-row ✕ does after the loop).
    let middle = state
        .values
        .iter()
        .position(|(k, v)| k == "--key" && *v == FlagValue::Text("xpubBBB".into()))
        .expect("middle row present");
    state.values.remove(middle);
    let argv = assemble_argv(&schema::mnemonic::SCHEMA, subcommand("build-descriptor"), &state);
    assert_eq!(
        occurrences(&argv, "--key"),
        ["xpubAAA", "xpubCCC"],
        "removing the middle row must preserve the order of the rest: {argv:?}"
    );
}

#[test]
fn cell_existing_repeating_flag_generalizes_restore_md1() {
    // (SPEC §6 cell b) the widget + assembler generalize over the
    // pre-existing repeating flags — restore --md1 (chunked wallet-policy
    // card) carries N rows the same way.
    let state = FormState::from_pairs(vec![
        ("--md1", FlagValue::Text("md1chunk1...".into())),
        ("--md1", FlagValue::Text("md1chunk2...".into())),
    ]);
    let argv = assemble_argv(&schema::mnemonic::SCHEMA, subcommand("restore"), &state);
    assert_eq!(
        occurrences(&argv, "--md1"),
        ["md1chunk1...", "md1chunk2..."],
        "restore --md1 rows must emit in row order: {argv:?}"
    );
}

#[test]
fn cell_added_untouched_allow_row_emits_nothing() {
    // (SPEC §6 cell c / R0-r1 I3) "+ add" seeds Dropdown("") — an
    // added-but-untouched --allow row must emit NOTHING (accidental
    // emission of an allowance is a funds-safety opt-out).
    let state = FormState::from_pairs(vec![(
        "--allow",
        FlagValue::Dropdown(String::new()),
    )]);
    let argv = assemble_argv(&schema::mnemonic::SCHEMA, subcommand("build-descriptor"), &state);
    assert!(
        !argv.contains(&"--allow".to_string()),
        "an added-untouched --allow row must not reach argv: {argv:?}"
    );
}

#[test]
fn cell_added_empty_key_row_emits_nothing() {
    // (SPEC §6 cell d) an added-then-left-empty Text row emits nothing
    // (emit_one skips empty Text).
    let state = FormState::from_pairs(vec![(
        "--key",
        FlagValue::Text(String::new()),
    )]);
    let argv = assemble_argv(&schema::mnemonic::SCHEMA, subcommand("build-descriptor"), &state);
    assert!(
        !argv.contains(&"--key".to_string()),
        "an empty --key row must not reach argv: {argv:?}"
    );
}

// ─── UI-path kittest cells ───────────────────────────────────────────────

/// Harness that renders every flag of `sub_name` through
/// `render_with_dispatch` — the same per-flag loop the live form runs
/// (visibility map omitted: every flag Visible, no disabled options).
fn full_form_harness(sub_name: &'static str) -> Harness<'static, FormState> {
    let sub = subcommand(sub_name);
    Harness::new_ui_state(
        move |ui, state: &mut FormState| {
            for flag in sub.flags {
                if flag.name == "--slot" && sub.allows_slots {
                    continue; // SlotEditor surface, not the flag widget.
                }
                widget::render_with_dispatch(ui, CliTab::Mnemonic, sub_name, flag, state, &[]);
            }
        },
        FormState::default(),
    )
}

#[test]
fn cell_build_descriptor_default_form_seeds_no_rows_and_no_archetype() {
    // (SPEC §6 cell e) the default build-descriptor form:
    //   - OPTIONAL repeating flags (--key/--recovery-key/--allow) seed
    //     ZERO rows (never auto-emit an allowance);
    //   - --archetype seeds the "" sentinel (default_value Some("")) and
    //     is suppressed from argv (R0-r1 C1).
    let mut harness = full_form_harness("build-descriptor");
    harness.run();

    let state = harness.state();
    for name in ["--key", "--recovery-key", "--allow"] {
        assert!(
            !state.values.iter().any(|(k, _)| k == name),
            "optional repeating {name} must seed NO rows in the default form"
        );
    }
    assert!(
        state
            .values
            .iter()
            .any(|(k, v)| k == "--archetype" && *v == FlagValue::Dropdown(String::new())),
        "--archetype must seed the \"\" UNSET sentinel"
    );

    let argv = assemble_argv(&schema::mnemonic::SCHEMA, subcommand("build-descriptor"), state);
    for name in ["--archetype", "--key", "--recovery-key", "--allow", "--spec"] {
        assert!(
            !argv.contains(&name.to_string()),
            "default build-descriptor form must not emit {name}: {argv:?}"
        );
    }
}

#[test]
fn cell_convert_default_form_still_emits_to_phrase() {
    // (SPEC §6 / R0-r1 C2 regression pin) convert --to is REQUIRED +
    // repeating: the per-frame seed rule must keep seeding one
    // Dropdown("phrase") row — the pre-v0.30.0 scalar-path behavior —
    // so the default convert form stays runnable.
    let flag = flag_named("convert", "--to");
    let mut harness = Harness::new_ui_state(
        |ui, state: &mut FormState| {
            widget::render_with_dispatch(ui, CliTab::Mnemonic, "convert", flag, state, &[]);
        },
        FormState::default(),
    );
    harness.run();

    let argv = assemble_argv(&schema::mnemonic::SCHEMA, subcommand("convert"), harness.state());
    assert_eq!(
        occurrences(&argv, "--to"),
        ["phrase"],
        "default convert form must still emit --to phrase: {argv:?}"
    );
}

#[test]
fn cell_required_repeating_last_row_remove_respawns_next_frame() {
    // R0-r2 M-C: removing the LAST row of a required repeating flag
    // respawns it on the next frame (the per-frame zero-rows-and-required
    // seed re-fires) — a required flag always shows >= 1 row. INTENDED.
    let flag = flag_named("convert", "--to");
    let mut harness = Harness::new_ui_state(
        |ui, state: &mut FormState| {
            widget::render_with_dispatch(ui, CliTab::Mnemonic, "convert", flag, state, &[]);
        },
        FormState::default(),
    );
    harness.run();
    assert_eq!(
        harness.state().values.iter().filter(|(k, _)| k == "--to").count(),
        1,
        "required --to seeds exactly one row"
    );

    harness.get_by_label("✕").click();
    harness.run(); // apply the remove, then the seed re-fires same/next frame
    assert_eq!(
        harness.state().values.iter().filter(|(k, _)| k == "--to").count(),
        1,
        "removing the last required row must respawn it (per-frame seed)"
    );
    assert!(
        harness
            .state()
            .values
            .iter()
            .any(|(k, v)| k == "--to" && *v == FlagValue::Dropdown("phrase".into())),
        "the respawned row carries the flag-aware default (phrase)"
    );
}

#[test]
fn cell_add_button_appends_empty_text_and_empty_dropdown_rows() {
    // (SPEC §3 add-row semantics) "+ add" on a Text repeating flag seeds
    // Text(""); on a Dropdown repeating flag seeds Dropdown("") — NOT
    // opts[0] (R0-r1 I3).
    let key = flag_named("build-descriptor", "--key");
    let allow = flag_named("build-descriptor", "--allow");
    let mut harness = Harness::new_ui_state(
        |ui, state: &mut FormState| {
            ui.push_id("key", |ui| {
                widget::render_with_dispatch(
                    ui,
                    CliTab::Mnemonic,
                    "build-descriptor",
                    key,
                    state,
                    &[],
                );
            });
            ui.push_id("allow", |ui| {
                widget::render_with_dispatch(
                    ui,
                    CliTab::Mnemonic,
                    "build-descriptor",
                    allow,
                    state,
                    &[],
                );
            });
        },
        FormState::default(),
    );
    harness.run();

    // Two "+ add" buttons (one per flag header), in render order.
    let add_buttons: Vec<_> = harness.get_all_by_label("+ add").collect();
    assert_eq!(add_buttons.len(), 2, "one + add per repeating-flag header");
    add_buttons[0].click();
    harness.run();
    let add_buttons: Vec<_> = harness.get_all_by_label("+ add").collect();
    add_buttons[1].click();
    harness.run();

    let state = harness.state();
    assert!(
        state
            .values
            .iter()
            .any(|(k, v)| k == "--key" && *v == FlagValue::Text(String::new())),
        "+ add on --key must append Text(\"\"): {:?}",
        state.values
    );
    assert!(
        state
            .values
            .iter()
            .any(|(k, v)| k == "--allow" && *v == FlagValue::Dropdown(String::new())),
        "+ add on --allow must append Dropdown(\"\"), NOT opts[0]: {:?}",
        state.values
    );

    // Neither freshly-added row reaches argv.
    let argv = assemble_argv(&schema::mnemonic::SCHEMA, subcommand("build-descriptor"), state);
    assert!(
        !argv.contains(&"--key".to_string()) && !argv.contains(&"--allow".to_string()),
        "freshly-added empty rows must not emit: {argv:?}"
    );
}

#[test]
fn cell_remove_middle_row_via_ui_preserves_order() {
    // UI-path twin of the state-level middle-remove cell: three --key rows
    // rendered, the SECOND ✕ clicked, row order preserved.
    let flag = flag_named("build-descriptor", "--key");
    let initial = FormState::from_pairs(vec![
        ("--key", FlagValue::Text("xpubAAA".into())),
        ("--key", FlagValue::Text("xpubBBB".into())),
        ("--key", FlagValue::Text("xpubCCC".into())),
    ]);
    let mut harness = Harness::new_ui_state(
        |ui, state: &mut FormState| {
            widget::render_with_dispatch(
                ui,
                CliTab::Mnemonic,
                "build-descriptor",
                flag,
                state,
                &[],
            );
        },
        initial,
    );
    harness.run();

    let removes: Vec<_> = harness.get_all_by_label("✕").collect();
    assert_eq!(removes.len(), 3, "one ✕ per row");
    removes[1].click();
    harness.run();

    let argv = assemble_argv(&schema::mnemonic::SCHEMA, subcommand("build-descriptor"), harness.state());
    assert_eq!(
        occurrences(&argv, "--key"),
        ["xpubAAA", "xpubCCC"],
        "clicking the middle ✕ must remove exactly that row: {argv:?}"
    );
}

#[test]
fn cell_archetype_unset_sentinel_displays_as_none_label() {
    // (R0-r2 I-1) the "" sentinel renders as the "(none)" display label.
    // The CLOSED ComboBox button is not label-queryable through accesskit
    // (egui's `ComboBox::from_id_salt` registers WidgetInfo with an empty
    // label — combo_box.rs:246 — and paints the selected text as a raw
    // galley), so the UI-path pin drives the POPUP: open the combo and
    // assert the "" option's selectable row carries the "(none)" display
    // text. Both display sites (selected_text + popup row) share the same
    // mapping in the Dropdown render arm; the stored value staying "" is
    // pinned by cell_build_descriptor_default_form_seeds_no_rows_and_no_archetype.
    let flag = flag_named("build-descriptor", "--archetype");
    let mut harness = Harness::new_ui_state(
        |ui, state: &mut FormState| {
            widget::render_with_dispatch(
                ui,
                CliTab::Mnemonic,
                "build-descriptor",
                flag,
                state,
                &[],
            );
        },
        FormState::default(),
    );
    harness.run();
    assert!(
        harness.query_by_label("(none)").is_none(),
        "popup closed: no \"(none)\" row yet"
    );
    harness
        .get_by_role(egui::accesskit::Role::ComboBox)
        .click();
    harness.run();
    assert!(
        harness.query_by_label("(none)").is_some(),
        "the \"\" sentinel option must display as \"(none)\" in the popup, \
         not a ~4px empty sliver"
    );
    assert!(
        harness.query_by_label("decaying-multisig").is_some(),
        "the real archetype rows render alongside the \"(none)\" escape row"
    );
}
