//! v0.16.0 P4 — SPEC §6.10 drift gate.
//!
//! Shells out to `<MNEMONIC_BIN> gui-schema`, parses the v2
//! `conditional_rules` projection, synthesizes an exemplar `FormState`
//! satisfying each rule's predicate, invokes the corresponding hand-coded
//! `SubcommandSchema.conditional` fn, and asserts the returned
//! `FlagVisibility` map contains the rule's declared `(effect.flag,
//! effect.visibility)`.
//!
//! Failure messages cite the rule's `rationale` and `spec_ref` so future
//! drift surfaces the exact divergence and its SPEC reference.
//!
//! Skipped (returns early) when `MNEMONIC_BIN` is unset.

use mnemonic_gui::form::slot_editor::SlotRow;
use mnemonic_gui::schema::{
    self, FlagValue, FormState, SubcommandSchema, TaggedOrIndexedValue,
    Visibility,
};
use mnemonic_gui::schema_check::{
    parse_gui_schema_conditional_rules, ConditionalRule, Effect, Predicate,
    VisibilityProjection,
};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::Command;

fn mnemonic_bin() -> Option<PathBuf> {
    std::env::var("MNEMONIC_BIN")
        .ok()
        .map(PathBuf::from)
        .or_else(|| Some(PathBuf::from("mnemonic")))
}

fn gui_schema_json() -> String {
    let bin = mnemonic_bin().expect("MNEMONIC_BIN or PATH lookup");
    let out = Command::new(&bin)
        .arg("gui-schema")
        .output()
        .expect("failed to spawn `mnemonic gui-schema`");
    assert!(
        out.status.success(),
        "`mnemonic gui-schema` exited non-zero: {:?}\nstderr: {}",
        out.status,
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).expect("gui-schema stdout must be UTF-8")
}

fn subcommand_named(name: &str) -> Option<&'static SubcommandSchema> {
    schema::mnemonic::SCHEMA
        .subcommands
        .iter()
        .find(|s| s.name == name)
}

/// Translate the JSON-side VisibilityProjection to the GUI-side Visibility.
/// v0.6.0 (schema v3): added PinValue arm.
/// v0.7.0 (schema v4): added DisableOptions arm.
fn vis_to_visibility(v: VisibilityProjection) -> Visibility {
    match v {
        VisibilityProjection::Hidden => Visibility::Hidden,
        VisibilityProjection::Disabled => Visibility::Disabled,
        VisibilityProjection::Required => Visibility::Required,
        VisibilityProjection::PinValue { value } => Visibility::PinValue { value },
        VisibilityProjection::DisableOptions { values } => {
            Visibility::DisableOptions { values }
        }
    }
}

/// Synthesize a FormState satisfying the given predicate, layered on top
/// of `base`. Returns the new state.
fn synthesize_satisfying(predicate: &Predicate, base: FormState) -> FormState {
    match predicate {
        Predicate::FlagPresent { flag } => {
            // Pick a FlagValue that satisfies `has_value` for any flag kind.
            // The schema-level kind dictates which variant; for FormState
            // purposes, any non-empty string-typed variant works because
            // `flag_value_is_present` returns true for non-empty Text /
            // Dropdown / Path / Composite (and unconditionally true for
            // Number / Boolean / Range / Timestamp / TaggedOrIndexed).
            // `Text` is universally accepted by `has_value` though the
            // argv-emit path may reject it for non-text-kinded flags;
            // we don't emit here, just check visibility.
            push_or_replace(base, flag, FlagValue::Text("exemplar".into()))
        }
        Predicate::DropdownValueIn { flag, values } => {
            let first = values
                .first()
                .cloned()
                .expect("dropdown_value_in.values must be non-empty");
            push_or_replace(base, flag, FlagValue::Dropdown(first))
        }
        Predicate::CompositeNodeIs { flag, node } => push_or_replace(
            base,
            flag,
            FlagValue::NodeValueComposite {
                node: node.clone(),
                value: "exemplar".into(),
            },
        ),
        Predicate::PositionalPresent { index } => {
            let mut s = base;
            while s.positionals.len() <= *index {
                s.positionals.push(String::new());
            }
            s.positionals[*index] = "exemplar".into();
            s
        }
        Predicate::AllOf { predicates } => {
            let mut s = base;
            for p in predicates {
                s = synthesize_satisfying(p, s);
            }
            s
        }
        Predicate::AnyOf { predicates } => {
            // Satisfy the first child predicate; one suffices for AnyOf.
            let first = predicates
                .first()
                .expect("any_of.predicates must be non-empty");
            synthesize_satisfying(first, base)
        }
        Predicate::Not { predicate: _ } => {
            // Satisfying a Not means NOT satisfying the inner predicate. We
            // start from the empty `base` which by construction satisfies
            // `Not(<typical-predicate>)` because no flag is set. Drift gate
            // does NOT add anything for Not — caller must pass an empty
            // base when they want to test the satisfied-Not direction.
            base
        }
        // v0.6.0 SPEC §6.10.2 v3 slot-count predicates. The minimally-
        // satisfying state for each variant sets `slots.rows.len() ==
        // predicate.value` (the RHS). For Gte/Lte that's the boundary
        // value; a future strict-satisfaction sweep could also check
        // value+1 / value-1. v0.6.0 ships dead-code on the toolkit side
        // (no rule emits these), but the arms keep the match exhaustive
        // and prepare for the future Effect-grammar-extension cycle (rows
        // 9/10/11).
        Predicate::SlotCountEq { value }
        | Predicate::SlotCountGte { value }
        | Predicate::SlotCountLte { value } => set_slot_count(base, *value),
    }
}

