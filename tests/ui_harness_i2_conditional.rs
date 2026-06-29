//! Phase P2 — I2 conditional & state-integrity metamorphics over the **17
//! conditional subcommands** (the ones whose `SubcommandSchema.conditional` is
//! non-`None`: mnemonic ×12 + md ×3 + ms ×1 + mk ×1).
//!
//! Plan: `design/IMPLEMENTATION_PLAN_gui_ui_test_harness.md` Phase P2.
//! Spec:  `design/SPEC_gui_automated_ui_test_harness.md` §5 I2 (the 6-Visibility
//!        effect table is authoritative).
//!
//! ## KEY DIFFERENCE FROM P1 (plan m3): render the WHOLE form
//! P1/I1 rendered ONE flag in isolation (its render→store→argv wiring). I2
//! renders the **whole subcommand form** via [`ui_harness::render_whole_form`]
//! (a byte-faithful copy of `src/main.rs`'s per-flag visibility gate) so the
//! subcommand's `conditional(state)` fn actually RUNS and gates the fields. The
//! per-flag target is the flag's **name LABEL** (`query_by_label(flag.name)`,
//! an exact match — see [`probe_label_match_is_exact_not_substring`]); its
//! presence + `is_disabled()` read the rendered effect. See the module doc in
//! `tests/ui_harness/mod.rs` for the full targeting rationale.
//!
//! ## The 6 Visibility effects, as observed by the renderer (verified vs source)
//! | Effect           | renderer leg                              | observable here          |
//! |------------------|-------------------------------------------|--------------------------|
//! | `Visible`        | normal                                    | label present + enabled  |
//! | `Required`       | normal (no asterisk wired for *conditional* Required — `main.rs` consumes only Hidden/Disabled; the schema `*` is separate) | label present + enabled |
//! | `Hidden`         | form loop `continue`s the flag            | label ABSENT             |
//! | `Disabled`       | `add_enabled_ui(false, …)`                | label present + disabled |
//! | `PinValue{v}`    | NOT consumed in render (argv-only) — render assert is best-effort | label present + enabled; FIRM on argv |
//! | `DisableOptions` | Dropdown options greyed (NO live producer in the 17) | narrowed/synthetic; FIRM on argv |
//!
//! Source notes confirmed while authoring: `main.rs:648-686` applies ONLY
//! `Hidden`(→`continue`) and `Disabled`(→`add_enabled_ui`); `PinValue` and the
//! conditional `Required` marker are NOT rendered (PinValue is argv-only in
//! `invocation.rs:208`). Of the 17 conditionals, ONLY `bundle` emits `PinValue`
//! (`--account`→0) and NONE emit `DisableOptions` (the v0.7.1 bundle row-10/11
//! `DisableOptions` pushes were reverted in v0.7.2 — `conditional.rs:257`).
//!
//! ## Anti-tautology (stated)
//! I2 asserts the RENDERER faithfully applies `conditional()` (catches
//! renderer↔rule **desync** + that egui→AccessKit yields the expected
//! absent/disabled states). It does NOT re-assert the rule's *correctness* — a
//! wrong rule is owned by `tests/conditional_visibility.rs` /
//! `tests/gui_schema_conditional_drift.rs`. `effect_of(...)` preconditions
//! (used where `Required` is render-indistinguishable from `Visible`) read the
//! rule only to keep the cell honest about what state it set up.

mod ui_harness;

use egui::accesskit::Role;
use egui_kittest::kittest::Queryable;
use egui_kittest::Harness;

use mnemonic_gui::app::CliTab;
use mnemonic_gui::form::invocation::assemble_argv;
use mnemonic_gui::form::tree_model::TreeState;
use mnemonic_gui::schema::{
    FlagKind, FlagSchema, FlagValue, FlagVisibility, FormState, Schema, SubcommandSchema,
    Visibility,
};

use ui_harness::{
    effect_of, eligible_for_label_check, label_disabled, label_present, render_whole_form_harness,
    schema_for, seed, seed_composite, sub_of, visibility_projection,
};

/// A per-subcommand state-seeding function (shared by the targeted cells + the
/// universal sweeps). Aliased to keep the case tables under clippy's
/// `type_complexity` bar.
type SeedFn = fn(&mut FormState, &'static SubcommandSchema);

// ─── local render-assertion helpers (read the rendered effect off the label) ─

fn assert_absent(h: &Harness<'static, FormState>, name: &str) {
    assert!(
        !label_present(h, name),
        "expected `{name}` ABSENT (Hidden / mode-suppressed) but its name-label rendered"
    );
}
fn assert_disabled(h: &Harness<'static, FormState>, name: &str) {
    match label_disabled(h, name) {
        Some(true) => {}
        Some(false) => panic!("expected `{name}` present + DISABLED but it rendered ENABLED"),
        None => panic!("expected `{name}` present + DISABLED but its name-label is ABSENT"),
    }
}
fn assert_enabled(h: &Harness<'static, FormState>, name: &str) {
    match label_disabled(h, name) {
        Some(false) => {}
        Some(true) => panic!("expected `{name}` present + ENABLED but it rendered DISABLED"),
        None => panic!("expected `{name}` present + ENABLED but its name-label is ABSENT"),
    }
}

/// Build a fresh whole-form harness, apply `seed_fn`, and run to stable.
fn form(
    tab: CliTab,
    sub_name: &str,
    seed_fn: SeedFn,
) -> Harness<'static, FormState> {
    let sub = sub_of(tab, sub_name);
    let mut state = FormState::default();
    seed_fn(&mut state, sub);
    let mut h = render_whole_form_harness(tab, sub, state);
    h.run();
    h
}

/// `conditional(settled_state)`'s effect for `flag` — the precondition reader.
fn eff(tab: CliTab, sub_name: &str, h: &Harness<'static, FormState>, flag: &str) -> Visibility {
    effect_of(sub_of(tab, sub_name), h.state(), flag)
}

fn argv(tab: CliTab, sub_name: &str, h: &Harness<'static, FormState>) -> Vec<String> {
    assemble_argv(schema_for(tab), sub_of(tab, sub_name), h.state())
}

