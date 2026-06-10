//! v0.31.0 (archetype forms SPEC §6) — archetype param-form cells.
//!
//! Three layers (the repo's established split, `tests/repeating_rows.rs`
//! is the v0.30.0 reference):
//!   - state-level argv cells: per-archetype "only the declared params
//!     emit" (the §5 conditional Hidden projection through
//!     `assemble_argv`) + the kofn happy path;
//!   - resolve cells: `NumberMax::FromRowCount` row-count semantics
//!     (ALL rows incl. empty — R0-r1 I3b; `.max(1)` floor — R0-r2 m1);
//!   - UI-path kittest cells driving the LIB seam
//!     (`archetype_form::active_archetype` + `archetype_form::render` —
//!     R0-r1 I2: main.rs only dispatches): mode switch, data preservation
//!     round-trip, the C1 surplus-rows arm, mode-independent visibility,
//!     the hex-digest hint, and the threshold clamp-on-render.

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;

use mnemonic_gui::app::CliTab;
use mnemonic_gui::form::archetype_form;
use mnemonic_gui::form::invocation::assemble_argv;
use mnemonic_gui::form::widget;
use mnemonic_gui::schema::archetypes::{find, ARCHETYPE_PARAM_FLAGS, ARCHETYPE_SPECS};
use mnemonic_gui::schema::{self, FlagValue, FormState, NumberMax};

const HEX64: &str = "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08";

fn subcommand(name: &str) -> &'static schema::SubcommandSchema {
    schema::mnemonic::SCHEMA
        .subcommands
        .iter()
        .find(|s| s.name == name)
        .unwrap_or_else(|| panic!("subcommand {} not in schema", name))
}

fn build_descriptor_argv(state: &FormState) -> Vec<String> {
    assemble_argv(&schema::mnemonic::SCHEMA, subcommand("build-descriptor"), state)
}

/// Occurrences of `flag` (with its following value token) in argv, in order.
fn occurrences(argv: &[String], flag: &str) -> Vec<String> {
    argv.windows(2)
        .filter(|w| w[0] == flag)
        .map(|w| w[1].clone())
        .collect()
}

/// FormState with rows for ALL 9 archetype params + two mode-independent
/// flags, plus the selected archetype. NO `--spec` row: a populated
/// `--spec` fires the OTHER direction of the v0.30.0 mutex (`--spec`
/// present → `--archetype` Disabled → neither emits), a state the live
/// form can't reach (the `--spec` widget is Disabled the moment an
/// archetype is selected, and suppressed entirely in archetype mode).
fn state_with_all_nine_params(archetype: &str) -> FormState {
    FormState::from_pairs(vec![
        ("--archetype", FlagValue::Dropdown(archetype.into())),
        ("--key", FlagValue::Text("xpubKEY1".into())),
        ("--key", FlagValue::Text("xpubKEY2".into())),
        ("--threshold", FlagValue::Number(2)),
        ("--recovery-key", FlagValue::Text("xpubREC1".into())),
        ("--recovery-key", FlagValue::Text("xpubREC2".into())),
        ("--recovery-threshold", FlagValue::Number(2)),
        ("--final-key", FlagValue::Text("xpubFINAL".into())),
        ("--older", FlagValue::Number(144)),
        ("--recovery-older", FlagValue::Number(288)),
        ("--after", FlagValue::Number(800_000)),
        ("--hash", FlagValue::Text(HEX64.into())),
        ("--network", FlagValue::Dropdown("testnet".into())),
        ("--json", FlagValue::Boolean(true)),
    ])
}

// ─── per-archetype argv cells (SPEC §5/§6) ───────────────────────────────