/// Set the slot-row count to exactly `count`, preserving any existing rows
/// from `base` (truncate / pad as needed). Pad rows use SlotRow::default
/// (index 0, subkey Xpub, empty value) — fine for visibility predicates
/// which only consult `slots.rows.len()`, not row contents.
fn set_slot_count(mut state: FormState, count: usize) -> FormState {
    while state.slots.rows.len() < count {
        state.slots.rows.push(SlotRow::default());
    }
    state.slots.rows.truncate(count);
    state
}

/// Replace or push `(flag, value)` in state.values. Used so the drift
/// gate can layer predicate constraints without duplicates.
fn push_or_replace(mut state: FormState, flag: &str, value: FlagValue) -> FormState {
    if let Some(slot) = state.values.iter_mut().find(|(k, _)| k == flag) {
        slot.1 = value;
    } else {
        state.values.push((flag.into(), value));
    }
    state
}

/// Top-level drift gate: enumerate every subcommand with conditional rules
/// in the JSON, synthesize a satisfying FormState per rule, invoke the
/// hand-coded conditional fn, and assert visibility matches.
#[test]
fn gui_schema_conditional_rules_match_hand_coded_conditionals() {
    let Some(bin) = mnemonic_bin() else {
        eprintln!("MNEMONIC_BIN unset; skipping drift gate");
        return;
    };
    if !bin.exists() {
        // Best-effort PATH lookup: try invoking with `--help` cheaply; if
        // even that fails, skip. (We can't rely on the `which` crate
        // without adding a dev-dependency.)
        let probe = Command::new(&bin).arg("--help").output();
        if probe.is_err() {
            eprintln!(
                "mnemonic binary not resolvable (probe failed: {probe:?}); \
                 skipping drift gate"
            );
            return;
        }
    }
    let json = gui_schema_json();

    // Discover subcommand names from the JSON. Iterate them; for each,
    // pull conditional_rules and exercise every rule.
    let root: serde_json::Value = serde_json::from_str(&json).unwrap();
    let subs = root["subcommands"].as_array().expect("subcommands array");
    // v0.6.1 P3 #5B: per-subcommand rule counts replace the prior
    // `total_rules > 0` vacuous-pass assertion. Populated only for
    // subcommands the gate ACTUALLY exercises (post early-exit checks).
    let mut per_subcommand_rules: BTreeMap<String, usize> = BTreeMap::new();
    let mut skipped_no_conditional = 0_usize;

    for sub in subs {
        let sub_name = sub["name"].as_str().expect("subcommand name");
        let Some(rules) = parse_gui_schema_conditional_rules(&json, sub_name) else {
            continue; // version-gated absent on v1 docs; here always v2.
        };
        if rules.is_empty() {
            continue;
        }
        let Some(handcoded) = subcommand_named(sub_name) else {
            // Some toolkit subcommands aren't in the GUI schema (e.g.,
            // future additions); skip those for drift purposes.
            continue;
        };
        let Some(conditional_fn) = handcoded.conditional else {
            // Subcommand has rules in JSON but no hand-coded conditional —
            // legitimate drift; flag once.
            skipped_no_conditional += 1;
            eprintln!(
                "WARN: subcommand `{sub_name}` has {} JSON conditional_rules \
                 but no hand-coded conditional fn — partial drift",
                rules.len()
            );
            continue;
        };
        per_subcommand_rules.insert(sub_name.to_string(), rules.len());

        for rule in &rules {
            // Satisfied direction: synthesize FormState, invoke fn, check.
            // v0.7.0: search for an entry MATCHING the expected visibility
            // rather than the first entry for the flag. Multiple rules can
            // emit entries for the same flag with orthogonal effects (e.g.,
            // --template can be both Required AND have DisableOptions). The
            // runtime render loop handles composition by consuming the
            // primary first-rule-wins visibility + extracting DisableOptions
            // separately; the drift gate verifies the GUI EMITS the rule's
            // expected visibility somewhere in the map (presence, not order).
            let state = synthesize_satisfying(&rule.when, FormState::default());
            let vis_map = conditional_fn(&state);
            let expected = vis_to_visibility(rule.effect.visibility.clone());
            let found = vis_map
                .iter()
                .any(|(k, v)| *k == rule.effect.flag.as_str() && v == &expected);
            assert!(
                found,
                "drift in subcommand `{sub_name}`:\n  \
                 rule rationale: {}\n  \
                 spec_ref: {}\n  \
                 predicate: {:?}\n  \
                 target flag: {}\n  \
                 expected visibility: {expected:?}\n  \
                 vis_map for this flag: {:?}",
                rule.rationale,
                rule.spec_ref,
                rule.when,
                rule.effect.flag,
                vis_map
                    .iter()
                    .filter(|(k, _)| *k == rule.effect.flag.as_str())
                    .collect::<Vec<_>>(),
            );
        }
    }
    // v0.6.1 P3 #5B: per-subcommand lower-bound floors. The prior
    // `total_rules > 0` assertion would have silently passed a regression
    // that dropped the actual ~36 emitted rules down to a non-zero
    // handful (per [feedback-ci-snapshot-test-substring-vacuity]). The
    // floors below are the v0.18.0/v0.7.0 baseline; future cycles that
    // legitimately REDUCE a subcommand's rule count (rare — typically
    // only on intentional grammar refactors) must bump the floor in
    // lockstep.
    // v0.7.0 cycle: bundle bumps 11 -> 13 (rows 10 + 11 added — two
    // disable_options rules for slot_count-driven --template option
    // disablement). Total bumps 34 -> 36 in lockstep.
    const SUBCOMMAND_FLOORS: &[(&str, usize)] = &[
        ("bundle", 13),
        ("verify-bundle", 10),
        ("export-wallet", 6),
        ("convert", 4),
        ("derive-child", 3),
    ];
    for (name, floor) in SUBCOMMAND_FLOORS {
        let actual = per_subcommand_rules.get(*name).copied().unwrap_or(0);
        assert!(
            actual >= *floor,
            "drift gate per-subcommand floor violated: subcommand `{name}` \
             emitted {actual} rules, expected >= {floor}. Either a \
             regression dropped rules, or an intentional reduction requires \
             bumping the floor in tests/gui_schema_conditional_drift.rs::\
             SUBCOMMAND_FLOORS. (skipped_no_conditional: {skipped_no_conditional})"
        );
    }
    // Total-count sanity check derived from the floors (sum = 36).
    let total_rules: usize = per_subcommand_rules.values().sum();
    assert!(
        total_rules >= 36,
        "drift gate total: expected >= 36 rules across all subcommands, got \
         {total_rules}. Per-subcommand breakdown: {per_subcommand_rules:?}"
    );
}