// ─── seed fns (shared by the targeted cells + the universal sweeps) ──────────

fn s_noop(_st: &mut FormState, _sub: &'static SubcommandSchema) {}

fn s_bundle_descriptor(st: &mut FormState, sub: &'static SubcommandSchema) {
    // wpkh(@0) classifies CANONICAL → the --account PinValue(0) pin fires.
    seed(st, sub, "--descriptor", "wpkh(@0)");
}
fn s_bundle_singlesig(st: &mut FormState, sub: &'static SubcommandSchema) {
    seed(st, sub, "--template", "bip84"); // ∈ SINGLE_SIG_TEMPLATES
}
fn s_verify_bundle_json(st: &mut FormState, sub: &'static SubcommandSchema) {
    seed(st, sub, "--bundle-json", "{}");
}
fn s_convert_address(st: &mut FormState, sub: &'static SubcommandSchema) {
    seed_composite(st, "--from", "phrase", "x"); // NOT electrum
    seed(st, sub, "--to", "address");
    // Seed --template EMPTY so the form-loop's auto-seed (which would default a
    // Dropdown to a PRESENT opts[0]) cannot set `has_template`, keeping the
    // `to_has_address && !has_template ⇒ --script-type Required` arm reachable.
    seed(st, sub, "--template", "");
}
fn s_convert_phrase(st: &mut FormState, sub: &'static SubcommandSchema) {
    seed_composite(st, "--from", "phrase", "x");
    seed(st, sub, "--to", "phrase");
}
fn s_export_tr(st: &mut FormState, sub: &'static SubcommandSchema) {
    seed(st, sub, "--template", "tr-multi-a"); // ∈ TAPROOT_INTERNAL_KEY_TEMPLATES
}
fn s_export_singlesig(st: &mut FormState, sub: &'static SubcommandSchema) {
    seed(st, sub, "--template", "bip84"); // ∈ SINGLE_SIG_TEMPLATES
}
fn s_export_descriptor(st: &mut FormState, sub: &'static SubcommandSchema) {
    seed(st, sub, "--descriptor", "wsh(sortedmulti(2,@0,@1))");
    // Empty --template (defeat the form-loop auto-seed) so `has_template` stays
    // false ⇒ the descriptor-mode `--template Disabled` arm is observable and
    // --descriptor itself is NOT cross-disabled.
    seed(st, sub, "--template", "");
}
fn s_build_spec(st: &mut FormState, sub: &'static SubcommandSchema) {
    // --spec set + --archetype empty ⇒ active_archetype None (no mode
    // suppression) ⇒ the conditional's `--archetype Disabled` is observable.
    seed(st, sub, "--spec", "/tmp/x.json");
}
fn s_build_archetype(st: &mut FormState, sub: &'static SubcommandSchema) {
    seed(st, sub, "--archetype", "decaying-multisig"); // valid id
}
fn s_build_tree(st: &mut FormState, _sub: &'static SubcommandSchema) {
    st.tree = Some(TreeState {
        enabled: true,
        ..Default::default()
    });
}
fn s_restore_md1(st: &mut FormState, sub: &'static SubcommandSchema) {
    seed(st, sub, "--md1", "md1xyz");
}
fn s_derive_dice(st: &mut FormState, sub: &'static SubcommandSchema) {
    seed(st, sub, "--application", "dice");
}
fn s_slip39_split_entropy(st: &mut FormState, _sub: &'static SubcommandSchema) {
    seed_composite(st, "--from", "entropy", "00");
}
fn s_slip39_combine_entropy(st: &mut FormState, sub: &'static SubcommandSchema) {
    seed(st, sub, "--to", "entropy");
}
fn s_slip39_combine_phrase(st: &mut FormState, sub: &'static SubcommandSchema) {
    seed(st, sub, "--to", "phrase");
}
fn s_repair_one(st: &mut FormState, sub: &'static SubcommandSchema) {
    seed(st, sub, "--ms1", "ms1abc");
}
fn s_compare_mini(st: &mut FormState, sub: &'static SubcommandSchema) {
    seed(st, sub, "--miniscript", "pk(A)");
}
fn s_compare_desc(st: &mut FormState, sub: &'static SubcommandSchema) {
    seed(st, sub, "--descriptor", "wsh(pk(A))");
}
fn s_md_encode_positional(st: &mut FormState, _sub: &'static SubcommandSchema) {
    st.positionals.push("wsh(pk(@0))".to_string());
}
fn s_md_encode_frompolicy(st: &mut FormState, sub: &'static SubcommandSchema) {
    seed(st, sub, "--from-policy", "and(pk(A),pk(B))");
}
fn s_md_encode_segwit(st: &mut FormState, sub: &'static SubcommandSchema) {
    seed(st, sub, "--from-policy", "and(pk(A),pk(B))");
    seed(st, sub, "--context", "segwitv0");
}
fn s_md_compile_segwit(st: &mut FormState, sub: &'static SubcommandSchema) {
    seed(st, sub, "--context", "segwitv0");
}
fn s_md_address_positional(st: &mut FormState, _sub: &'static SubcommandSchema) {
    st.positionals.push("ms1phrasewords".to_string());
}
fn s_ms_encode_hex(st: &mut FormState, sub: &'static SubcommandSchema) {
    seed(st, sub, "--hex", "00ff");
}
fn s_mk_encode_fp(st: &mut FormState, sub: &'static SubcommandSchema) {
    seed(st, sub, "--origin-fingerprint", "deadbeef");
}
fn s_mk_encode_priv(st: &mut FormState, sub: &'static SubcommandSchema) {
    seed(st, sub, "--privacy-preserving", "true");
}

// ════════════════════════════════════════════════════════════════════════
// PROBES — the three load-bearing render/targeting assumptions
// ════════════════════════════════════════════════════════════════════════

#[test]
fn probe_label_match_is_exact_not_substring() {
    // `bundle` carries BOTH `--descriptor`/`--descriptor-file` and
    // `--passphrase`/`--passphrase-stdin`. A substring `query_by_label` would
    // match two nodes and panic — proving exact-match per-flag targeting.
    let h = form(CliTab::Mnemonic, "bundle", s_noop);
    assert!(label_present(&h, "--descriptor"));
    assert!(label_present(&h, "--descriptor-file"));
    assert!(label_present(&h, "--passphrase"));
    assert!(label_present(&h, "--passphrase-stdin"));
}

#[test]
fn probe_disabled_state_reflects_add_enabled_ui_gate() {
    // compare-cost: --miniscript populated ⇒ --descriptor Disabled. Proves the
    // `add_enabled_ui(false)` wrap propagates to the inner label's
    // `is_disabled()`, and the populated sibling stays enabled.
    let h = form(CliTab::Mnemonic, "compare-cost", s_compare_mini);
    assert_enabled(&h, "--miniscript");
    assert_disabled(&h, "--descriptor");
}

#[test]
fn probe_hidden_flag_label_is_absent() {
    // ms encode: --hex set ⇒ --language Hidden ⇒ its label is absent.
    let h = form(CliTab::Ms, "encode", s_ms_encode_hex);
    assert_absent(&h, "--language");
}

// ════════════════════════════════════════════════════════════════════════
// INVARIANT 1 — renderer-applies-the-rule, per effect, across the 17
// ════════════════════════════════════════════════════════════════════════

#[test]
fn i1_bundle() {
    // Descriptor mode (canonical): mutex Disables + the --account PinValue.
    let h = form(CliTab::Mnemonic, "bundle", s_bundle_descriptor);
    assert_disabled(&h, "--descriptor-file");
    assert_disabled(&h, "--template");
    assert_disabled(&h, "--threshold");
    assert_disabled(&h, "--multisig-path-family");
    // PinValue is NOT consumed in render (argv-only) → best-effort: the
    // --account widget renders present + enabled. The FIRM PinValue claim is
    // i1_bundle_pinvalue_firm_argv.
    assert_eq!(
        eff(CliTab::Mnemonic, "bundle", &h, "--account"),
        Visibility::PinValue {
            value: serde_json::json!(0)
        }
    );
    assert_enabled(&h, "--account");

    // Empty form: --template Required (present + enabled).
    let h = form(CliTab::Mnemonic, "bundle", s_noop);
    assert_eq!(
        eff(CliTab::Mnemonic, "bundle", &h, "--template"),
        Visibility::Required
    );
    assert_enabled(&h, "--template");

    // Single-sig template Disables threshold + path-family.
    let h = form(CliTab::Mnemonic, "bundle", s_bundle_singlesig);
    assert_disabled(&h, "--threshold");
    assert_disabled(&h, "--multisig-path-family");
}

#[test]
fn i1_verify_bundle() {
    // --bundle-json populated ⇒ the three cards Disabled.
    let h = form(CliTab::Mnemonic, "verify-bundle", s_verify_bundle_json);
    assert_disabled(&h, "--ms1");
    assert_disabled(&h, "--mk1");
    assert_disabled(&h, "--md1");

    // Empty: --mk1/--md1 Required, --template Required (present + enabled).
    let h = form(CliTab::Mnemonic, "verify-bundle", s_noop);
    assert_eq!(
        eff(CliTab::Mnemonic, "verify-bundle", &h, "--mk1"),
        Visibility::Required
    );
    assert_enabled(&h, "--mk1");
    assert_enabled(&h, "--md1");
    assert_enabled(&h, "--template");
}

#[test]
fn i1_convert() {
    // --to address (no --template): --script-type + --path Required; --xpub-prefix
    // + --electrum-version Disabled.
    let h = form(CliTab::Mnemonic, "convert", s_convert_address);
    assert_enabled(&h, "--script-type");
    assert_enabled(&h, "--path");
    assert_eq!(
        eff(CliTab::Mnemonic, "convert", &h, "--script-type"),
        Visibility::Required
    );
    assert_disabled(&h, "--xpub-prefix");
    assert_disabled(&h, "--electrum-version");

    // --to phrase: --script-type / --path / --template Disabled.
    let h = form(CliTab::Mnemonic, "convert", s_convert_phrase);
    assert_disabled(&h, "--script-type");
    assert_disabled(&h, "--path");
    assert_disabled(&h, "--template");
}

#[test]
fn i1_export_wallet() {
    // Descriptor mode (--template kept empty): --template Disabled; --descriptor
    // enabled. (The "neither set ⇒ both Required" arm is NOT render-reachable —
    // the form-loop auto-seeds --template to a present opts[0] on a fresh form,
    // so `has_template` is always true there; tested via the seeded modes below.)
    let h = form(CliTab::Mnemonic, "export-wallet", s_export_descriptor);
    assert_disabled(&h, "--template");
    assert_enabled(&h, "--descriptor");

    // Taproot multi-leaf template: --taproot-internal-key Required; --descriptor
    // Disabled.
    let h = form(CliTab::Mnemonic, "export-wallet", s_export_tr);
    assert_enabled(&h, "--taproot-internal-key");
    assert_eq!(
        eff(CliTab::Mnemonic, "export-wallet", &h, "--taproot-internal-key"),
        Visibility::Required
    );
    assert_disabled(&h, "--descriptor");

    // Single-sig template: --threshold + --multisig-path-family +
    // --taproot-internal-key Disabled.
    let h = form(CliTab::Mnemonic, "export-wallet", s_export_singlesig);
    assert_disabled(&h, "--threshold");
    assert_disabled(&h, "--multisig-path-family");
    assert_disabled(&h, "--taproot-internal-key");
}

#[test]
fn i1_build_descriptor() {
    // --spec set (archetype EMPTY ⇒ no mode suppression): --archetype Disabled,
    // --spec present + enabled.
    let h = form(CliTab::Mnemonic, "build-descriptor", s_build_spec);
    assert_disabled(&h, "--archetype");
    assert_enabled(&h, "--spec");

    // Archetype mode: --spec is mode-suppressed (ABSENT) by
    // `suppressed_in_archetype_mode`; --archetype stays present + enabled.
    let h = form(CliTab::Mnemonic, "build-descriptor", s_build_archetype);
    assert_absent(&h, "--spec");
    assert_enabled(&h, "--archetype");

    // Tree mode: --spec / --archetype / --emit-spec all mode-suppressed (ABSENT).
    let h = form(CliTab::Mnemonic, "build-descriptor", s_build_tree);
    assert_absent(&h, "--spec");
    assert_absent(&h, "--archetype");
    assert_absent(&h, "--emit-spec");
}

#[test]
fn i1_restore() {
    // No --md1 ⇒ --from Required; with --md1 ⇒ --from Visible. Render is
    // present+enabled in BOTH (no conditional-Required asterisk wired), so the
    // EFFECT transition is the observable substance here.
    let h = form(CliTab::Mnemonic, "restore", s_noop);
    assert_eq!(
        eff(CliTab::Mnemonic, "restore", &h, "--from"),
        Visibility::Required
    );
    assert_enabled(&h, "--from");

    let h = form(CliTab::Mnemonic, "restore", s_restore_md1);
    assert_eq!(
        eff(CliTab::Mnemonic, "restore", &h, "--from"),
        Visibility::Visible
    );
    assert_enabled(&h, "--from");
}

#[test]
fn i1_derive_child() {
    // --application dice ⇒ --dice-sides Required (the only non-secret effect;
    // the passphrase pair's effect is on a secret Boolean, excluded).
    let h = form(CliTab::Mnemonic, "derive-child", s_derive_dice);
    assert_eq!(
        eff(CliTab::Mnemonic, "derive-child", &h, "--dice-sides"),
        Visibility::Required
    );
    assert_enabled(&h, "--dice-sides");

    let h = form(CliTab::Mnemonic, "derive-child", s_noop);
    assert_eq!(
        eff(CliTab::Mnemonic, "derive-child", &h, "--dice-sides"),
        Visibility::Visible
    );
    assert_enabled(&h, "--dice-sides");
}

#[test]
fn i1_slip39_split() {
    // --from entropy ⇒ --language Hidden; --from phrase ⇒ --language Visible.
    let h = form(CliTab::Mnemonic, "slip39-split", s_slip39_split_entropy);
    assert_absent(&h, "--language");

    let mut state = FormState::default();
    seed_composite(&mut state, "--from", "phrase", "abandon abandon");
    let sub = sub_of(CliTab::Mnemonic, "slip39-split");
    let mut h = render_whole_form_harness(CliTab::Mnemonic, sub, state);
    h.run();
    assert_enabled(&h, "--language");
}

#[test]
fn i1_slip39_combine() {
    // --to entropy ⇒ --language Hidden; --to phrase ⇒ --language Visible.
    let h = form(CliTab::Mnemonic, "slip39-combine", s_slip39_combine_entropy);
    assert_absent(&h, "--language");

    let h = form(CliTab::Mnemonic, "slip39-combine", s_slip39_combine_phrase);
    assert_enabled(&h, "--language");
}

#[test]
fn i1_repair() {
    // Zero cards ⇒ all three Required; one card ⇒ all Visible (D35 mixed-HRP:
    // the others are NOT disabled). Render present+enabled in both → EFFECT is
    // the substance.
    let h = form(CliTab::Mnemonic, "repair", s_noop);
    for c in ["--ms1", "--mk1", "--md1"] {
        assert_eq!(
            eff(CliTab::Mnemonic, "repair", &h, c),
            Visibility::Required
        );
        assert_enabled(&h, c);
    }
    let h = form(CliTab::Mnemonic, "repair", s_repair_one);
    for c in ["--ms1", "--mk1", "--md1"] {
        assert_eq!(eff(CliTab::Mnemonic, "repair", &h, c), Visibility::Visible);
        assert_enabled(&h, c);
    }
}

#[test]
fn i1_inspect() {
    let h = form(CliTab::Mnemonic, "inspect", s_noop);
    for c in ["--ms1", "--mk1", "--md1"] {
        assert_eq!(
            eff(CliTab::Mnemonic, "inspect", &h, c),
            Visibility::Required
        );
        assert_enabled(&h, c);
    }
}

#[test]
fn i1_compare_cost() {
    let h = form(CliTab::Mnemonic, "compare-cost", s_compare_mini);
    assert_disabled(&h, "--descriptor");
    assert_enabled(&h, "--miniscript");

    let h = form(CliTab::Mnemonic, "compare-cost", s_compare_desc);
    assert_disabled(&h, "--miniscript");
    assert_enabled(&h, "--descriptor");
}

#[test]
fn i1_md_encode() {
    // Positional template filled: --from-policy Disabled; --context +
    // --unspendable-key Hidden.
    let h = form(CliTab::Md, "encode", s_md_encode_positional);
    assert_disabled(&h, "--from-policy");
    assert_absent(&h, "--context");
    assert_absent(&h, "--unspendable-key");

    // --from-policy set: --context Required (present + enabled).
    let h = form(CliTab::Md, "encode", s_md_encode_frompolicy);
    assert_eq!(
        eff(CliTab::Md, "encode", &h, "--context"),
        Visibility::Required
    );
    assert_enabled(&h, "--context");

    // --context segwitv0 (+ from-policy): --unspendable-key Disabled.
    let h = form(CliTab::Md, "encode", s_md_encode_segwit);
    assert_disabled(&h, "--unspendable-key");
}

#[test]
fn i1_md_compile() {
    let h = form(CliTab::Md, "compile", s_md_compile_segwit);
    assert_disabled(&h, "--unspendable-key");

    let h = form(CliTab::Md, "compile", s_noop); // --context defaults to "tap"
    assert_enabled(&h, "--unspendable-key");
}

#[test]
fn i1_md_address() {
    // Positional phrases filled: --template/--key/--fingerprint Disabled.
    let h = form(CliTab::Md, "address", s_md_address_positional);
    assert_disabled(&h, "--template");
    assert_disabled(&h, "--key");
    assert_disabled(&h, "--fingerprint");

    // Empty: --template Required.
    let h = form(CliTab::Md, "address", s_noop);
    assert_eq!(
        eff(CliTab::Md, "address", &h, "--template"),
        Visibility::Required
    );
    assert_enabled(&h, "--template");
}

#[test]
fn i1_ms_encode() {
    // --hex set: --phrase Disabled, --language Hidden.
    let h = form(CliTab::Ms, "encode", s_ms_encode_hex);
    assert_disabled(&h, "--phrase");
    assert_absent(&h, "--language");

    // Empty: --phrase + --hex Required.
    let h = form(CliTab::Ms, "encode", s_noop);
    assert_eq!(
        eff(CliTab::Ms, "encode", &h, "--phrase"),
        Visibility::Required
    );
    assert_enabled(&h, "--phrase");
    assert_enabled(&h, "--hex");
}

#[test]
fn i1_mk_encode() {
    let h = form(CliTab::Mk, "encode", s_mk_encode_fp);
    assert_disabled(&h, "--privacy-preserving");

    let h = form(CliTab::Mk, "encode", s_mk_encode_priv);
    assert_disabled(&h, "--origin-fingerprint");
}

// ─── INVARIANT 1, PinValue FIRM (argv coercion), the bundle --account pin ────

#[test]
fn i1_bundle_pinvalue_firm_argv() {
    // PinValue's renderer leg is best-effort (not consumed in render); the FIRM
    // claim is on argv: `--account` emits the COERCED `0`, NOT the user's typed
    // value. Set --account 5 + a canonical --descriptor, settle, assemble.
    let sub = sub_of(CliTab::Mnemonic, "bundle");
    let mut state = FormState::default();
    seed(&mut state, sub, "--descriptor", "wpkh(@0)"); // canonical ⇒ pin fires
    state
        .values
        .push(("--account".to_string(), FlagValue::Number(5)));
    let mut h = render_whole_form_harness(CliTab::Mnemonic, sub, state);
    h.run();

    let argv = argv(CliTab::Mnemonic, "bundle", &h);
    let pos = argv
        .iter()
        .position(|t| t == "--account")
        .unwrap_or_else(|| panic!("--account must emit (pinned): {argv:?}"));
    assert_eq!(
        argv.get(pos + 1).map(String::as_str),
        Some("0"),
        "PinValue must REPLACE the user's --account 5 with the coerced 0: {argv:?}"
    );
    assert!(
        !argv.iter().any(|t| t == "5"),
        "the user-typed --account value 5 must NOT survive the pin: {argv:?}"
    );
}

// ─── INVARIANT 1, DisableOptions narrow (no live producer) — synthetic ───────

const DISOPT_OPTIONS: &[&str] = &["alpha", "beta", "gamma"];

const DISOPT_FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--mark",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Test marker — when true, the conditional greys out the `beta` \
               option of --pick via DisableOptions.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--pick",
        kind: FlagKind::Dropdown(DISOPT_OPTIONS),
        required: false,
        repeating: false,
        help: "Dropdown whose `beta` option is DisableOptions-greyed.",
        secret: false,
        default_value: None,
        global: false,
    },
];