/// With rows synthesized for all 9 params, selecting `id` must emit ONLY
/// its declared params (+ the mode-independent flags); `--spec` is
/// Disabled by the unchanged mutex and never emits.
fn assert_only_declared_params_emit(id: &str) {
    let spec = find(id).unwrap_or_else(|| panic!("unknown archetype {id}"));
    let state = state_with_all_nine_params(id);
    let argv = build_descriptor_argv(&state);
    for &name in ARCHETYPE_PARAM_FLAGS {
        let declared = spec.params.iter().any(|p| p.flag == name);
        assert_eq!(
            argv.contains(&name.to_string()),
            declared,
            "{id}: param {name} must emit iff declared (declared={declared}): {argv:?}",
        );
    }
    assert!(
        !argv.contains(&"--spec".to_string()),
        "{id}: --spec must not emit in archetype mode: {argv:?}",
    );
    assert_eq!(
        occurrences(&argv, "--archetype"),
        [id],
        "{id}: the selected archetype must emit: {argv:?}",
    );
    // Mode-independent flags keep emitting.
    assert_eq!(occurrences(&argv, "--network"), ["testnet"], "{id}: {argv:?}");
    assert!(argv.contains(&"--json".to_string()), "{id}: {argv:?}");
}

#[test]
fn cell_decaying_multisig_emits_only_declared_params() {
    assert_only_declared_params_emit("decaying-multisig");
}

#[test]
fn cell_hashlock_gated_emits_only_declared_params() {
    assert_only_declared_params_emit("hashlock-gated");
}

#[test]
fn cell_kofn_recovery_emits_only_declared_params() {
    assert_only_declared_params_emit("kofn-recovery");
}

#[test]
fn cell_simple_timelocked_inheritance_emits_only_declared_params() {
    assert_only_declared_params_emit("simple-timelocked-inheritance");
}

#[test]
fn cell_tiered_recovery_emits_only_declared_params() {
    assert_only_declared_params_emit("tiered-recovery");
}

#[test]
fn cell_kofn_happy_path_argv_shape() {
    // SPEC §6: the kofn happy path end-to-end — state → exact argv (flag
    // emission order = schema declaration order; row order = quorum order).
    let state = FormState::from_pairs(vec![
        ("--archetype", FlagValue::Dropdown("kofn-recovery".into())),
        ("--key", FlagValue::Text("xpubA".into())),
        ("--key", FlagValue::Text("xpubB".into())),
        ("--threshold", FlagValue::Number(2)),
        ("--recovery-key", FlagValue::Text("xpubR".into())),
        ("--older", FlagValue::Number(52_560)),
    ]);
    let argv = build_descriptor_argv(&state);
    assert_eq!(
        argv,
        [
            "mnemonic",
            "build-descriptor",
            "--archetype",
            "kofn-recovery",
            "--key",
            "xpubA",
            "--key",
            "xpubB",
            "--threshold",
            "2",
            "--recovery-key",
            "xpubR",
            "--older",
            "52560",
        ],
        "kofn happy-path argv shape",
    );
}

// ─── NumberMax::FromRowCount resolve cells (SPEC §4/§6) ─────────────────

#[test]
fn cell_from_row_count_counts_all_rows_floors_at_one() {
    let max = NumberMax::FromRowCount("--key");
    // 3 rows INCLUDING an empty one (R0-r1 I3b: ALL rows count — an empty
    // row emits nothing, so max can exceed the emitted key count; the CLI
    // gate validates the residual).
    let mut state = FormState::from_pairs(vec![
        ("--key", FlagValue::Text("xpubA".into())),
        ("--key", FlagValue::Text("xpubB".into())),
        ("--key", FlagValue::Text(String::new())),
    ]);
    assert_eq!(max.resolve(&state), 3, "3 rows (incl. empty) → max 3");

    // Remove a row → 2.
    state.values.pop();
    assert_eq!(max.resolve(&state), 2, "2 rows → max 2");

    // ZERO rows → the m1 `.max(1)` floor (degenerate range min..=1 stays
    // valid — the FromSlotCount discipline).
    state.values.retain(|(k, _)| k != "--key");
    assert_eq!(max.resolve(&state), 1, "zero rows → max 1 (floor)");

    // Rows of OTHER flags don't count.
    state
        .values
        .push(("--recovery-key".to_string(), FlagValue::Text("xpubR".into())));
    assert_eq!(max.resolve(&state), 1, "other flags' rows must not count");
}

