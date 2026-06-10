//! Phase A.3 (v0.2 T-1): egui_kittest harness scaffold.
//!
//! Introduces the `egui_kittest::Harness` test harness for driving the
//! GUI's widget renderers in integration tests, complementing the
//! existing pure-logic tests (`argv_assembler*`, `conditional_visibility`,
//! `secrets`, etc.) which exercise the model layer without going through
//! egui rendering.
//!
//! Two cells (SPEC §6; Section A §2.4 Option B — assertion-only, no
//! snapshot `.png` files committed):
//!
//! 1. `cell_1_slot_editor_add_remove_writeback` — drives
//!    `slot_editor::render()` through the harness: add row → remove →
//!    add again; asserts the resulting `SlotState::to_slot_argv()`
//!    matches the expected byte-exact argv emission.
//! 2. `cell_2_conditional_visibility_toggle` — drives the
//!    `mnemonic export-wallet` form; mutates `FormState` to toggle
//!    `--template`; asserts `conditional::export_wallet()` returns the
//!    `FlagVisibility` map specified by SPEC §5 (mutual exclusion +
//!    runtime-pre-check Required).
//!
//! The originally-spec'd cell 3 (`cell_3_paste_warn_modal_trigger`) is
//! split into a separate file `tests/widget_secret.rs` landed in Phase
//! B.1 once `SecretLineEdit` exists (R1 C-1 fold — referencing a B.1
//! type from this A.3 file would block A.3's compile gate).

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;

use mnemonic_gui::form::conditional;
use mnemonic_gui::form::invocation::assemble_argv;
use mnemonic_gui::form::slot_editor::{self, SlotState};
use mnemonic_gui::schema::{self, FlagValue, FormState, Visibility};

fn subcommand(name: &str) -> &'static schema::SubcommandSchema {
    schema::mnemonic::SCHEMA
        .subcommands
        .iter()
        .find(|s| s.name == name)
        .unwrap_or_else(|| panic!("subcommand {} not in schema", name))
}

#[test]
fn cell_1_slot_editor_add_remove_writeback() {
    let initial = SlotState::default();
    let mut harness = Harness::new_ui_state(
        |ui, state| {
            slot_editor::render(ui, state, None);
        },
        initial,
    );

    // Initial state: zero rows; argv emission empty.
    assert!(
        harness.state().rows.is_empty(),
        "fresh SlotState should have no rows"
    );

    // Add a row. The "+ Add slot" label is rendered by slot_editor::render
    // at the bottom of the scroll area (slot_editor.rs:177).
    harness.get_by_label("+ Add slot").click();
    harness.run();
    assert_eq!(
        harness.state().rows.len(),
        1,
        "click on `+ Add slot` should append one row to the SlotState"
    );

    // The new row has default values: index = 0, subkey = SlotSubkey::Xpub
    // (hard-coded in SlotRow::default() at slot_editor.rs:83-90 — note
    // that this is NOT SlotSubkey::ALL[0] which is Phrase; the default
    // impl picks Xpub explicitly), value = "".

    // Remove the first row by clicking the ✕ button. Width-1 emoji
    // label; per the v0.1.2 dropdown-id fix convention, the button has
    // no shared-ID confusion. The label-find returns the first match
    // (there is only one ✕ since there is only one row).
    harness.get_by_label("✕").click();
    harness.run();
    assert!(
        harness.state().rows.is_empty(),
        "click on ✕ should remove the row"
    );

    // Add again. With one row added and value still empty,
    // SlotState::to_slot_argv() should emit no argv pairs (empty-value
    // rows are skipped per SPEC §6.7).
    harness.get_by_label("+ Add slot").click();
    harness.run();
    assert_eq!(harness.state().rows.len(), 1);

    let argv = harness.state().to_slot_argv();
    assert_eq!(
        argv,
        Vec::<String>::new(),
        "to_slot_argv() must emit no pairs when the only row has empty value (SPEC §6.7 empty-value omission)"
    );
}

