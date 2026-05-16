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
fn vis_to_visibility(v: VisibilityProjection) -> Visibility {
    match v {
        VisibilityProjection::Hidden => Visibility::Hidden,
        VisibilityProjection::Disabled => Visibility::Disabled,
        VisibilityProjection::Required => Visibility::Required,
        VisibilityProjection::PinValue { value } => Visibility::PinValue { value },
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
    let mut total_rules = 0_usize;
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

        for rule in &rules {
            total_rules += 1;
            // Satisfied direction: synthesize FormState, invoke fn, check.
            let state = synthesize_satisfying(&rule.when, FormState::default());
            let vis_map = conditional_fn(&state);
            let actual = vis_map
                .iter()
                .find(|(k, _)| *k == rule.effect.flag.as_str())
                .map(|(_, v)| v.clone())
                .unwrap_or(Visibility::Visible);
            let expected = vis_to_visibility(rule.effect.visibility.clone());
            assert_eq!(
                actual,
                expected,
                "drift in subcommand `{sub_name}`:\n  \
                 rule rationale: {}\n  \
                 spec_ref: {}\n  \
                 predicate: {:?}\n  \
                 target flag: {}\n  \
                 expected visibility: {expected:?}\n  \
                 actual visibility:   {actual:?}",
                rule.rationale,
                rule.spec_ref,
                rule.when,
                rule.effect.flag,
            );
        }
    }
    assert!(
        total_rules > 0,
        "drift gate must exercise at least one rule; got {total_rules} \
         (skipped_no_conditional: {skipped_no_conditional})"
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