// ─── UI-path kittest cells (the LIB seam — R0-r1 I2) ─────────────────────

/// Harness mirroring main.rs's dispatch: the generic per-flag loop with the
/// archetype-mode name-set `continue`, plus `archetype_form::render` under
/// the `--archetype` dropdown. The form logic under test is entirely the
/// library's (`active_archetype` / `suppressed_in_archetype_mode` /
/// `render`); this closure is dispatch-only, like main.rs.
fn dispatch_form_harness(initial: FormState) -> Harness<'static, FormState> {
    let sub = subcommand("build-descriptor");
    Harness::new_ui_state(
        move |ui, state: &mut FormState| {
            let spec = archetype_form::active_archetype(state);
            for flag in sub.flags {
                if spec.is_some()
                    && archetype_form::suppressed_in_archetype_mode(flag.name)
                {
                    continue;
                }
                widget::render_with_dispatch(
                    ui,
                    CliTab::Mnemonic,
                    "build-descriptor",
                    flag,
                    state,
                    &[],
                );
                if flag.name == "--archetype" {
                    if let Some(spec) = spec {
                        archetype_form::render(
                            ui,
                            CliTab::Mnemonic,
                            "build-descriptor",
                            spec,
                            state,
                        );
                    }
                }
            }
        },
        initial,
    )
}

/// Set the stored `--archetype` Dropdown row to `id` (what selecting a row
/// in the combobox popup does; the popup-interaction path itself is pinned
/// by `repeating_rows::cell_archetype_unset_sentinel_displays_as_none_label`).
fn select_archetype(state: &mut FormState, id: &str) {
    if let Some(row) = state.values.iter_mut().find(|(k, _)| k == "--archetype") {
        row.1 = FlagValue::Dropdown(id.to_string());
    } else {
        state
            .values
            .push(("--archetype".to_string(), FlagValue::Dropdown(id.to_string())));
    }
}

#[test]
fn cell_mode_switch_kofn_renders_param_form_and_round_trips() {
    // SPEC §6 kittest: select kofn-recovery → the param form renders
    // (4 params, summary line, `(min 2)` on --key); deselect → the generic
    // form returns; entered --key rows survive the archetype round-trip
    // (the §3 data-preservation claim).
    let kofn_summary = find("kofn-recovery").unwrap().summary;
    let mut harness = dispatch_form_harness(FormState::default());
    harness.run();

    // Generic mode: --spec + all params render; no summary line.
    assert!(harness.query_by_label("--spec").is_some(), "generic: --spec");
    assert!(harness.query_by_label("--hash").is_some(), "generic: --hash");
    assert!(
        harness.query_by_label(kofn_summary).is_none(),
        "generic: no summary line"
    );

    // Enter two --key rows, then select kofn-recovery.
    harness
        .state_mut()
        .values
        .push(("--key".to_string(), FlagValue::Text("xpubA".into())));
    harness
        .state_mut()
        .values
        .push(("--key".to_string(), FlagValue::Text("xpubB".into())));
    select_archetype(harness.state_mut(), "kofn-recovery");
    harness.run();

    // Archetype mode: summary + the 4 declared params; the 5 undeclared
    // params + --spec are gone; `(min 2)` annotates --key.
    assert!(
        harness.query_by_label(kofn_summary).is_some(),
        "kofn: summary line renders under the dropdown"
    );
    for declared in ["--key", "--threshold", "--recovery-key", "--older"] {
        assert!(
            harness.query_by_label(declared).is_some(),
            "kofn: declared param {declared} must render"
        );
    }
    for absent in ["--spec", "--hash", "--final-key", "--recovery-older", "--after", "--recovery-threshold"] {
        assert!(
            harness.query_by_label(absent).is_none(),
            "kofn: {absent} must NOT render in archetype mode"
        );
    }
    assert!(
        harness.query_by_label("(min 2)").is_some(),
        "kofn: --key carries the (min 2) annotation"
    );

    // Deselect ("(none)" → stored ""): the generic form returns and the
    // entered --key rows survived the round-trip.
    select_archetype(harness.state_mut(), "");
    harness.run();
    assert!(
        harness.query_by_label(kofn_summary).is_none(),
        "deselect: summary gone"
    );
    assert!(
        harness.query_by_label("--spec").is_some(),
        "deselect: generic --spec returns"
    );
    assert!(
        harness.query_by_label("--hash").is_some(),
        "deselect: generic --hash returns"
    );
    let keys: Vec<_> = harness
        .state()
        .values
        .iter()
        .filter(|(k, v)| k == "--key" && matches!(v, FlagValue::Text(s) if !s.is_empty()))
        .collect();
    assert_eq!(
        keys.len(),
        2,
        "--key rows must survive the archetype round-trip: {:?}",
        harness.state().values
    );
}