fn disopt_conditional(state: &FormState) -> FlagVisibility {
    if state.has_value("--mark") {
        vec![(
            "--pick",
            Visibility::DisableOptions {
                values: vec!["beta".to_string()],
            },
        )]
    } else {
        Vec::new()
    }
}

const DISOPT_SUB: SubcommandSchema = SubcommandSchema {
    name: "do-thing",
    human_name: "Do Thing",
    flags: DISOPT_FLAGS,
    positional_args: &[],
    allows_slots: false,
    conditional: Some(disopt_conditional),
};
const DISOPT_SUBS: &[SubcommandSchema] = &[DISOPT_SUB];
static DISOPT_SCHEMA: Schema = Schema {
    cli_name: "mock",
    pinned_version: "mock 0.0.0",
    subcommands: DISOPT_SUBS,
};

#[test]
fn i1_disable_options_no_live_producer_in_the_17() {
    // None of the 17 conditionals emit DisableOptions (the v0.7.1 bundle row
    // 10/11 pushes were reverted in v0.7.2). Pin that fact so a future live
    // producer is a deliberate, reviewed change that must extend this cell.
    for tab in CliTab::ALL.iter().copied() {
        for sub in schema_for(tab).subcommands {
            let Some(cf) = sub.conditional else { continue };
            // Probe a spread of states; none should ever yield DisableOptions.
            let mut probes: Vec<FormState> = Vec::new();
            probes.push(FormState::default());
            for flag in sub.flags {
                let mut s = FormState::default();
                s.values.push((flag.name.to_string(), default_present(flag)));
                probes.push(s);
            }
            for st in &probes {
                for (_name, v) in cf(st) {
                    assert!(
                        !matches!(v, Visibility::DisableOptions { .. }),
                        "{}/{}: a conditional emitted DisableOptions — extend the I2 \
                         DisableOptions cell to render-cover it",
                        tab.bin_name(),
                        sub.name
                    );
                }
            }
        }
    }
}