// Round-trip a tiny in-memory v2 JSON to exercise the parse + synthesize
// path without depending on MNEMONIC_BIN.
#[test]
fn synthesize_satisfying_flag_present_pushes_text() {
    let predicate = Predicate::FlagPresent {
        flag: "--descriptor".into(),
    };
    let state = synthesize_satisfying(&predicate, FormState::default());
    assert!(state.has_value("--descriptor"));
}

#[test]
fn synthesize_satisfying_dropdown_value_in_picks_first() {
    let predicate = Predicate::DropdownValueIn {
        flag: "--template".into(),
        values: vec!["bip44".into(), "bip49".into(), "bip84".into(), "bip86".into()],
    };
    let state = synthesize_satisfying(&predicate, FormState::default());
    assert_eq!(state.dropdown_value("--template"), Some("bip44"));
}

#[test]
fn synthesize_satisfying_all_of_layers_multiple_predicates() {
    let predicate = Predicate::AllOf {
        predicates: vec![
            Predicate::FlagPresent {
                flag: "--descriptor".into(),
            },
            Predicate::DropdownValueIn {
                flag: "--network".into(),
                values: vec!["mainnet".into()],
            },
        ],
    };
    let state = synthesize_satisfying(&predicate, FormState::default());
    assert!(state.has_value("--descriptor"));
    assert_eq!(state.dropdown_value("--network"), Some("mainnet"));
}

// Suppress unused-import warning if the predicate variants get optimized
// out under cfg(test).
#[test]
fn _unused_imports_anchor() {
    let _e: Option<Effect> = None;
    let _r: Option<ConditionalRule> = None;
    let _t: Option<TaggedOrIndexedValue> = None;
}