#[test]
fn cell_c1_switch_surplus_recovery_keys_render_with_add_suppressed_and_emit() {
    // R0-r1 C1 switch cell: 2 --recovery-key rows entered under
    // tiered-recovery → switch to kofn-recovery (archetype-NON-repeatable
    // --recovery-key) → BOTH rows render through the repeating widget with
    // "+ add" suppressed and an `(exactly 1)` annotation, and argv carries
    // both until the user removes one — what emits is what renders.
    let initial = FormState::from_pairs(vec![
        ("--archetype", FlagValue::Dropdown("tiered-recovery".into())),
        ("--recovery-key", FlagValue::Text("xpubR1".into())),
        ("--recovery-key", FlagValue::Text("xpubR2".into())),
    ]);
    let mut harness = dispatch_form_harness(initial);
    harness.run();
    // Under tiered-recovery, --recovery-key is repeatable: (min 2) + add.
    assert!(
        harness.query_by_label("(exactly 1)").is_none(),
        "tiered: --recovery-key is repeatable, no (exactly 1)"
    );

    select_archetype(harness.state_mut(), "kofn-recovery");
    harness.run();

    assert!(
        harness.query_by_label("(exactly 1)").is_some(),
        "kofn with surplus rows: the (exactly 1) annotation renders"
    );
    // "+ add" census: --key (repeatable, 1) + --allow (mode-independent
    // repeating, 1) = 2; --recovery-key's add is SUPPRESSED (would be 3).
    let add_count = harness.query_all_by_label("+ add").count();
    assert_eq!(
        add_count, 2,
        "kofn with surplus recovery rows: --recovery-key '+ add' suppressed"
    );
    // Both surplus rows still emit (render==argv).
    let argv = build_descriptor_argv(harness.state());
    assert_eq!(
        occurrences(&argv, "--recovery-key"),
        ["xpubR1", "xpubR2"],
        "both surplus rows emit until removed: {argv:?}"
    );

    // Remove one row via its ✕: rows with a remove button are --key's
    // seeded empty row (required repeating seeds one) then the two
    // --recovery-key rows — index 1 is xpubR1.
    let removes: Vec<_> = harness.query_all_by_label("✕").collect();
    assert_eq!(removes.len(), 3, "one ✕ per repeating row (1 key + 2 recovery)");
    removes[1].click();
    drop(removes);
    harness.run();

    let argv = build_descriptor_argv(harness.state());
    assert_eq!(
        occurrences(&argv, "--recovery-key"),
        ["xpubR2"],
        "after removal exactly one recovery key emits: {argv:?}"
    );
    // Back to ≤1 row: the scalar single-row arm takes over, annotation gone.
    assert!(
        harness.query_by_label("(exactly 1)").is_none(),
        "≤1 row: scalar arm, no (exactly 1) annotation"
    );
}