/// A present-ish value for `flag` (probe helper for the no-producer scan).
fn default_present(flag: &'static FlagSchema) -> FlagValue {
    match flag.kind {
        FlagKind::Boolean => FlagValue::Boolean(true),
        FlagKind::Dropdown(opts) => {
            FlagValue::Dropdown(opts.first().copied().unwrap_or("x").to_string())
        }
        FlagKind::NodeValueComposite(opts) => FlagValue::NodeValueComposite {
            node: opts.first().copied().unwrap_or("x").to_string(),
            value: "v".to_string(),
        },
        FlagKind::Number { .. } => FlagValue::Number(1),
        FlagKind::Path { .. } => FlagValue::Path("p".to_string()),
        _ => FlagValue::Text("v".to_string()),
    }
}

#[test]
fn i1_disable_options_render_best_effort_and_argv_firm() {
    // Synthetic (no live producer): a Dropdown with `beta` DisableOptions-greyed
    // and a STALE `beta` already in state.
    let mut state = FormState::default();
    state
        .values
        .push(("--mark".to_string(), FlagValue::Boolean(true)));
    state
        .values
        .push(("--pick".to_string(), FlagValue::Dropdown("beta".to_string())));

    let mut h = Harness::new_ui_state(
        |ui, st: &mut FormState| {
            ui_harness::render_whole_form(ui, CliTab::Mnemonic, &DISOPT_SUB, st);
        },
        state,
    );
    h.run();

    // Best-effort render: --pick itself is NOT a per-flag Disabled node — the
    // flag's own label is present + ENABLED (only its `beta` OPTION is greyed,
    // inside the popup). Open the popup; the greyed option still renders.
    assert_enabled(&h, "--pick");
    h.get_by_role(Role::ComboBox).click();
    h.run();
    assert!(
        h.query_by_label("beta").is_some(),
        "the DisableOptions-greyed option still renders (greyed, non-selectable)"
    );

    // FIRM argv: DisableOptions is schema-time only — the stale `beta` value
    // STILL emits (it does NOT join the Hidden|Disabled suppress set).
    let argv = assemble_argv(&DISOPT_SCHEMA, &DISOPT_SUB, h.state());
    assert!(
        argv.windows(2).any(|w| w[0] == "--pick" && w[1] == "beta"),
        "DisableOptions must NOT change argv — the stale `beta` still emits: {argv:?}"
    );
}

