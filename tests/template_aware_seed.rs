//! v0.6.0 P4 — Template-aware default form-state seed tests.
//!
//! The `form::conditional::template_defaults_for` helper returns the
//! template-specific defaults applied by the per-frame egui hook in
//! `main.rs::update()`. Single-sig templates (bip44/49/84/86) have no
//! template-specific defaults — the universal seed at form-state init
//! already covers `--network` / `--account`. Multisig templates default
//! `--threshold = 2` + `--multisig-path-family = bip48` so the form is
//! one-click-runnable post-template-change.
//!
//! The egui hook applies these defaults via a seed-on-empty discipline —
//! only flags that aren't already present in `state.values` get seeded.
//! User-typed values across template switches are preserved; the hook
//! never overwrites, never clears, no undo affordance needed.
//!
//! These cells test:
//!   1. `template_defaults_for` shape per template class (single-sig vs
//!      multisig).
//!   2. The seed-on-empty composition pattern at the FormState level —
//!      seeds absent flags, preserves present ones.
//!   3. Regression: switching from multisig → single-sig does NOT clear
//!      the previously-seeded multisig fields (visibility gate handles
//!      Disabled rendering; values are preserved for round-trip to
//!      multisig).

use mnemonic_gui::form::conditional::{template_defaults_for, SINGLE_SIG_TEMPLATES};
use mnemonic_gui::schema::{FlagValue, FormState};

// ── (1) template_defaults_for per-template shape ──────────────────────

#[test]
fn template_defaults_for_single_sig_returns_empty() {
    for tmpl in SINGLE_SIG_TEMPLATES {
        let defaults = template_defaults_for(tmpl);
        assert!(
            defaults.is_empty(),
            "single-sig template `{tmpl}` must have no template-specific \
             defaults (universal seed at form-state init covers --network / \
             --account); got {defaults:?}",
        );
    }
}

#[test]
fn template_defaults_for_multisig_seeds_threshold_and_path_family() {
    // Spot-check three of the four multisig templates; the same defaults
    // apply for all (the else-branch in template_defaults_for hits every
    // non-single-sig template uniformly).
    for tmpl in ["wsh-multi", "wsh-sortedmulti", "tr-sortedmulti-a"] {
        let defaults = template_defaults_for(tmpl);
        let names: Vec<&str> = defaults.iter().map(|(n, _)| *n).collect();
        assert!(
            names.contains(&"--threshold"),
            "multisig template `{tmpl}` must seed --threshold; got {names:?}",
        );
        assert!(
            names.contains(&"--multisig-path-family"),
            "multisig template `{tmpl}` must seed --multisig-path-family; \
             got {names:?}",
        );
        // Specific values.
        let threshold = defaults
            .iter()
            .find(|(n, _)| *n == "--threshold")
            .map(|(_, v)| v)
            .unwrap();
        assert_eq!(*threshold, FlagValue::Number(2));
        let path_family = defaults
            .iter()
            .find(|(n, _)| *n == "--multisig-path-family")
            .map(|(_, v)| v)
            .unwrap();
        assert_eq!(*path_family, FlagValue::Dropdown("bip48".into()));
    }
}

// ── (2) Seed-on-empty composition pattern ──────────────────────────────

/// Reusable helper mirroring the per-frame hook in `main.rs::update()`.
/// Apply `template_defaults_for(new_template)` to `state.values`, but only
/// for flags that aren't already set.
fn apply_seed_on_empty(state: &mut FormState, new_template: &str) {
    for (name, default_value) in template_defaults_for(new_template) {
        if !state.has_value(name) {
            state.values.push((name.to_string(), default_value));
        }
    }
}

#[test]
fn seed_on_empty_pushes_defaults_for_absent_flags() {
    // Starting state has --scheme but no --threshold / --multisig-path-family.
    let mut state = FormState::from_pairs(vec![(
        "--scheme",
        FlagValue::Dropdown("wsh-sortedmulti".into()),
    )]);
    assert!(!state.has_value("--threshold"));
    assert!(!state.has_value("--multisig-path-family"));

    apply_seed_on_empty(&mut state, "wsh-sortedmulti");

    assert!(
        state.has_value("--threshold"),
        "seed-on-empty must push --threshold default for multisig template",
    );
    assert!(
        state.has_value("--multisig-path-family"),
        "seed-on-empty must push --multisig-path-family default",
    );
}