#[test]
fn cell_mode_independent_flags_render_in_archetype_mode() {
    // R0-r1 M5a: --allow + --emit-spec (+ --format/--network/--json +
    // --spec-schema + --no-auto-repair) keep their generic widgets in
    // archetype mode.
    let initial = FormState::from_pairs(vec![(
        "--archetype",
        FlagValue::Dropdown("kofn-recovery".into()),
    )]);
    let mut harness = dispatch_form_harness(initial);
    harness.run();
    for name in [
        "--spec-schema",
        "--archetype",
        "--emit-spec",
        "--allow",
        "--format",
        "--network",
        "--json",
        "--no-auto-repair",
    ] {
        assert!(
            harness.query_by_label(name).is_some(),
            "mode-independent {name} must render in archetype mode"
        );
    }
}

#[test]
fn cell_threshold_clamps_to_live_key_row_count_on_render() {
    // SPEC §6 clamp cell (render leg): an over-max stored --threshold
    // clamps on the next render — the bespoke Number widget's max is
    // NumberMax::FromRowCount("--key") and egui DragValue clamps the
    // existing value to range by default.
    let initial = FormState::from_pairs(vec![
        ("--archetype", FlagValue::Dropdown("kofn-recovery".into())),
        ("--key", FlagValue::Text("xpubA".into())),
        ("--key", FlagValue::Text("xpubB".into())),
        ("--threshold", FlagValue::Number(5)),
    ]);
    let mut harness = dispatch_form_harness(initial);
    harness.run();
    harness.run();
    let threshold = harness
        .state()
        .values
        .iter()
        .find_map(|(k, v)| match (k.as_str(), v) {
            ("--threshold", FlagValue::Number(n)) => Some(*n),
            _ => None,
        })
        .expect("--threshold row present");
    assert_eq!(
        threshold, 2,
        "an over-max stored threshold (5) must clamp to the live --key \
         row count (2) on render"
    );
}

#[test]
fn cell_hex_digest_warning_renders_only_when_non_empty_and_invalid() {
    // SPEC §4 hex_digest arm: a NON-blocking "expected 64 hex chars"
    // warning label when the value is non-empty and invalid; absent for
    // empty and for a valid 64-hex digest.
    const WARNING: &str = "⚠ expected 64 hex chars";
    let initial = FormState::from_pairs(vec![
        ("--archetype", FlagValue::Dropdown("hashlock-gated".into())),
        ("--hash", FlagValue::Text(String::new())),
    ]);
    let mut harness = dispatch_form_harness(initial);
    harness.run();
    assert!(
        harness.query_by_label(WARNING).is_none(),
        "empty --hash: no warning"
    );

    if let Some(row) = harness
        .state_mut()
        .values
        .iter_mut()
        .find(|(k, _)| k == "--hash")
    {
        row.1 = FlagValue::Text("not-hex".into());
    }
    harness.run();
    assert!(
        harness.query_by_label(WARNING).is_some(),
        "invalid --hash: the non-blocking warning renders"
    );

    if let Some(row) = harness
        .state_mut()
        .values
        .iter_mut()
        .find(|(k, _)| k == "--hash")
    {
        row.1 = FlagValue::Text(HEX64.into());
    }
    harness.run();
    assert!(
        harness.query_by_label(WARNING).is_none(),
        "valid 64-hex --hash: no warning"
    );
}

// ─── pairing + suppression-set sanity (integration-visible twins of the
//     in-module units; ARCHETYPE_SPECS + paired_key_flag are pub) ────────

#[test]
fn every_archetype_declaring_a_threshold_declares_the_paired_key() {
    for spec in ARCHETYPE_SPECS {
        for p in spec.params.iter().filter(|p| p.kind == "threshold") {
            let paired = archetype_form::paired_key_flag(p.flag)
                .unwrap_or_else(|| panic!("{}: {} unpaired", spec.id, p.flag));
            assert!(
                spec.params.iter().any(|q| q.flag == paired && q.kind == "key"),
                "{} declares {} but not its paired key {}",
                spec.id,
                p.flag,
                paired,
            );
        }
    }
}