// ─── INVARIANT 1, universal: render matches conditional() projection ─────────

/// One representative state per conditional subcommand (≥1 non-Visible effect
/// each) for the universal render↔rule faithfulness sweep.
const CASES: &[(CliTab, &str, SeedFn)] = &[
    (CliTab::Mnemonic, "bundle", s_bundle_descriptor),
    (CliTab::Mnemonic, "verify-bundle", s_verify_bundle_json),
    (CliTab::Mnemonic, "convert", s_convert_address),
    (CliTab::Mnemonic, "export-wallet", s_export_tr),
    (CliTab::Mnemonic, "build-descriptor", s_build_spec),
    (CliTab::Mnemonic, "restore", s_noop),
    (CliTab::Mnemonic, "derive-child", s_derive_dice),
    (CliTab::Mnemonic, "slip39-split", s_slip39_split_entropy),
    (CliTab::Mnemonic, "slip39-combine", s_slip39_combine_entropy),
    (CliTab::Mnemonic, "repair", s_noop),
    (CliTab::Mnemonic, "inspect", s_noop),
    (CliTab::Mnemonic, "compare-cost", s_compare_mini),
    (CliTab::Md, "encode", s_md_encode_positional),
    (CliTab::Md, "compile", s_md_compile_segwit),
    (CliTab::Md, "address", s_md_address_positional),
    (CliTab::Ms, "encode", s_ms_encode_hex),
    (CliTab::Mk, "encode", s_mk_encode_fp),
];