#[test]
fn cell_2_conditional_visibility_toggle() {
    // Drive a FormState that mirrors the `mnemonic export-wallet` form.
    // The harness renders an unrelated probe widget (the export-wallet
    // form's per-flag widget rendering is exercised separately in v0.1
    // pure-logic tests); the focus here is harness-driven state
    // mutation followed by the conditional::export_wallet() visibility
    // query. The probe widget gives the harness something to render so
    // its frame-step actually runs.
    let initial = FormState::default();
    let mut harness = Harness::new_ui_state(
        |ui, state| {
            // Minimal probe: a button labeled "set-template-bip84" that,
            // when clicked, mutates the FormState to set --template.
            // A real form renderer is unnecessary for the visibility
            // query — the assertion is on the conditional fn output,
            // not on rendered geometry.
            if ui.button("set-template-bip84").clicked() {
                state.values.push((
                    "--template".to_string(),
                    FlagValue::Dropdown("bip84".to_string()),
                ));
            }
            if ui.button("set-descriptor").clicked() {
                state.values.push((
                    "--descriptor".to_string(),
                    FlagValue::Text("wpkh(@0/**)".to_string()),
                ));
            }
            if ui.button("clear-form").clicked() {
                state.values.clear();
            }
        },
        initial,
    );

    // Baseline: no template, no descriptor → conditional marks BOTH as
    // Required (export_wallet runtime pre-check, conditional.rs:120-126).
    let vis = conditional::export_wallet(harness.state());
    assert_eq!(
        vis_for(&vis, "--template"),
        Some(Visibility::Required),
        "with neither template nor descriptor, --template should be Required"
    );
    assert_eq!(
        vis_for(&vis, "--descriptor"),
        Some(Visibility::Required),
        "with neither template nor descriptor, --descriptor should be Required"
    );

    // Toggle template via the harness click.
    harness.get_by_label("set-template-bip84").click();
    harness.run();
    let vis = conditional::export_wallet(harness.state());
    assert_eq!(
        vis_for(&vis, "--descriptor"),
        Some(Visibility::Disabled),
        "with template set, --descriptor should be Disabled (mutual exclusion)"
    );
    assert_ne!(
        vis_for(&vis, "--template"),
        Some(Visibility::Required),
        "with template set, --template should no longer be Required"
    );

    // Clear and re-test the descriptor path.
    harness.get_by_label("clear-form").click();
    harness.run();
    harness.get_by_label("set-descriptor").click();
    harness.run();
    let vis = conditional::export_wallet(harness.state());
    assert_eq!(
        vis_for(&vis, "--template"),
        Some(Visibility::Disabled),
        "with descriptor set, --template should be Disabled (mutual exclusion)"
    );
}

fn vis_for(vis: &mnemonic_gui::schema::FlagVisibility, flag: &str) -> Option<Visibility> {
    // v0.6.0: Visibility no longer Copy (PinValue carries serde_json::Value).
    vis.iter().find(|(k, _)| *k == flag).map(|(_, v)| v.clone())
}

// ── Phase D.4 (v0.2): kittest cells 4+5 for representative new subcommands ───

#[test]
fn cell_4_ms_encode_xor_phrase_hex_via_harness() {
    // Drives the v0.2 D.2 `ms encode` --phrase XOR --hex conditional
    // through the harness. Synthetic probe pattern (cell 2 reuse):
    // mutate FormState via probe buttons, then query
    // `conditional::ms_encode()` for the SPEC §5 visibility map.
    use mnemonic_gui::form::conditional;
    use mnemonic_gui::schema::FormState;

    let initial = FormState::default();
    let mut harness = Harness::new_ui_state(
        |ui, state| {
            if ui.button("set-phrase").clicked() {
                state.values.push((
                    "--phrase".to_string(),
                    FlagValue::Text("abandon abandon ...".into()),
                ));
            }
            if ui.button("set-hex").clicked() {
                state.values.push((
                    "--hex".to_string(),
                    FlagValue::Text("00112233".into()),
                ));
            }
            if ui.button("clear-form").clicked() {
                state.values.clear();
            }
        },
        initial,
    );

    // Baseline: neither set → both Required.
    let vis = conditional::ms_encode(harness.state());
    assert_eq!(vis_for(&vis, "--phrase"), Some(Visibility::Required));
    assert_eq!(vis_for(&vis, "--hex"), Some(Visibility::Required));

    // Set --phrase → --hex Disabled.
    harness.get_by_label("set-phrase").click();
    harness.run();
    let vis = conditional::ms_encode(harness.state());
    assert_eq!(vis_for(&vis, "--hex"), Some(Visibility::Disabled));

    // Clear, set --hex → --phrase Disabled AND --language Hidden.
    harness.get_by_label("clear-form").click();
    harness.run();
    harness.get_by_label("set-hex").click();
    harness.run();
    let vis = conditional::ms_encode(harness.state());
    assert_eq!(vis_for(&vis, "--phrase"), Some(Visibility::Disabled));
    assert_eq!(vis_for(&vis, "--language"), Some(Visibility::Hidden));
}

// ── v0.3 (Phase 2.4): kittest cells for the 5 new mnemonic surfaces ────
//
// Pattern matches D.4 cells 4+5: inline probe-button closure mutates
// FormState.values on click; final assertion calls `assemble_argv`
// directly on `harness.state()` (no MnemonicGuiApp boot — main-mod
// state is private to integration tests; see widget_secret.rs:18-24).
// Probe values need not be cryptographically valid; cells inspect
// argv-assembly, not toolkit semantics (plan §3.2 P2.4 canned-value
// discipline note).