#[test]
fn seed_on_empty_preserves_user_typed_values() {
    // User typed --threshold = 3; switching to multisig template must NOT
    // overwrite their value with the seed default (2).
    let mut state = FormState::from_pairs(vec![
        ("--scheme", FlagValue::Dropdown("wsh-sortedmulti".into())),
        ("--threshold", FlagValue::Number(3)),
    ]);

    apply_seed_on_empty(&mut state, "wsh-sortedmulti");

    // --threshold stays at the user's 3, not the seeded default 2.
    let threshold = state
        .values
        .iter()
        .find(|(k, _)| k == "--threshold")
        .map(|(_, v)| v.clone())
        .expect("--threshold must remain in state");
    assert_eq!(
        threshold,
        FlagValue::Number(3),
        "user-typed --threshold = 3 MUST be preserved (seed-on-empty); \
         got {threshold:?}",
    );
    // --multisig-path-family was absent → still gets seeded.
    assert!(state.has_value("--multisig-path-family"));
}

// ── (3) Regression: template back-and-forth preserves prior seeds ──────

#[test]
fn template_round_trip_multisig_to_single_sig_preserves_seeded_values() {
    // Simulate the per-frame hook firing twice: once on template change
    // to multisig (seeds defaults), once on template change back to
    // single-sig (template_defaults_for returns empty → no clear).
    let mut state = FormState::from_pairs(vec![(
        "--scheme",
        FlagValue::Dropdown("wsh-sortedmulti".into()),
    )]);

    // Frame 1: user picked multisig → seed defaults.
    apply_seed_on_empty(&mut state, "wsh-sortedmulti");
    assert!(state.has_value("--threshold"));
    assert!(state.has_value("--multisig-path-family"));

    // Frame 2: user changed template back to single-sig.
    // Update --scheme in state.
    state
        .values
        .iter_mut()
        .find(|(k, _)| k == "--scheme")
        .unwrap()
        .1 = FlagValue::Dropdown("bip84".into());
    apply_seed_on_empty(&mut state, "bip84");

    // Multisig fields STILL present in state — visibility gate (separate
    // concern) handles their Disabled rendering. The hook MUST NOT have
    // cleared them. This is the "no destructive mutation" invariant.
    assert!(
        state.has_value("--threshold"),
        "--threshold MUST be preserved across multisig → single-sig switch \
         (seed-on-empty pattern is purely additive)",
    );
    assert!(state.has_value("--multisig-path-family"));
}

// ─── 2026-11-26 rename: the seed hook's scoping ─────────────────────────────
//
// `build-descriptor --scheme` (ex-`--archetype`) shares the flag name with the
// five toolkit wallet-scheme forms but not their value set; `md compose
// --scheme` and `ms derive --origin-path` are other CLIs. The hook must read a
// scheme on exactly the five, or picking e.g. `decaying-multisig` would seed
// `--threshold 2` into build-descriptor's argv.

#[test]
fn scheme_seed_value_fires_for_exactly_the_five_toolkit_scheme_forms() {
    use mnemonic_gui::form::conditional::{scheme_seed_value, SCHEME_TEMPLATE_SUBCOMMANDS};
    use mnemonic_gui::schema;

    let st = FormState::from_pairs(vec![
        ("--scheme", FlagValue::Dropdown("wsh-multi".into())),
        ("--origin-path", FlagValue::Dropdown("bip48".into())),
    ]);
    assert_eq!(
        SCHEME_TEMPLATE_SUBCOMMANDS,
        ["bundle", "verify-bundle", "convert", "export-wallet", "restore"]
    );
    for sub in SCHEME_TEMPLATE_SUBCOMMANDS {
        assert_eq!(scheme_seed_value("mnemonic", sub, &st), Some("wsh-multi"), "{sub}");
    }
    let bd = FormState::from_pairs(vec![(
        "--scheme",
        FlagValue::Dropdown("decaying-multisig".into()),
    )]);
    assert_eq!(scheme_seed_value("mnemonic", "build-descriptor", &bd), None);
    assert_eq!(scheme_seed_value("md", "compose", &st), None);
    assert_eq!(scheme_seed_value("ms", "derive", &st), None);

    // Every mnemonic form with a `--scheme` Dropdown is either one of the
    // five or build-descriptor: a new consumer must be classified here.
    let with_scheme: Vec<&str> = schema::mnemonic::SCHEMA
        .subcommands
        .iter()
        .filter(|s| s.flags.iter().any(|f| f.name == "--scheme"))
        .map(|s| s.name)
        .collect();
    for name in &with_scheme {
        assert!(
            SCHEME_TEMPLATE_SUBCOMMANDS.contains(name) || *name == "build-descriptor",
            "unclassified --scheme consumer {name}"
        );
        let fires = scheme_seed_value("mnemonic", name, &bd).is_some();
        assert_eq!(fires, *name != "build-descriptor", "{name}");
    }
    assert_eq!(with_scheme.len(), 6, "{with_scheme:?}");
}