#[test]
fn i1_render_matches_conditional_projection_over_all_17() {
    // For each case, the rendered presence/enabled state of EVERY eligible flag
    // must equal `conditional(settled_state)`'s effect for it. Breadth check
    // that pins the per-effect→AccessKit translation + the form-loop gate's
    // fidelity across all 17 conditional subs and all their flags.
    assert_eq!(CASES.len(), 17, "CASES must cover all 17 conditional subcommands");
    let mut total_checked = 0usize;
    let mut non_visible = 0usize;
    for (tab, sub_name, seed_fn) in CASES.iter().copied() {
        let h = form(tab, sub_name, seed_fn);
        let sub = sub_of(tab, sub_name);
        let state = h.state();
        for flag in sub.flags {
            if !eligible_for_label_check(sub, flag, state) {
                continue;
            }
            let effect = effect_of(sub, state, flag.name);
            match &effect {
                Visibility::Hidden => assert_absent(&h, flag.name),
                Visibility::Disabled => assert_disabled(&h, flag.name),
                // Visible / Required / PinValue / (DisableOptions — none live):
                // present + enabled.
                _ => assert_enabled(&h, flag.name),
            }
            if !matches!(effect, Visibility::Visible) {
                non_visible += 1;
            }
            total_checked += 1;
        }
    }
    assert!(total_checked > 0, "the universal sweep checked no flags");
    assert!(
        non_visible > 0,
        "the universal sweep observed no non-Visible effects — it is vacuous"
    );
}

// ════════════════════════════════════════════════════════════════════════
// INVARIANT 2 — value-suppression fenced to Hidden|Disabled
// ════════════════════════════════════════════════════════════════════════
// NOTE the fence: suppression is asserted ONLY for Hidden|Disabled. It is NOT
// asserted for PinValue (emits-REPLACED — i1_bundle_pinvalue_firm_argv) nor
// DisableOptions (emits-STALE — i1_disable_options_render_best_effort_and_argv_firm);
// asserting suppression there would false-RED documented-correct behavior.

#[test]
fn i2_value_suppression_disabled_md_compile_fully_widget_driven() {
    // md compile renders exactly ONE TextInput (--unspendable-key) and ONE
    // ComboBox (--context), so BOTH the value injection AND the gating drive are
    // uniquely role-targetable — a fully widget-driven render→store→suppress
    // cell (no seeding).
    let sub = sub_of(CliTab::Md, "compile");
    let mut h = render_whole_form_harness(CliTab::Md, sub, FormState::default());
    h.run();

    // 1. Widget-inject a fixture into --unspendable-key (the unique TextInput).
    h.get_by_role(Role::TextInput).type_text("UNSPEND_FIXTURE");
    h.run();
    h.run(); // settle: TextEdit write-back lands at frame end

    // Negative control: while --context is "tap" (default), the value emits.
    let argv0 = argv(CliTab::Md, "compile", &h);
    assert!(
        argv0
            .windows(2)
            .any(|w| w[0] == "--unspendable-key" && w[1] == "UNSPEND_FIXTURE"),
        "precondition: the widget-injected value reaches argv while enabled: {argv0:?}"
    );

    // 2. Widget-drive --context → segwitv0 (the unique ComboBox); the md_compile
    //    conditional turns that into Disabled on --unspendable-key.
    h.get_by_role(Role::ComboBox).click();
    h.run();
    h.get_by_label("segwitv0").click();
    h.run();
    assert_disabled(&h, "--unspendable-key");

    // 3. The Disabled flag's value must NOT reach argv (Hidden|Disabled fence).
    let argv1 = argv(CliTab::Md, "compile", &h);
    assert!(
        !argv1.iter().any(|t| t == "--unspendable-key"),
        "a Disabled flag's value must be suppressed from argv: {argv1:?}"
    );
    assert!(
        !argv1.iter().any(|t| t == "UNSPEND_FIXTURE"),
        "the suppressed value token must not survive: {argv1:?}"
    );
}