#[test]
fn cell_v0_3_slip39_split_argv_assembles() {
    let initial = FormState::default();
    let mut harness = Harness::new_ui_state(
        |ui, state| {
            if ui.button("set-from-phrase").clicked() {
                state.values.push((
                    "--from".into(),
                    FlagValue::NodeValueComposite {
                        node: "phrase".into(),
                        value: "abandon abandon ...".into(),
                    },
                ));
            }
            if ui.button("set-group-threshold").clicked() {
                state
                    .values
                    .push(("--group-threshold".into(), FlagValue::Number(1)));
            }
            if ui.button("set-group").clicked() {
                state
                    .values
                    .push(("--group".into(), FlagValue::Text("2,2".into())));
            }
        },
        initial,
    );

    harness.get_by_label("set-from-phrase").click();
    harness.run();
    harness.get_by_label("set-group-threshold").click();
    harness.run();
    harness.get_by_label("set-group").click();
    harness.run();

    let argv = assemble_argv(
        &schema::mnemonic::SCHEMA,
        subcommand("slip39-split"),
        harness.state(),
    );
    assert!(argv.contains(&"slip39-split".to_string()));
    assert!(argv.contains(&"--from".to_string()));
    assert!(argv.contains(&"phrase=abandon abandon ...".to_string()));
    assert!(argv.contains(&"--group-threshold".to_string()));
    assert!(argv.contains(&"--group".to_string()));
    assert!(argv.contains(&"2,2".to_string()));
}

#[test]
fn cell_v0_3_slip39_combine_argv_assembles() {
    // v0.31.1 (SPEC §5, R0-r1 I4 migration): slip39-combine `--share` is
    // secret + repeating + Text — its rows live as per-row
    // `SecretLineEdit`s in `state.secret_widgets` (the vec source), not
    // `state.values`. Contrast `cell_v0_3_seed_xor_combine_argv_assembles`
    // below, which stays values-routed UNCHANGED (NodeValueComposite —
    // the §2 counter-example pin).
    use mnemonic_gui::form::secret_widget::SecretLineEdit;
    let initial = FormState::default();
    let mut harness = Harness::new_ui_state(
        |ui, state: &mut FormState| {
            if ui.button("set-share-1").clicked() {
                state
                    .secret_widgets
                    .entry("--share".into())
                    .or_default()
                    .push(SecretLineEdit::from_text("share-1-words"));
            }
            if ui.button("set-share-2").clicked() {
                state
                    .secret_widgets
                    .entry("--share".into())
                    .or_default()
                    .push(SecretLineEdit::from_text("share-2-words"));
            }
        },
        initial,
    );

    harness.get_by_label("set-share-1").click();
    harness.run();
    harness.get_by_label("set-share-2").click();
    harness.run();

    let argv = assemble_argv(
        &schema::mnemonic::SCHEMA,
        subcommand("slip39-combine"),
        harness.state(),
    );
    assert!(argv.contains(&"slip39-combine".to_string()));
    // Two --share emissions in order.
    let share_indices: Vec<_> = argv
        .iter()
        .enumerate()
        .filter(|(_, s)| *s == "--share")
        .map(|(i, _)| i)
        .collect();
    assert_eq!(share_indices.len(), 2, "expected 2 --share argv tokens; got argv = {argv:?}");
    assert_eq!(argv[share_indices[0] + 1], "share-1-words");
    assert_eq!(argv[share_indices[1] + 1], "share-2-words");
}

#[test]
fn cell_v0_3_seed_xor_split_argv_assembles() {
    let initial = FormState::default();
    let mut harness = Harness::new_ui_state(
        |ui, state| {
            if ui.button("set-from-phrase").clicked() {
                state.values.push((
                    "--from".into(),
                    FlagValue::NodeValueComposite {
                        node: "phrase".into(),
                        value: "abandon abandon ...".into(),
                    },
                ));
            }
            if ui.button("set-shares-2").clicked() {
                state
                    .values
                    .push(("--shares".into(), FlagValue::Number(2)));
            }
        },
        initial,
    );

    harness.get_by_label("set-from-phrase").click();
    harness.run();
    harness.get_by_label("set-shares-2").click();
    harness.run();

    let argv = assemble_argv(
        &schema::mnemonic::SCHEMA,
        subcommand("seed-xor-split"),
        harness.state(),
    );
    assert!(argv.contains(&"seed-xor-split".to_string()));
    assert!(argv.contains(&"--from".to_string()));
    assert!(argv.contains(&"phrase=abandon abandon ...".to_string()));
    assert!(argv.contains(&"--shares".to_string()));
    assert!(argv.contains(&"2".to_string()));
}