#[test]
fn i2_value_suppression_hidden_ms_encode_render_settled() {
    // Hidden-leg suppression after a real whole-form render-settle. The
    // suppressed value (--language) is seeded (I1 + the md_compile cell own the
    // widget→store proof); the gate (--hex) is what flips it Hidden.
    let sub = sub_of(CliTab::Ms, "encode");

    // Negative control: --language non-default reaches argv when Visible.
    let mut visible = FormState::default();
    visible
        .values
        .push(("--language".to_string(), FlagValue::Dropdown("korean".to_string())));
    let mut hv = render_whole_form_harness(CliTab::Ms, sub, visible);
    hv.run();
    assert_enabled(&hv, "--language");
    let argv_v = argv(CliTab::Ms, "encode", &hv);
    assert!(
        argv_v
            .windows(2)
            .any(|w| w[0] == "--language" && w[1] == "korean"),
        "precondition: a Visible non-default --language reaches argv: {argv_v:?}"
    );

    // Gate ON: seed --hex ⇒ --language Hidden ⇒ its seeded value is suppressed.
    let mut hidden = FormState::default();
    hidden
        .values
        .push(("--language".to_string(), FlagValue::Dropdown("korean".to_string())));
    hidden
        .values
        .push(("--hex".to_string(), FlagValue::Text("00ff".to_string())));
    let mut hh = render_whole_form_harness(CliTab::Ms, sub, hidden);
    hh.run();
    assert_absent(&hh, "--language");
    let argv_h = argv(CliTab::Ms, "encode", &hh);
    assert!(
        !argv_h.iter().any(|t| t == "--language"),
        "a Hidden flag's value must be suppressed from argv: {argv_h:?}"
    );
    assert!(
        !argv_h.iter().any(|t| t == "korean"),
        "the suppressed --language value must not survive: {argv_h:?}"
    );
}

/// Universal value-suppression sweep: for each (sub, gate-state, suppressed
/// flag, seeded value, expected Hidden/Disabled), a real render-settle then
/// `assemble_argv` must DROP the value. Novel vs `argv_assembler_visibility.rs`
/// (hand-built state) in that it asserts AFTER a whole-form render (a render
/// that re-added the value would RED here).
struct SuppressCase {
    tab: CliTab,
    sub: &'static str,
    gate: SeedFn,
    flag: &'static str,
    value: &'static str,
    hidden: bool,
}

#[test]
fn i2_value_suppression_universal_render_settled() {
    let cases = [
        // Disabled legs
        SuppressCase { tab: CliTab::Mnemonic, sub: "compare-cost", gate: s_compare_mini, flag: "--descriptor", value: "wsh(pk(Z))", hidden: false },
        SuppressCase { tab: CliTab::Mk, sub: "encode", gate: s_mk_encode_priv, flag: "--origin-fingerprint", value: "feedface", hidden: false },
        SuppressCase { tab: CliTab::Md, sub: "compile", gate: s_md_compile_segwit, flag: "--unspendable-key", value: "NUMSX", hidden: false },
        // Hidden legs
        SuppressCase { tab: CliTab::Ms, sub: "encode", gate: s_ms_encode_hex, flag: "--language", value: "korean", hidden: true },
        SuppressCase { tab: CliTab::Md, sub: "encode", gate: s_md_encode_positional, flag: "--unspendable-key", value: "NUMSY", hidden: true },
    ];
    for c in cases {
        let sub = sub_of(c.tab, c.sub);
        let mut state = FormState::default();
        seed(&mut state, sub, c.flag, c.value);
        (c.gate)(&mut state, sub);
        let mut h = render_whole_form_harness(c.tab, sub, state);
        h.run();
        // The flag is suppressed in render per its leg…
        if c.hidden {
            assert_absent(&h, c.flag);
        } else {
            assert_disabled(&h, c.flag);
        }
        // …and its value does not reach argv.
        let argv = argv(c.tab, c.sub, &h);
        assert!(
            !argv.iter().any(|t| t == c.flag),
            "{}/{}: {} ({}) must be suppressed from argv: {argv:?}",
            c.tab.bin_name(),
            c.sub,
            c.flag,
            if c.hidden { "Hidden" } else { "Disabled" }
        );
    }
}

// ════════════════════════════════════════════════════════════════════════
// INVARIANT 3 — toggle round-trip (no stuck visibility-state)
// ════════════════════════════════════════════════════════════════════════
// Equivalence is DEFINED over the VISIBILITY-state projection
// (`ui_harness::visibility_projection` = conditional(state) over every flag),
// NOT the stored values (a toggle may legitimately destroy values).