#[test]
fn cell_v0_3_seed_xor_combine_argv_assembles() {
    let initial = FormState::default();
    let mut harness = Harness::new_ui_state(
        |ui, state| {
            if ui.button("set-share-1").clicked() {
                state.values.push((
                    "--share".into(),
                    FlagValue::NodeValueComposite {
                        node: "phrase".into(),
                        value: "share-one-words".into(),
                    },
                ));
            }
            if ui.button("set-share-2").clicked() {
                state.values.push((
                    "--share".into(),
                    FlagValue::NodeValueComposite {
                        node: "phrase".into(),
                        value: "share-two-words".into(),
                    },
                ));
            }
            if ui.button("set-shares-2").clicked() {
                state
                    .values
                    .push(("--shares".into(), FlagValue::Number(2)));
            }
        },
        initial,
    );

    harness.get_by_label("set-share-1").click();
    harness.run();
    harness.get_by_label("set-share-2").click();
    harness.run();
    harness.get_by_label("set-shares-2").click();
    harness.run();

    let argv = assemble_argv(
        &schema::mnemonic::SCHEMA,
        subcommand("seed-xor-combine"),
        harness.state(),
    );
    assert!(argv.contains(&"seed-xor-combine".to_string()));
    assert!(argv.contains(&"--shares".to_string()));
    let share_indices: Vec<_> = argv
        .iter()
        .enumerate()
        .filter(|(_, s)| *s == "--share")
        .map(|(i, _)| i)
        .collect();
    assert_eq!(share_indices.len(), 2);
    assert_eq!(argv[share_indices[0] + 1], "phrase=share-one-words");
    assert_eq!(argv[share_indices[1] + 1], "phrase=share-two-words");
}

#[test]
fn cell_v0_3_final_word_argv_assembles() {
    let initial = FormState::default();
    let mut harness = Harness::new_ui_state(
        |ui, state| {
            if ui.button("set-from-phrase").clicked() {
                state.values.push((
                    "--from".into(),
                    FlagValue::NodeValueComposite {
                        node: "phrase".into(),
                        value: "eleven words go here ...".into(),
                    },
                ));
            }
        },
        initial,
    );

    harness.get_by_label("set-from-phrase").click();
    harness.run();

    let argv = assemble_argv(
        &schema::mnemonic::SCHEMA,
        subcommand("final-word"),
        harness.state(),
    );
    assert!(argv.contains(&"final-word".to_string()));
    assert!(argv.contains(&"--from".to_string()));
    assert!(argv.contains(&"phrase=eleven words go here ...".to_string()));
}

#[test]
fn cell_5_md_encode_dropdown_value_inspect_via_harness() {
    // Drives the v0.2 D.3 `md encode` --context value-inspect
    // conditional through the harness. This is the first
    // dropdown-value-dependent conditional (D.1 finding #2) and the
    // first egui_kittest cell that exercises a Dropdown FlagValue.
    use mnemonic_gui::form::conditional;
    use mnemonic_gui::schema::FormState;

    let initial = FormState::default();
    let mut harness = Harness::new_ui_state(
        |ui, state| {
            if ui.button("set-from-policy").clicked() {
                state.values.push((
                    "--from-policy".to_string(),
                    FlagValue::Text("pk(@0)".into()),
                ));
            }
            if ui.button("set-context-tap").clicked() {
                state.values.push((
                    "--context".to_string(),
                    FlagValue::Dropdown("tap".into()),
                ));
            }
            if ui.button("set-context-segwitv0").clicked() {
                state.values.push((
                    "--context".to_string(),
                    FlagValue::Dropdown("segwitv0".into()),
                ));
            }
        },
        initial,
    );

    // Set --from-policy + --context=tap → --unspendable-key NOT
    // disabled (tap is the only context that supports unspendable
    // internal keys).
    harness.get_by_label("set-from-policy").click();
    harness.run();
    harness.get_by_label("set-context-tap").click();
    harness.run();
    let vis = conditional::md_encode(harness.state());
    assert_ne!(
        vis_for(&vis, "--unspendable-key"),
        Some(Visibility::Disabled),
        "--unspendable-key should not be Disabled when --context=tap"
    );

    // Change to --context=segwitv0 → --unspendable-key Disabled.
    // (state.values now has two --context entries; conditional sees
    // the LATER segwitv0 via dropdown_value's find iteration which
    // returns the first match — so we clear and re-set.)
    harness.state_mut().values.retain(|(k, _)| k != "--context");
    harness.get_by_label("set-context-segwitv0").click();
    harness.run();
    let vis = conditional::md_encode(harness.state());
    assert_eq!(
        vis_for(&vis, "--unspendable-key"),
        Some(Visibility::Disabled),
        "--unspendable-key should be Disabled when --context=segwitv0"
    );
}