#[test]
fn i3_toggle_roundtrip_md_compile_widget_driven() {
    // Drive the unique --context ComboBox tap→segwitv0→tap→segwitv0 and assert
    // the gated --unspendable-key's RENDERED disabled-state AND the visibility
    // projection both round-trip (no stuck state). --context defaults to "tap".
    let sub = sub_of(CliTab::Md, "compile");
    let mut h = render_whole_form_harness(CliTab::Md, sub, FormState::default());
    h.run();
    assert_enabled(&h, "--unspendable-key"); // baseline (tap)
    let proj_tap = visibility_projection(sub, h.state());

    select_context(&mut h, "segwitv0");
    assert_disabled(&h, "--unspendable-key");
    let proj_seg = visibility_projection(sub, h.state());

    select_context(&mut h, "tap");
    assert_enabled(&h, "--unspendable-key");
    let proj_tap2 = visibility_projection(sub, h.state());

    select_context(&mut h, "segwitv0");
    assert_disabled(&h, "--unspendable-key");
    let proj_seg2 = visibility_projection(sub, h.state());

    assert_ne!(
        proj_tap, proj_seg,
        "the toggle must actually change the visibility projection (non-vacuous)"
    );
    assert_eq!(
        proj_tap, proj_tap2,
        "the 'tap' visibility projection must round-trip (no stuck state)"
    );
    assert_eq!(
        proj_seg, proj_seg2,
        "the 'segwitv0' visibility projection must round-trip (no stuck state)"
    );
}

/// Select `opt` in the (unique) --context combo. egui's ComboBox does NOT
/// auto-close its popup on a `selectable_value` click (verified), so after the
/// first selection the popup stays open — re-clicking the combo would CLOSE it.
/// So: only toggle the combo open when `opt` isn't already a visible popup
/// option. Call contract: `opt` must differ from the current selection (so it
/// is unique to the popup, never colliding with the combo's selected-text
/// label) — the tap↔segwitv0 round-trip honours this.
fn select_context(h: &mut Harness<'static, FormState>, opt: &str) {
    if h.query_by_label(opt).is_none() {
        h.get_by_role(Role::ComboBox).click();
        h.run();
        h.run(); // settle: the popup's option nodes land in the AccessKit tree
    }
    h.get_by_label(opt).click();
    h.run();
    h.run(); // settle: the selection write-back
}

#[test]
fn i3_visibility_projection_roundtrip_universal() {
    // Breadth: for a gate per sub, the ON-projection is path-independent and the
    // ON/OFF projections genuinely differ. Toggling the gating value on the SAME
    // FormState object (push→retain→push) must return the projection to baseline.
    let cases: &[(CliTab, &str, SeedFn, &str)] = &[
        (CliTab::Mnemonic, "compare-cost", s_compare_mini, "--miniscript"),
        (CliTab::Ms, "encode", s_ms_encode_hex, "--hex"),
        (CliTab::Mk, "encode", s_mk_encode_fp, "--origin-fingerprint"),
        (CliTab::Md, "compile", s_md_compile_segwit, "--context"),
        (CliTab::Mnemonic, "export-wallet", s_export_tr, "--template"),
        // slip39-combine: OFF (no --to) already Hides --language (`to.is_none()`),
        // so the non-vacuous toggle is --to=phrase (which UN-hides --language).
        (CliTab::Mnemonic, "slip39-combine", s_slip39_combine_phrase, "--to"),
    ];
    for (tab, sub_name, gate, gate_flag) in cases.iter().copied() {
        let sub = sub_of(tab, sub_name);

        let mut st = FormState::default();
        let proj_off = visibility_projection(sub, &st);

        gate(&mut st, sub); // ON
        let proj_on1 = visibility_projection(sub, &st);

        st.values.retain(|(k, _)| k != gate_flag); // OFF (destroy the gate value)
        let proj_off2 = visibility_projection(sub, &st);

        gate(&mut st, sub); // ON again
        let proj_on2 = visibility_projection(sub, &st);

        assert_ne!(
            proj_off, proj_on1,
            "{sub_name}: toggling `{gate_flag}` must change the visibility projection"
        );
        assert_eq!(
            proj_off, proj_off2,
            "{sub_name}: the OFF visibility projection must round-trip"
        );
        assert_eq!(
            proj_on1, proj_on2,
            "{sub_name}: the ON visibility projection must round-trip (no stuck state)"
        );
    }
}

// ════════════════════════════════════════════════════════════════════════
// INVARIANT 4 — conditional() purity (cheap unit check, not a headline)
// ════════════════════════════════════════════════════════════════════════

#[test]
fn i4_conditional_is_a_pure_fn_of_state() {
    // Same state ⇒ identical FlagVisibility (same entries, same order): no
    // frame/order/interior-mutability dependence. Checked over every conditional
    // sub against a spread of states.
    for tab in CliTab::ALL.iter().copied() {
        for sub in schema_for(tab).subcommands {
            let Some(cf) = sub.conditional else { continue };
            let mut probes: Vec<FormState> = vec![FormState::default()];
            for flag in sub.flags {
                let mut s = FormState::default();
                s.values.push((flag.name.to_string(), default_present(flag)));
                probes.push(s);
            }
            for st in &probes {
                let a = cf(st);
                let b = cf(st);
                assert_eq!(
                    a, b,
                    "{}/{}: conditional() is not a pure fn of state (two calls disagreed)",
                    tab.bin_name(),
                    sub.name
                );
            }
        }
    }
}

// ─── scope sanity: exactly the 17 conditional subcommands ────────────────────

#[test]
fn the_17_conditional_subcommands_are_enumerated() {
    let mut found: Vec<String> = Vec::new();
    for tab in CliTab::ALL.iter().copied() {
        for sub in schema_for(tab).subcommands {
            if sub.conditional.is_some() {
                found.push(format!("{}/{}", tab.bin_name(), sub.name));
            }
        }
    }
    assert_eq!(
        found.len(),
        17,
        "expected exactly 17 conditional subcommands (mnemonic 12 + md 3 + ms 1 + mk 1); got {found:?}"
    );
    // Spot-pin the per-CLI counts.
    let mn = found.iter().filter(|s| s.starts_with("mnemonic/")).count();
    let md = found.iter().filter(|s| s.starts_with("md/")).count();
    let ms = found.iter().filter(|s| s.starts_with("ms/")).count();
    let mk = found.iter().filter(|s| s.starts_with("mk/")).count();
    assert_eq!((mn, md, ms, mk), (12, 3, 1, 1), "per-CLI conditional counts drifted: {found:?}");
}
